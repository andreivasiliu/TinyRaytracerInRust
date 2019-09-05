use crate::raytracer::vector::Vector;
use crate::raytracer::transformation::{MatrixTransformation, Transformation};

use arr_macro::arr;

pub enum DebugShape {
    Cube {
        center: Vector,
        length: f64,
        points: [Vector; 8],
    },
    Sphere {
        center: Vector,
        radius: f64,
    },
}

impl DebugShape {
    fn get_cube_points(
        center: Vector, length: f64, transformation: MatrixTransformation
    ) -> [Vector; 8] {
        let mut points: [Vector; 8] = arr![Vector::new(0.0, 0.0, 0.0); 8];

        // Make points on the cube's corners.
        for i in 0..8 {
            points[i] = center;

            for axis in 0..3 {
                if i & (1 << axis) != 0 {
                    *points[i].axis_mut(axis) += length;
                } else {
                    *points[i].axis_mut(axis) -= length;
                }
            }
        }

        // And apply the object's current transformation on them.
        for i in 0..8 {
            points[i] = transformation.transform_vector(points[i]);
        }

        points
    }

    pub fn draw<F>(&self, draw_line: F)
    where
        F: Fn(Vector, Vector),
    {
        match self {
            DebugShape::Cube {points, .. } => {
                let corners = [0, 3, 5, 6];

                for corner in corners.iter() {
                    for axis in 0..3 {
                        draw_line(points[*corner], points[corner ^ (1 << axis)]);
                    }
                }
            },
            DebugShape::Sphere { .. } => {
                // Spheres have no lines
            }
        }
    }
}
