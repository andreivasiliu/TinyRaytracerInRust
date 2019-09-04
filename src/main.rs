#![allow(dead_code)]

use cairo;
use gtk::prelude::*;
use gtk::{CheckButton, Inhibit, Window, WindowType};
use std::cell::RefCell;
use std::rc::Rc;

mod raytracer;
mod raydebugger;

const WIDTH: i32 = 480;
const HEIGHT: i32 = 360;


fn main() {
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

        for i in 0..(HEIGHT * WIDTH * 4) {
            surface_data[i as usize] = 100;
        }

        let mut debug_window = raydebugger::debug_window::DebugWindow::new(
            WIDTH as usize,
            HEIGHT as usize,
            surface_data,
        );

        debug_window.render_frame();
    }
    let surface = Rc::new(RefCell::new(surface));
    let surface_clone = surface.clone();

    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_size_request(WIDTH, HEIGHT);
    drawing_area.connect_draw(move |_widget, context: &cairo::Context| {
        let surface = surface_clone.borrow();
        context.set_source_surface(&*surface, 0.0, 0.0);
        context.paint();

        Inhibit(false)
    });

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&hbox, false, false, 0);
    vbox.pack_start(&drawing_area, true, true, 0);

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
