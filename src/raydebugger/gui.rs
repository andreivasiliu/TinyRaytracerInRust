use super::debug_window::{DebugWindow, RenderedLineSender, ANTIALIAS_THRESHOLD};
use super::ray_debugger::RayDebugger;

use cairo;
use gtk::prelude::*;
use gtk::{CheckButton, Inhibit};
use gdk::EventMask;
use glib;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use gio::{ApplicationExt, ApplicationExtManual};

const WIDTH: i32 = 480;
const HEIGHT: i32 = 360;

pub const MAX_FRAMES: usize = 30;

#[derive(Clone, Copy)]
pub enum DrawingArea {
    MainView,
    TopView,
    FrontView,
    SideView,
}

struct DebuggerContext {
    button_down: bool,
    raytrace_ortho_views: bool,
    current_frame: usize,
    animating: bool,
    frames: Vec<FrameContext>,
}

impl DebuggerContext {
    pub fn new() -> Self {
        let mut frames = Vec::new();

        for _ in 0..MAX_FRAMES {
            frames.push(FrameContext::new());
        }

        DebuggerContext {
            button_down: false,
            raytrace_ortho_views: false,
            current_frame: 0,
            animating: false,
            frames,
        }
    }

    pub fn frame(&mut self) -> &mut FrameContext {
        &mut self.frames[self.current_frame]
    }

    pub fn render_all_frames(
        &mut self, rendered_line_sender: RenderedLineSender
    ) {
        for frame in 0..MAX_FRAMES {
            self.frames[frame].render_frame(
                self.raytrace_ortho_views,
                frame, rendered_line_sender.clone(),
            )
        }
    }

    pub fn render_all_ortho_frames(
        &mut self, rendered_line_sender: RenderedLineSender
    ) {
        for frame in 0..MAX_FRAMES {
            self.frames[frame].render_ortho_frame(
                frame, rendered_line_sender.clone(),
            )
        }
    }

    pub fn anti_alias_all_frames(
        &mut self, rendered_line_sender: RenderedLineSender
    ) {
        for frame in 0..MAX_FRAMES {
            self.frames[frame].anti_alias_frame(
                frame, rendered_line_sender.clone(),
            )
        }
    }
}

fn frame(context: &Rc<RefCell<DebuggerContext>>) -> RefMut<FrameContext> {
    RefMut::map(
        context.borrow_mut(),
        |context| &mut context.frames[context.current_frame]
    )
}

struct FrameContext {
    debug_window: DebugWindow,
    ray_debugger: RayDebugger,
    main_surface: cairo::ImageSurface,
    top_surface: cairo::ImageSurface,
    front_surface: cairo::ImageSurface,
    side_surface: cairo::ImageSurface,
    edge_pixels: cairo::ImageSurface,
}

impl FrameContext {
    fn new() -> Self {
        let debug_window = DebugWindow::new(
            WIDTH as usize,
            HEIGHT as usize,
        );

        let ray_debugger = RayDebugger::new(
            WIDTH,
            HEIGHT,
        );

        // Top-left
        let main_surface = cairo::ImageSurface::create(cairo::Format::Rgb24, WIDTH, HEIGHT).unwrap();

        // Top-right
        let top_surface = cairo::ImageSurface::create(cairo::Format::Rgb24, WIDTH, HEIGHT).unwrap();

        // Bottom-right
        let front_surface = cairo::ImageSurface::create(cairo::Format::Rgb24, WIDTH, HEIGHT).unwrap();

        // Bottom-left
        let side_surface = cairo::ImageSurface::create(cairo::Format::Rgb24, WIDTH, HEIGHT).unwrap();

        // Brighten some pixels to show which pixels will be anti-aliased
        let edge_pixels = cairo::ImageSurface::create(cairo::Format::ARgb32, WIDTH, HEIGHT).unwrap();

        FrameContext {
            debug_window,
            ray_debugger,
            main_surface,
            top_surface,
            front_surface,
            side_surface,
            edge_pixels
        }
    }

    pub fn record_rays(&mut self, x: f64, y: f64) {
        let ray_tracer = self.debug_window.ray_tracer();
        self.ray_debugger.record_rays(ray_tracer, x, y);
    }

    pub fn check_anti_aliasing(&mut self) {
        let surface_data: &mut [u8] = &mut self.main_surface.get_data().unwrap();
        let edge_data: &mut [u8] = &mut self.edge_pixels.get_data().unwrap();
        self.debug_window.check_anti_aliasing_threshold(surface_data, edge_data);
    }

    pub fn set_line_anti_aliased(&mut self, y: usize, anti_aliased: bool) {
        if anti_aliased {
            let edge_data: &mut [u8] = &mut self.edge_pixels.get_data().unwrap();
            self.debug_window.clear_anti_aliased_edges_line(y, edge_data)
        }
    }

    pub fn render_frame(
        &mut self, raytrace_ortho_views: bool, current_frame: usize,
        rendered_line_sender: RenderedLineSender,
    ) {
        self.debug_window.reload_ray_tracer(current_frame);

        if raytrace_ortho_views {
            self.debug_window.create_rendering_thread(
                current_frame, DrawingArea::TopView, rendered_line_sender.clone()
            );

            self.debug_window.create_rendering_thread(
                current_frame, DrawingArea::FrontView, rendered_line_sender.clone()
            );

            self.debug_window.create_rendering_thread(
                current_frame, DrawingArea::SideView, rendered_line_sender.clone()
            );
        }

        self.debug_window.create_rendering_thread(
            current_frame, DrawingArea::MainView, rendered_line_sender
        );
    }

    pub fn render_ortho_frame(
        &mut self, current_frame: usize, rendered_line_sender: RenderedLineSender,
    ) {
        self.debug_window.create_rendering_thread(
            current_frame, DrawingArea::TopView, rendered_line_sender.clone()
        );

        self.debug_window.create_rendering_thread(
            current_frame, DrawingArea::FrontView, rendered_line_sender.clone()
        );

        self.debug_window.create_rendering_thread(
            current_frame, DrawingArea::SideView, rendered_line_sender
        );
    }

    pub fn anti_alias_frame(
        &mut self, current_frame: usize, rendered_line_sender: RenderedLineSender,
    ) {
        let surface_data: &mut [u8] = &mut self.main_surface.get_data().unwrap();

        self.debug_window.create_anti_aliasing_thread(
            current_frame, rendered_line_sender, surface_data
        );
    }
}

pub fn run_application() {
    let application = gtk::Application::new(
        Some("com.github.andreivasiliu.tinyraytracerinrust"),
        Default::default(),
    ).expect("Could not create GTK application!");

    application.connect_activate(|app| {
        build_gui(app);
    });

    let args: Vec<String> = std::env::args().collect();
    application.run(&args);
}

fn build_gui(application: &gtk::Application) {
    let debugger_context
        = Rc::new(RefCell::new(DebuggerContext::new()));

    // Create the main window.
    let window = gtk::ApplicationWindow::new(application);
    window.set_title("Ray Debugger");

    let (rendered_line_sender, rendered_line_receiver) =
        glib::MainContext::channel(glib::PRIORITY_DEFAULT_IDLE);

    let top_debug_area = gtk::DrawingArea::new();
    top_debug_area.set_size_request(WIDTH, HEIGHT);

    let front_debug_area = gtk::DrawingArea::new();
    front_debug_area.set_size_request(WIDTH, HEIGHT);

    let side_debug_area = gtk::DrawingArea::new();
    side_debug_area.set_size_request(WIDTH, HEIGHT);

    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_size_request(WIDTH, HEIGHT);
    drawing_area.add_events(
        EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_MOTION_MASK
    );

    let hbox_top = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    hbox_top.pack_start(&drawing_area, true, true, 1);
    hbox_top.pack_start(&top_debug_area, true, true, 1);

    let hbox_bottom = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    hbox_bottom.pack_start(&side_debug_area, true, true, 1);
    hbox_bottom.pack_start(&front_debug_area, true, true, 1);

    let show_ortho_views_button =
        gtk::CheckButton::new_with_label("Show orthogonal views");
    show_ortho_views_button.set_active(true);

    let raytrace_ortho_views_button =
        gtk::CheckButton::new_with_label("Raytrace orthogonal views");

    let animate_button =
        gtk::CheckButton::new_with_label("Animate");

    let frame_spin_button =
    gtk::SpinButton::new_with_range(0.0, MAX_FRAMES as f64 - 1.0, 1.0);

    let threshold_scale =
        gtk::Scale::new_with_range(gtk::Orientation::Horizontal, 0.0, 0.1, 0.001);
    threshold_scale.set_digits(3);
    threshold_scale.set_draw_value(true);
    threshold_scale.set_value(ANTIALIAS_THRESHOLD);
    threshold_scale.set_value_pos(gtk::PositionType::Left);

    let show_anti_alias_edges_button =
        CheckButton::new_with_label("Show edges");

    let anti_alias_button = gtk::Button::new_with_label("Anti-alias");

    let render_button = gtk::Button::new_with_label("Render");

    // First bar:
    // [Show ortho] [Raytrace ortho] [Animate] [Show edges] Threshold: [----O----------] <Render>

    // Second bar:
    // Frame: [  0]+- Frame: [----O---------] Max Frames: [ 10]+- [Loop] <Render All>

    let hbox_bar_1 = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    hbox_bar_1.pack_end(&render_button, false, false, 0);
    hbox_bar_1.pack_end(&anti_alias_button, false, false, 0);
    hbox_bar_1.pack_end(&frame_spin_button, false, false, 0);
    hbox_bar_1.pack_end(&threshold_scale, true, true, 10);
    hbox_bar_1.pack_start(&show_ortho_views_button, false, true, 0);
    hbox_bar_1.pack_start(&raytrace_ortho_views_button, false, true, 0);
    hbox_bar_1.pack_start(&animate_button, false, true, 0);
    hbox_bar_1.pack_start(&show_anti_alias_edges_button, false, true, 0);

    //let hbox_bar_2 = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    //hbox_bar_2.pack_start(&frame_spin_button, false, false, 0);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&hbox_bar_1, false, false, 0);
    //vbox.pack_start(&hbox_bar_2, false, false, 0);
    vbox.pack_start(&hbox_top, true, true, 1);
    vbox.pack_start(&hbox_bottom, true, true, 1);

    window.add(&vbox);

    top_debug_area.connect_draw({
        let debugger_context = debugger_context.clone();
        move |widget, context: &cairo::Context| {
            let frame = frame(&debugger_context);

            // Scale to occupy the whole drawing area
            let width = widget.get_allocated_width();
            let height = widget.get_allocated_height();
            context.scale(width as f64 / WIDTH as f64, height as f64 / HEIGHT as f64);

            frame.ray_debugger.draw_ortho_view(
                context, &frame.top_surface, DrawingArea::TopView
            );

            Inhibit(false)
        }
    });
    front_debug_area.connect_draw({
        let debugger_context = debugger_context.clone();
        move |widget, context: &cairo::Context| {
            let frame = frame(&debugger_context);

            // Scale to occupy the whole drawing area
            let width = widget.get_allocated_width();
            let height = widget.get_allocated_height();
            context.scale(width as f64 / WIDTH as f64, height as f64 / HEIGHT as f64);

            frame.ray_debugger.draw_ortho_view(
                context, &frame.front_surface, DrawingArea::FrontView
            );

            Inhibit(false)
        }
    });

    side_debug_area.connect_draw({
        let debugger_context = debugger_context.clone();
        move |widget, context: &cairo::Context| {
            let frame = frame(&debugger_context);

            // Scale to occupy the whole drawing area
            let width = widget.get_allocated_width();
            let height = widget.get_allocated_height();
            context.scale(width as f64 / WIDTH as f64, height as f64 / HEIGHT as f64);

            frame.ray_debugger.draw_ortho_view(
                context, &frame.side_surface, DrawingArea::SideView
            );

            Inhibit(false)
        }
    });

    drawing_area.connect_draw({
        let debugger_context = debugger_context.clone();
        let show_anti_alias_edges_button = show_anti_alias_edges_button.clone();
        move |widget, context: &cairo::Context| {
            let frame = frame(&debugger_context);

            // Scale to occupy the whole drawing area
            let width = widget.get_allocated_width();
            let height = widget.get_allocated_height();
            if width != WIDTH || height != HEIGHT {
                context.scale(width as f64 / WIDTH as f64, height as f64 / HEIGHT as f64);
            }

            // Paint the raytraced image
            context.set_source_surface(&*frame.main_surface, 0.0, 0.0);
            context.paint();

            if show_anti_alias_edges_button.get_active() {
                // Highlight which pixels would be anti-aliased
                context.set_source_surface(&*frame.edge_pixels, 0.0, 0.0);
                context.paint();
            }

            Inhibit(false)
        }
    });

    drawing_area.connect_button_press_event({
        let debugger_context = debugger_context.clone();
        let top_debug_area = top_debug_area.clone();
        let front_debug_area = front_debug_area.clone();
        let side_debug_area = side_debug_area.clone();
        move |widget, event| {
            let mut debugger_context = debugger_context.borrow_mut();

            let (x, y) = event.get_position();

            let width = widget.get_allocated_width();
            let height = widget.get_allocated_height();

            let x = x * (WIDTH as f64 / width as f64);
            let y = y * (HEIGHT as f64 / height as f64);

            debugger_context.frame().record_rays(x, y);
            debugger_context.button_down = true;
            top_debug_area.queue_draw();
            front_debug_area.queue_draw();
            side_debug_area.queue_draw();

            Inhibit(false)
        }
    });

    drawing_area.connect_button_release_event({
        let debugger_context = debugger_context.clone();
        move |_widget, _event| {
            debugger_context.borrow_mut().button_down = false;

            Inhibit(false)
        }
    });

    drawing_area.connect_motion_notify_event({
        let debugger_context = debugger_context.clone();
        let top_debug_area = top_debug_area.clone();
        let front_debug_area = front_debug_area.clone();
        let side_debug_area = side_debug_area.clone();
        move |widget, event| {
            let mut debugger_context = debugger_context.borrow_mut();
            if debugger_context.button_down {
                let (x, y) = event.get_position();

                let width = widget.get_allocated_width();
                let height = widget.get_allocated_height();

                let x = x * (WIDTH as f64 / width as f64);
                let y = y * (HEIGHT as f64 / height as f64);

                debugger_context.frame().record_rays(x, y);
                top_debug_area.queue_draw();
                front_debug_area.queue_draw();
                side_debug_area.queue_draw();
            }

            Inhibit(false)
        }
    });

    show_ortho_views_button.connect_clicked({
        let top_debug_area = top_debug_area.clone();
        let front_debug_area = front_debug_area.clone();
        let side_debug_area = side_debug_area.clone();
        let hbox_bottom = hbox_bottom.clone();
        let raytrace_ortho_views_button = raytrace_ortho_views_button.clone();
        move |button| {
            let show_ortho_views = button.get_active();

            top_debug_area.set_visible(show_ortho_views);
            front_debug_area.set_visible(show_ortho_views);
            side_debug_area.set_visible(show_ortho_views);
            hbox_bottom.set_visible(show_ortho_views);
            raytrace_ortho_views_button.set_visible(show_ortho_views);
        }
    });

    raytrace_ortho_views_button.connect_clicked({
        let debugger_context = debugger_context.clone();
        let rendered_line_sender = rendered_line_sender.clone();
        move |button| {
            debugger_context.borrow_mut().raytrace_ortho_views = button.get_active();
            let animating = debugger_context.borrow().animating;

            if button.get_active() {
                if animating {
                    debugger_context.borrow_mut().render_all_ortho_frames(
                        rendered_line_sender.clone()
                    )
                } else {
                    let current_frame = debugger_context.borrow().current_frame;
                    frame(&debugger_context).render_ortho_frame(
                        current_frame, rendered_line_sender.clone()
                    );
                }
            }
        }
    });

    animate_button.connect_clicked({
        let debugger_context = debugger_context.clone();
        let frame_spin_button = frame_spin_button.clone();

        move |button| {
            if button.get_active() {
                gtk::timeout_add(33, {
                    let debugger_context = debugger_context.clone();
                    let frame_spin_button = frame_spin_button.clone();

                    move || {
                        let current_frame = debugger_context.borrow().current_frame;
                        frame_spin_button.set_value(((current_frame + 1) % MAX_FRAMES) as f64);

                        Continue(debugger_context.borrow().animating)
                    }
                });
            }

            debugger_context.borrow_mut().animating = button.get_active();
        }
    });

    frame_spin_button.connect_value_changed({
        let debugger_context = debugger_context.clone();
        let drawing_area = drawing_area.clone();
        let top_debug_area = top_debug_area.clone();
        let front_debug_area = front_debug_area.clone();
        let side_debug_area = side_debug_area.clone();
        move |spin_button| {
            debugger_context.borrow_mut().current_frame = spin_button.get_value() as usize;
            drawing_area.queue_draw();
            top_debug_area.queue_draw();
            front_debug_area.queue_draw();
            side_debug_area.queue_draw();
        }
    });

    threshold_scale.connect_value_changed({
        let debugger_context = debugger_context.clone();
        let drawing_area = drawing_area.clone();
        move |scale| {
            let mut frame = frame(&debugger_context);

            frame.debug_window.set_anti_aliasing_threshold(scale.get_value());
            frame.check_anti_aliasing();
            drawing_area.queue_draw();
        }
    });

    show_anti_alias_edges_button.connect_clicked({
        let debugger_context = debugger_context.clone();
        let drawing_area = drawing_area.clone();
        let threshold_scale = threshold_scale.clone();

        move |button| {
            let show_edges = if button.get_active() {
                threshold_scale.show();
                true
            } else {
                threshold_scale.hide();
                false
            };
            let mut frame = frame(&debugger_context);
            frame.debug_window.set_show_anti_aliasing_edges(show_edges);
            frame.check_anti_aliasing();
            drawing_area.queue_draw();
        }
    });

    rendered_line_receiver.attach(None, {
        let debugger_context = debugger_context.clone();
        let drawing_area = drawing_area.clone();
        let top_debug_area = top_debug_area.clone();
        let front_debug_area = front_debug_area.clone();
        let side_debug_area = side_debug_area.clone();

        move |(rendered_frame, area, y, rendered_line, anti_aliased)| {
            let current_frame = debugger_context.borrow().current_frame;
            let frame = &mut debugger_context.borrow_mut().frames[rendered_frame];

            match area {
                DrawingArea::MainView => {
                    frame.set_line_anti_aliased(y, anti_aliased);
                    let surface_data: &mut [u8] = &mut frame.main_surface.get_data().unwrap();
                    frame.debug_window.apply_line(y, &rendered_line, surface_data);
                }
                DrawingArea::TopView => {
                    let surface_data: &mut [u8] = &mut frame.top_surface.get_data().unwrap();

                    frame.debug_window.apply_line(y, &rendered_line, surface_data);
                }
                DrawingArea::FrontView => {
                    let surface_data: &mut [u8] = &mut frame.front_surface.get_data().unwrap();

                    frame.debug_window.apply_line(y, &rendered_line, surface_data);
                }
                DrawingArea::SideView => {
                    let surface_data: &mut [u8] = &mut frame.side_surface.get_data().unwrap();

                    frame.debug_window.apply_line(y, &rendered_line, surface_data);
                }
            }

            if rendered_frame == current_frame {
                side_debug_area.queue_draw();
                front_debug_area.queue_draw();
                drawing_area.queue_draw();
                top_debug_area.queue_draw();
            }

            glib::Continue(true)
        }
    });

    render_button.connect_clicked({
        let debugger_context = debugger_context.clone();
        let rendered_line_sender = rendered_line_sender.clone();
        move |_button| {
            let animating = debugger_context.borrow().animating;

            if animating {
                debugger_context.borrow_mut().render_all_frames(rendered_line_sender.clone());
            } else {
                let raytrace_ortho_views = debugger_context.borrow().raytrace_ortho_views;
                let current_frame = debugger_context.borrow().current_frame;

                frame(&debugger_context).render_frame(
                    raytrace_ortho_views, current_frame, rendered_line_sender.clone()
                );
            }
        }
    });

    anti_alias_button.connect_clicked({
        let debugger_context = debugger_context.clone();
        let rendered_line_sender = rendered_line_sender.clone();
        move |_button| {
            let animating = debugger_context.borrow().animating;

            if animating {
                debugger_context.borrow_mut().anti_alias_all_frames(
                    rendered_line_sender.clone()
                );
            } else {
                let current_frame = debugger_context.borrow().current_frame;
                frame(&debugger_context).anti_alias_frame(
                    current_frame, rendered_line_sender.clone()
                );
            }
        }
    });

    // Don't forget to make all widgets visible.
    window.show_all();
    threshold_scale.hide();

    window.add_events(EventMask::KEY_PRESS_MASK);
    window.connect_key_press_event({
        let window = window.clone();
        move |_window, event| {
            if event.get_keyval() == gdk::enums::key::Escape {
                window.close();
            }
            Inhibit(true)
        }
    });

    // Handle closing of the window.
    window.connect_delete_event(|_, _| {
        // Stop the main loop.
        gtk::main_quit();
        // Let the default handler destroy the window.
        Inhibit(false)
    });

    // Kick off a threaded render so the user doesn't have to
    debugger_context.borrow_mut().frame().debug_window
        .create_rendering_thread(
            0,
            DrawingArea::MainView,
            rendered_line_sender.clone()
        );

    // Run the main loop.
    gtk::main();
}
