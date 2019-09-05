#![allow(dead_code)]

use cairo;
use gtk::prelude::*;
use gtk::{CheckButton, Inhibit, Window, WindowType};
use gdk::EventMask;
use std::cell::RefCell;
use std::rc::Rc;

mod raytracer;
mod raydebugger;

const WIDTH: i32 = 480;
const HEIGHT: i32 = 360;

struct DebuggerContext {
    debug_window: raydebugger::debug_window::DebugWindow,
    ray_debugger: raydebugger::ray_debugger::RayDebugger,
    button_down: bool,
}

impl DebuggerContext {
    pub fn record_rays(&mut self, x: f64, y: f64) {
        let ray_tracer = self.debug_window.ray_tracer();
        self.ray_debugger.record_rays(ray_tracer, x, y);
        println!("{} rays recorded.", self.ray_debugger.rays.len());
    }
}

fn main() {

    let debug_window = raydebugger::debug_window::DebugWindow::new(
        WIDTH as usize,
        HEIGHT as usize,
    );

    let ray_debugger = raydebugger::ray_debugger::RayDebugger::new(
        WIDTH,
        HEIGHT,
    );

    let original_debugger_context = Rc::new(RefCell::new(DebuggerContext {
        debug_window,
        ray_debugger,
        button_down: false,
    }));

    gtk::init().unwrap();
    // Create the main window.
    let window = Window::new(WindowType::Toplevel);
    window.set_title("Ray Debugger");

    let threshold_scale = gtk::Scale::new_with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.01);
    threshold_scale.set_digits(2);
    threshold_scale.set_draw_value(true);
    threshold_scale.set_value(0.1);
    threshold_scale.set_value_pos(gtk::PositionType::Left);
    let show_anti_alias_edges_button = CheckButton::new_with_label("Show edges");

    let threshold_scale_clone = threshold_scale.clone();
    show_anti_alias_edges_button.connect_clicked(move |button| {
        if button.get_active() {
            threshold_scale_clone.show();
        } else {
            threshold_scale_clone.hide();
        }
    });

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    hbox.pack_end(&threshold_scale, true, true, 10);
    hbox.pack_start(&show_anti_alias_edges_button, false, true, 0);

    let mut surface = cairo::ImageSurface::create(cairo::Format::Rgb24, WIDTH, HEIGHT).unwrap();
    {
        let surface_data: &mut [u8] = &mut surface.get_data().unwrap();

        original_debugger_context.borrow().debug_window.render_frame(surface_data);
    }
    let surface = Rc::new(RefCell::new(surface));
    let surface_clone = surface.clone();

    let debug_area = gtk::DrawingArea::new();
    debug_area.set_size_request(WIDTH, HEIGHT);
    let debugger_context = original_debugger_context.clone();
    debug_area.connect_draw(move |_widget, context: &cairo::Context| {
        let ray_debugger = &debugger_context.borrow().ray_debugger;
        let (axis_x, axis_y, _axis_z) = (0, 1, 2);
        ray_debugger.draw_grid(context, 2.0);
        ray_debugger.draw_objects(context, axis_x, axis_y, 1.0, -1.0, 2.0);

        Inhibit(false)
    });

    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_size_request(WIDTH, HEIGHT);
    drawing_area.add_events(
        (EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_MOTION_MASK).bits() as i32
    );
    drawing_area.connect_draw(move |_widget, context: &cairo::Context| {
        let surface = surface_clone.borrow();
        context.set_source_surface(&*surface, 0.0, 0.0);
        context.paint();

        Inhibit(false)
    });

    let debug_area_clone = debug_area.clone();
    let debugger_context = original_debugger_context.clone();
    drawing_area.connect_button_press_event(move |_widget, event| {
        let mut debugger_context = debugger_context.borrow_mut();

        let (x, y) = event.get_position();
        debugger_context.record_rays(x, y);
        debugger_context.button_down = true;
        debug_area_clone.queue_draw();

        Inhibit(false)
    });

    let debugger_context = original_debugger_context.clone();
    drawing_area.connect_button_release_event(move |_widget, _event| {
        debugger_context.borrow_mut().button_down = false;

        Inhibit(false)
    });

    let debugger_context = original_debugger_context.clone();
    drawing_area.connect_motion_notify_event(move |_widget, _event| {
        let debugger_context = debugger_context.borrow_mut();
        if debugger_context.button_down {
            
        }

        Inhibit(false)
    });

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&hbox, false, false, 0);
    vbox.pack_start(&drawing_area, true, true, 0);
    vbox.pack_start(&debug_area, true, true, 0);

    window.add(&vbox);

    // Don't forget to make all widgets visible.
    window.show_all();
    threshold_scale.hide();
    // Handle closing of the window.
    window.connect_delete_event(|_, _| {
        // Stop the main loop.
        gtk::main_quit();
        // Let the default handler destroy the window.
        Inhibit(false)
    });

    // Run the main loop.
    gtk::main();
}
