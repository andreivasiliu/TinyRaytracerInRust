use crate::raytracer::vector::{Vector, Ray};
use crate::raytracer::color::Color;
use crate::raytracer::raytracer::{RayTracer, RayType};
use crate::raytracer::rt_object::RTObject;
use crate::raytracer::math::INFINITY;
use super::debug_shape::DebugShape;

use cairo;

pub struct RayInfo {
    depth: i32,
    ray: Ray,
    color: Color,
    ray_type: RayType,
    intersection_distance: f64,
    intersected: bool,
    intersection_point: Vector,
    normal: Option<Vector>,
}

pub struct RayDebugger {
    pub shapes: Vec<DebugShape>,
    pub rays: Vec<RayInfo>,

    width: i32,
    height: i32,
    show_normals: bool,
}

impl RayDebugger {
    pub fn new(width: i32, height: i32) -> Self {
        RayDebugger {
            shapes: vec![],
            rays: vec![],
            width,
            height,
            show_normals: true,
        }
    }

    pub fn record_rays(&mut self, ray_tracer: &RayTracer, x: f64, y: f64) {
        self.rays.clear();

        let mut ray_debugger_callback = |
            depth: i32, ray: Ray, intersection_distance: f64, intersected_object: Option<&RTObject>,
            color: &Color, ray_type: &RayType
        | {
            let intersected = intersection_distance != INFINITY;

            let intersection_point = ray.point + ray.direction * if intersected {
                intersection_distance
            } else {
                1000.0
            };

            let normal = intersected_object.map(|obj| obj
                .get_shape()
                .get_normal(intersection_point)
            );

            let ray_info = RayInfo {
                depth,
                ray,
                intersection_distance,
                color: *color,
                ray_type: *ray_type,
                normal,
                intersected,
                intersection_point,
            };

            self.rays.push(ray_info);
        };

        ray_tracer.get_pixel(x, y, &mut Some(&mut ray_debugger_callback));
    }

    pub fn draw_grid(&self, context: &cairo::Context, scale: f64) {
        let width = self.width as f64;
        let height = self.height as f64;
        let center_x = width / 2.0;
        let center_y = height / 2.0;

        context.save();

        context.set_source_rgb(0.1, 0.1, 0.1);
        context.paint();

        context.set_source_rgb(0.6, 0.2, 0.6);
        context.set_line_width(0.1);

        for x in (-self.width..self.width).step_by(10) {
            let x = x as f64;
            context.move_to(center_x + (x * scale), center_y - height);
            context.line_to(center_x + (x * scale), center_y + height);
            context.stroke();
        }

        for y in (-self.height..self.height).step_by(10) {
            let y = y as f64;
            context.move_to(center_x - width, center_y + (y * scale));
            context.line_to(center_x + width, center_y + (y * scale));
            context.stroke();
        }

        context.restore();
    }

    pub fn draw_objects(
        &self, context: &cairo::Context, axis1: usize, axis2: usize,
        dir1: f64, dir2: f64, scale: f64
    ) {
        let draw_line = |from: Vector, to: Vector| {
            let center_x = self.width as f64 / 2.0;
            let center_y = self.height as f64 / 2.0;

            context.move_to(
                center_x + scale * dir1 * from.axis(axis1),
                center_y + scale * dir2 * from.axis(axis2),
            );
            context.line_to(
                center_x + scale * dir1 * to.axis(axis1),
                center_y + scale * dir2 * to.axis(axis2),
            );
            context.stroke();
        };

        // Shapes
        context.save();
        context.set_line_width(1.0);

        for shape in self.shapes.iter() {
            shape.draw(draw_line);
        }

        context.restore();

        // Rays
        context.save();
        context.set_line_width(1.0);

        for ray_info in self.rays.iter() {
            // Show the normal.
            if ray_info.intersected && self.show_normals {
                if let Some(normal) = ray_info.normal {
                    context.set_source_rgb(1.0, 0.0, 1.0);
                    let temp = ray_info.intersection_point + normal * 10.0;
                    draw_line(ray_info.intersection_point, temp);
                }
            }

            // And the ray
            match ray_info.ray_type {
                RayType::NormalRay => context.set_source_rgb(1.0, 0.0, 0.0),
                RayType::ReflectionRay => context.set_source_rgb(0.0, 1.0, 0.0),
                RayType::TransmissionRay => context.set_source_rgb(0.0, 0.0, 1.0),
            }

            draw_line(ray_info.ray.point, ray_info.intersection_point);
        }

        context.restore();
    }
}