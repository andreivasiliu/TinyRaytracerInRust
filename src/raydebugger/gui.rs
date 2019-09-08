use super::debug_window::{DebugWindow, ANTIALIAS_THRESHOLD};
use super::ray_debugger::RayDebugger;

use cairo;
use gtk::prelude::*;
use gtk::{CheckButton, Inhibit};
use gdk::EventMask;
use glib;
use std::cell::RefCell;
use std::rc::Rc;
use gio::{ApplicationExt, ApplicationExtManual};

const WIDTH: i32 = 480;
const HEIGHT: i32 = 360;

#[derive(Clone, Copy)]
pub enum DrawingArea {
    MainView,
    TopView,
    FrontView,
    SideView,
}

struct DebuggerContext {
    debug_window: DebugWindow,
    ray_debugger: RayDebugger,
    button_down: bool,
    main_surface: cairo::ImageSurface,
    top_surface: cairo::ImageSurface,
    front_surface: cairo::ImageSurface,
    side_surface: cairo::ImageSurface,
    edge_pixels: cairo::ImageSurface,
}

impl DebuggerContext {
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

    let debugger_context = Rc::new(RefCell::new(DebuggerContext {
        debug_window,
        ray_debugger,
        button_down: false,
        main_surface,
        top_surface,
        front_surface,
        side_surface,
        edge_pixels,
    }));

    // Create the main window.
    let window = gtk::ApplicationWindow::new(application);
    window.set_title("Ray Debugger");

    let (rendered_line_sender, rendered_line_receiver) =
        glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let top_debug_area = gtk::DrawingArea::new();
    top_debug_area.set_size_request(WIDTH, HEIGHT);
    top_debug_area.connect_draw({
        let debugger_context = debugger_context.clone();
        move |_widget, context: &cairo::Context| {
            let debugger_context = debugger_context.borrow();

            debugger_context.ray_debugger.draw_ortho_view(
                context, &debugger_context.top_surface, DrawingArea::TopView
            );

            Inhibit(false)
        }
    });

    let front_debug_area = gtk::DrawingArea::new();
    front_debug_area.set_size_request(WIDTH, HEIGHT);
    front_debug_area.connect_draw({
        let debugger_context = debugger_context.clone();
        move |_widget, context: &cairo::Context| {
            let debugger_context = debugger_context.borrow();

            debugger_context.ray_debugger.draw_ortho_view(
                context, &debugger_context.front_surface, DrawingArea::FrontView
            );

            Inhibit(false)
        }
    });

    let side_debug_area = gtk::DrawingArea::new();
    side_debug_area.set_size_request(WIDTH, HEIGHT);
    side_debug_area.connect_draw({
        let debugger_context = debugger_context.clone();
        move |_widget, context: &cairo::Context| {
            let debugger_context = debugger_context.borrow();

            debugger_context.ray_debugger.draw_ortho_view(
                context, &debugger_context.side_surface, DrawingArea::SideView
            );

            Inhibit(false)
        }
    });

    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_size_request(WIDTH, HEIGHT);
    drawing_area.add_events(
        EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_MOTION_MASK
    );
    drawing_area.connect_draw({
        let debugger_context = debugger_context.clone();
        move |_widget, context: &cairo::Context| {
            let debugger_context = debugger_context.borrow();

            context.set_source_surface(&*debugger_context.main_surface, 0.0, 0.0);
            context.paint();

            context.set_source_surface(&*debugger_context.edge_pixels, 0.0, 0.0);
            context.paint();

            Inhibit(false)
        }
    });

    drawing_area.connect_button_press_event({
        let debugger_context = debugger_context.clone();
        let top_debug_area = top_debug_area.clone();
        let front_debug_area = front_debug_area.clone();
        let side_debug_area = side_debug_area.clone();
        move |_widget, event| {
            let mut debugger_context = debugger_context.borrow_mut();

            let (x, y) = event.get_position();
            debugger_context.record_rays(x, y);
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
        move |_widget, event| {
            let mut debugger_context = debugger_context.borrow_mut();
            if debugger_context.button_down {
                let (x, y) = event.get_position();
                debugger_context.record_rays(x, y);
                top_debug_area.queue_draw();
                front_debug_area.queue_draw();
                side_debug_area.queue_draw();
            }

            Inhibit(false)
        }
    });

    let raytrace_ortho_views_button =
        gtk::CheckButton::new_with_label("Raytrace orthogonal views");
    raytrace_ortho_views_button.connect_clicked({
        let debugger_context = debugger_context.clone();
        let rendered_line_sender = rendered_line_sender.clone();
        move |button| {
            let debug_window = &mut debugger_context.borrow_mut().debug_window;
            debug_window.raytrace_ortho_views = button.get_active();

            if debug_window.raytrace_ortho_views {
                debug_window.create_rendering_thread(
                    DrawingArea::TopView, rendered_line_sender.clone()
                );

                debug_window.create_rendering_thread(
                    DrawingArea::FrontView, rendered_line_sender.clone()
                );

                debug_window.create_rendering_thread(
                    DrawingArea::SideView, rendered_line_sender.clone()
                );
            }
        }
    });

    let threshold_scale =
        gtk::Scale::new_with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.01);
    threshold_scale.set_digits(2);
    threshold_scale.set_draw_value(true);
    threshold_scale.set_value(ANTIALIAS_THRESHOLD);
    threshold_scale.set_value_pos(gtk::PositionType::Left);

    threshold_scale.connect_value_changed({
        let debugger_context = debugger_context.clone();
        let drawing_area = drawing_area.clone();
        move |scale| {
            let debugger_context = &mut debugger_context.borrow_mut();

            debugger_context.debug_window.set_anti_aliasing_threshold(scale.get_value());
            debugger_context.check_anti_aliasing();
            drawing_area.queue_draw();
        }
    });

    let show_anti_alias_edges_button =
        CheckButton::new_with_label("Show edges");

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
            let mut debugger_context = debugger_context.borrow_mut();
            debugger_context.debug_window.set_show_anti_aliasing_edges(show_edges);
            debugger_context.check_anti_aliasing();
            drawing_area.queue_draw();
        }
    });

    rendered_line_receiver.attach(None, {
        let debugger_context = debugger_context.clone();
        let drawing_area = drawing_area.clone();
        let top_debug_area = top_debug_area.clone();
        let front_debug_area = front_debug_area.clone();
        let side_debug_area = side_debug_area.clone();

        move |(area, y, rendered_line, anti_aliased)| {
            let mut debugger_context = debugger_context.borrow_mut();
            let debugger_context: &mut DebuggerContext = &mut debugger_context;

            match area {
                DrawingArea::MainView => {
                    debugger_context.set_line_anti_aliased(y, anti_aliased);
                    let surface_data: &mut [u8] = &mut debugger_context.main_surface.get_data().unwrap();
                    debugger_context.debug_window.apply_line(y, &rendered_line, surface_data);
                    drawing_area.queue_draw();
                }
                DrawingArea::TopView => {
                    let surface_data: &mut [u8] = &mut debugger_context.top_surface.get_data().unwrap();

                    debugger_context.debug_window.apply_line(y, &rendered_line, surface_data);
                    top_debug_area.queue_draw();
                }
                DrawingArea::FrontView => {
                    let surface_data: &mut [u8] = &mut debugger_context.front_surface.get_data().unwrap();

                    debugger_context.debug_window.apply_line(y, &rendered_line, surface_data);
                    front_debug_area.queue_draw();
                }
                DrawingArea::SideView => {
                    let surface_data: &mut [u8] = &mut debugger_context.side_surface.get_data().unwrap();

                    debugger_context.debug_window.apply_line(y, &rendered_line, surface_data);
                    side_debug_area.queue_draw();
                }
            }

            glib::Continue(true)
        }
    });

    let render_button = gtk::Button::new_with_label("Render");
    render_button.connect_clicked({
        let debugger_context = debugger_context.clone();
        let rendered_line_sender = rendered_line_sender.clone();
        move |_button| {
            let debug_window = &debugger_context.borrow().debug_window;

            debug_window.create_rendering_thread(
                DrawingArea::MainView, rendered_line_sender.clone()
            );

            if debug_window.raytrace_ortho_views {
                debug_window.create_rendering_thread(
                    DrawingArea::TopView, rendered_line_sender.clone()
                );

                debug_window.create_rendering_thread(
                    DrawingArea::FrontView, rendered_line_sender.clone()
                );

                debug_window.create_rendering_thread(
                    DrawingArea::SideView, rendered_line_sender.clone()
                );
            }
        }
    });

    let anti_alias_button = gtk::Button::new_with_label("Anti-alias");
    anti_alias_button.connect_clicked({
        let debugger_context = debugger_context.clone();
        let rendered_line_sender = rendered_line_sender.clone();
        move |_button| {
            let mut debugger_context = debugger_context.borrow_mut();
            let debugger_context: &mut DebuggerContext = &mut debugger_context;
            let surface_data: &mut [u8] = &mut debugger_context.main_surface.get_data().unwrap();

            let rendered_line_sender = rendered_line_sender.clone();
            debugger_context.debug_window.create_anti_aliasing_thread(
                rendered_line_sender, surface_data
            );
        }
    });

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    hbox.pack_end(&render_button, false, false, 0);
    hbox.pack_end(&anti_alias_button, false, false, 0);
    hbox.pack_end(&threshold_scale, true, true, 10);
    hbox.pack_start(&raytrace_ortho_views_button, false, true, 0);
    hbox.pack_start(&show_anti_alias_edges_button, false, true, 0);

    let hbox_top = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    hbox_top.pack_start(&drawing_area, true, true, 1);
    hbox_top.pack_start(&top_debug_area, true, true, 1);

    let hbox_bottom = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    hbox_bottom.pack_start(&side_debug_area, true, true, 1);
    hbox_bottom.pack_start(&front_debug_area, true, true, 1);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&hbox, false, false, 0);
    vbox.pack_start(&hbox_top, false, false, 1);
    vbox.pack_start(&hbox_bottom, false, false, 1);

    window.add(&vbox);

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
    debugger_context.borrow().debug_window
        .create_rendering_thread(
            DrawingArea::MainView,
            rendered_line_sender.clone()
        );

    // Run the main loop.
    gtk::main();
}
