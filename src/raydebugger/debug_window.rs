use crate::sceneparser::scene_loader::load_scene;
use crate::raytracer::raytracer::RayTracer;
use crate::raytracer::color::{Color, ColorPixmap, RaytracerPixmap};
use crate::raytracer::vector::{Vector, Ray};
use crate::raytracer::math::INFINITY;
use crate::raytracer::antialiaser::AntiAliaser;
use super::easy_pixbuf::EasyPixbuf;
use super::gui::{DrawingArea, MAX_FRAMES};
use super::ray_debugger::OrthoAxes;

use glib::Sender;
use threadpool::ThreadPool;
use std::sync::Arc;

pub struct RenderedLine {
    pub frame: usize,
    pub area: DrawingArea,
    pub line: usize,
    pub rendered_line: Vec<Color>,
    pub anti_aliased: bool,
    pub size: (usize, usize),
}

pub type RenderedLineSender = Sender<RenderedLine>;

pub const ANTIALIAS_THRESHOLD: f64 = 0.01;
pub const ANTIALIAS_LEVEL: i32 = 3;

#[derive(Clone)]
pub struct DebugWindow {
    ray_tracer: Arc<RayTracer>,
    width: usize,
    height: usize,
    show_anti_aliasing_edges: bool,
    antialiasing_threshold: f64,
    antialiasing_level: i32,
    antialiased_lines: Vec<bool>,
}

impl DebugWindow {
    pub fn new(width: usize, height: usize, frame: usize) -> Self {
        DebugWindow {
            ray_tracer: Arc::new(Self::load_ray_tracer(width, height, frame)),
            width,
            height,
            show_anti_aliasing_edges: false,
            antialiasing_threshold: ANTIALIAS_THRESHOLD,
            antialiasing_level: ANTIALIAS_LEVEL,
            antialiased_lines: vec![false; height],
        }
    }

    fn load_ray_tracer(width: usize, height: usize, frame: usize) -> RayTracer {
        let mut ray_tracer = RayTracer::new_default(width, height);
        ray_tracer.add_test_objects();
        // FIXME: Max frames
        let time = frame as f64 / MAX_FRAMES as f64;
        if let Err(err) = load_scene(&mut ray_tracer, time) {
            eprintln!("Error parsing scene: {}", err);
        }
        ray_tracer
    }

    pub fn reload_ray_tracer(&mut self, frame: usize, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.ray_tracer = Arc::new(Self::load_ray_tracer(self.width, self.height, frame));
    }

    pub fn ray_tracer(&self) -> &RayTracer {
        &self.ray_tracer
    }

    pub fn render_lines<'a>(
        &'a self, line_range: Vec<usize>
    ) -> impl Iterator<Item=(usize, Vec<Color>)> + 'a {
        line_range
            .into_iter()
            .map(move |y| {
                let line: Vec<Color> = (0..self.width)
                    .map(|x| {
                        self.ray_tracer.get_pixel(x as f64, y as f64, &mut None)
                    })
                    .collect();
                (y, line)
            })
    }

    pub fn render_ortho_lines<'a>(
        &'a self, ortho_axes: OrthoAxes
    ) -> impl Iterator<Item=(usize, Vec<Color>)> + 'a {
        (0..self.height)
            .map(move |y| {
                (y, self.render_orthogonal_view_line(y, ortho_axes))
            })
    }

    pub fn set_anti_aliasing_threshold(&mut self, threshold: f64) {
        self.antialiasing_threshold = threshold;
    }

    pub fn set_show_anti_aliasing_edges(&mut self, show_edges: bool) {
        self.show_anti_aliasing_edges = show_edges;
    }

    pub fn clear_anti_aliased_edges_line(&self, y: usize, edge_pixels: &mut [u8]) {
        let mut edge_pixbuf = EasyPixbuf::new(
            self.width, self.height, self.width * 4, 4, &mut edge_pixels[..]
        );

        for x in 0..self.width {
            edge_pixbuf.set_pixel_color(x, y, Color::EMPTY);
        }
    }

    pub fn check_anti_aliasing_threshold(&self, scene: &mut [u8], edge_pixels: &mut [u8]) {
        let scene_pixbuf = EasyPixbuf::new(
            self.width, self.height, self.width * 4, 4, &mut scene[..]
        );
        let mut edge_pixbuf = EasyPixbuf::new(
            self.width, self.height, self.width * 4, 4, &mut edge_pixels[..]
        );

        edge_pixbuf.fill_with_color(Color::EMPTY);

        if self.show_anti_aliasing_edges {
            let mut mark_pixel = |x, y| {
                if !self.antialiased_lines[y as usize] {
                    edge_pixbuf.set_pixel_color(
                        x, y, Color::new(0.6, 1.0, 1.0, 0.5)
                    );
                }
            };

            AntiAliaser::mark_edge_pixels(
                self.antialiasing_threshold, &scene_pixbuf, &mut mark_pixel
            );
        }
    }

    pub fn set_line_anti_aliased(&mut self, y: usize, anti_aliased: bool) -> bool {
        let changed = self.antialiased_lines[y] != anti_aliased;
        self.antialiased_lines[y] = anti_aliased;
        changed
    }

    pub fn apply_line(&self, y: usize, rendered_line: &Vec<Color>, pixels: &mut [u8]) {
        let mut scene_pixbuf = EasyPixbuf::new(
            self.width, self.height, self.width * 4, 4, &mut pixels[..]
        );

        assert_eq!(rendered_line.len(), self.width);

        for (x, color) in rendered_line.iter().enumerate() {
            scene_pixbuf.set_pixel_color(x, y, *color);
        }

        // Make the next line white to show the progress.
        if y + 1 < self.height {
            for x in 0..self.width {
                scene_pixbuf.set_pixel_color(x, y + 1, Color::WHITE);
            }
        }
    }

    pub fn render_orthogonal_view_line(
        &self, y: usize, ortho_axes: OrthoAxes,
    ) -> Vec<Color> {
        let center_x = self.width as f64 / 2.0;
        let center_y = self.height as f64 / 2.0;

        let axis3 = if ortho_axes.axis1 != 0 && ortho_axes.axis2 != 0 {
            0
        } else if ortho_axes.axis1 != 1 && ortho_axes.axis2 != 1 {
            1
        } else if ortho_axes.axis1 != 2 && ortho_axes.axis2 != 2 {
            2
        } else {
            panic!("Invalid axes");
        };

        let mut direction = Vector::new(0.0, 0.0, 0.0);
        *direction.axis_mut(axis3) = 1.0;

        let get_origin_for_pixel = |x, y| {
            let mut origin = Vector::new(0.0, 0.0, 0.0);
            *origin.axis_mut(ortho_axes.axis1) =
                ((x - center_x) * ortho_axes.dir1) / ortho_axes.scale;
            *origin.axis_mut(ortho_axes.axis2) =
                ((y - center_y) * ortho_axes.dir2) / ortho_axes.scale;
            *origin.axis_mut(axis3) = 10000.0;
            origin
        };

        let mut rendered_line = Vec::with_capacity(self.width);

        for x in 0..self.width {
            // FIXME: Move to a cast_ray inside the RayTracer
            let ray = Ray {
                point: get_origin_for_pixel(x as f64, y as f64),
                direction,
            };
            let mut foremost_object = None;
            let mut distance = INFINITY;

            for object in self.ray_tracer.get_objects() {
                // FIXME: Skip planes
                let mut add_intersection = |d: f64| {
                    if d < distance {
                        foremost_object = Some(object);
                        distance = d;
                    }
                };

                object.intersects(ray.clone(), &mut add_intersection);
            }

            let color = if let Some(foremost_object) = foremost_object {
                foremost_object.get_color()
            } else {
                Color::EMPTY
            };
            rendered_line.push(color);
        }

        rendered_line
    }

    pub fn create_rendering_thread(
        &self, thread_pool: &ThreadPool, frame: usize, line_range: Vec<usize>,
        area: DrawingArea, rendered_line_sender: RenderedLineSender
    ) {
        // Clone the entire ray tracer and send it to another thread
        let debug_window = self.clone();

        thread_pool.execute(move || {
            match area {
                DrawingArea::MainView => {
                    for (y, rendered_line) in debug_window.render_lines(line_range) {
                        let rendered_line = RenderedLine {
                            frame,
                            area,
                            line: y,
                            rendered_line,
                            anti_aliased: false,
                            size: (debug_window.width, debug_window.height),
                        };
                        if let Err(_) = rendered_line_sender.send(rendered_line) {
                            // Exit if main thread is no longer interested.
                            break;
                        }
                    }
                },
                area => {
                    let ortho_axes: OrthoAxes = area.into();
                    for (y, rendered_line) in debug_window.render_ortho_lines(ortho_axes) {
                        let rendered_line = RenderedLine {
                            frame,
                            area,
                            line: y,
                            rendered_line,
                            anti_aliased: false,
                            size: (debug_window.width, debug_window.height),
                        };
                        if let Err(_) = rendered_line_sender.send(rendered_line) {
                            // Exit if main thread is no longer interested.
                            break;
                        }
                    }
                },
            }
        });
    }

    pub fn create_anti_aliasing_thread(
        &self, thread_pool: &ThreadPool, frame: usize, rendered_line_sender: RenderedLineSender, scene: &mut [u8]
    ) {
        // Clone the entire ray tracer and send it to another thread
        let debug_window = self.clone();

        // Also clone the entire scene
        let scene_pixbuf = EasyPixbuf::new(
            self.width, self.height, self.width * 4,
            4, scene
        );
        let cloned_scene = RaytracerPixmap::from_color_pixmap(&scene_pixbuf);

        thread_pool.execute(move || {
            let anti_aliaser = AntiAliaser::new(
                &debug_window.ray_tracer,
                Some(debug_window.antialiasing_threshold),
                Some(debug_window.antialiasing_level)
            );

            let mut sub_pixels = anti_aliaser.create_sub_pixel_buffer();
            let mut ray_counter = 0;

            for y in 0..debug_window.height - 1 {
                let rendered_line = anti_aliaser.anti_alias_line_vec(
                    y, &mut sub_pixels, &mut ray_counter, &cloned_scene
                );

                let rendered_line = RenderedLine {
                    frame,
                    area: DrawingArea::MainView,
                    line: y,
                    rendered_line,
                    anti_aliased: true,
                    size: (debug_window.width, debug_window.height),
                };

                if let Err(_) = rendered_line_sender.send(rendered_line) {
                    // Exit if main thread is no longer interested.
                    break;
                }
            }

            println!("Additional rays traced for anti-aliasing: {}.", ray_counter);
        });
    }
}
