/// Constructive Solid Geometry

use super::rt_object::RTObject;
use super::vector::{Vector, Ray, UV};
use super::math_shapes::{MathShape, AddIntersection};
use super::transformation::MatrixTransformation;

#[derive(Clone, Copy)]
pub enum Operator {
    Union,
    Intersection,
    Difference,
}

#[derive(Clone)]
pub struct CSG {
    transformation: MatrixTransformation,
    a_obj: RTObject,
    b_obj: RTObject,
    operator: Operator,
}

impl CSG {
    pub fn new(transformation: MatrixTransformation, a_obj: RTObject, b_obj: RTObject, operator: Operator) -> Self {
        CSG {
            transformation,
            a_obj,
            b_obj,
            operator,
        }
    }

    pub fn get_operation(&self) -> Operator {
        self.operator
    }
}

impl MathShape for CSG {
    fn intersects(&self, ray: Ray, add_intersection: AddIntersection) {
        let a = self.a_obj.get_shape();
        let b = self.b_obj.get_shape();

        match self.operator {
            Operator::Union => {
                let mut check_intersection_1a = |d: f64| {
                    if !b.is_inside(ray.point + ray.direction * d) {
                        add_intersection(d);
                    }
                };

                self.a_obj.intersects(ray.clone(), &mut check_intersection_1a);

                let mut check_intersection_1b = |d: f64| {
                    if !a.is_inside(ray.point + ray.direction * d) {
                        add_intersection(d);
                    }
                };

                self.b_obj.intersects(ray.clone(), &mut check_intersection_1b);
            }
            Operator::Intersection => {
                let mut check_intersection_2a = |d: f64| {
                    if b.is_inside(ray.point + ray.direction * d) {
                        add_intersection(d);
                    }
                };

                self.a_obj.intersects(ray.clone(), &mut check_intersection_2a);

                let mut check_intersection_2b = |d: f64| {
                    if a.is_inside(ray.point + ray.direction * d) {
                        add_intersection(d);
                    }
                };

                self.b_obj.intersects(ray.clone(), &mut check_intersection_2b);
            }
            Operator::Difference => {
                let mut check_intersection_3a = |d: f64| {
                    if !b.is_inside(ray.point + ray.direction * d) {
                        add_intersection(d);
                    }
                };

                self.a_obj.intersects(ray.clone(), &mut check_intersection_3a);

                let mut check_intersection_3b = |d: f64| {
                    if a.is_inside(ray.point + ray.direction * d) {
                        add_intersection(d);
                    }
                };

                self.b_obj.intersects(ray.clone(), &mut check_intersection_3b);
            }
        }
    }

    fn get_normal(&self, surface_point: Vector) -> Vector {
        let a = self.a_obj.get_shape();
        let b = self.b_obj.get_shape();

        match self.operator {
            Operator::Union | Operator::Intersection => {
                if a.is_on_surface(surface_point) {
                    a.get_normal(surface_point)
                } else if b.is_on_surface(surface_point) {
                    b.get_normal(surface_point)
                } else {
                    //panic!("Get CSG normal failed.")
                    Vector::new(1.0, 0.0, 0.0)
                }
            }
            Operator::Difference => {
                if a.is_on_surface(surface_point) {
                    a.get_normal(surface_point)
                } else if b.is_on_surface(surface_point) {
                    b.get_normal(surface_point)
                } else {
                    // FIXME: Weird, why doesn't this panic like the one above?
                    Vector::new(1.0, 0.0, 0.0)
                }
            }
        }
    }

    fn is_inside(&self, point: Vector) -> bool {
        let a = self.a_obj.get_shape();
        let b = self.b_obj.get_shape();

        match self.operator {
            Operator::Union => a.is_inside(point) || b.is_inside(point),
            Operator::Intersection => a.is_inside(point) && b.is_inside(point),
            Operator::Difference => a.is_inside(point) && !b.is_inside(point),
        }
    }

    fn is_on_surface(&self, point: Vector) -> bool {
        let a = self.a_obj.get_shape();
        let b = self.b_obj.get_shape();

        match self.operator {
            Operator::Union => {
                (a.is_on_surface(point) && !b.is_inside(point)) ||
                    (b.is_on_surface(point) && !a.is_inside(point))
            }
            Operator::Intersection => {
                (a.is_on_surface(point) && b.is_inside(point)) ||
                    (b.is_on_surface(point) && a.is_inside(point))
            }
            Operator::Difference => {
                (a.is_on_surface(point) && !b.is_inside(point)) ||
                    (b.is_on_surface(point) && a.is_inside(point))
            }
        }
    }

    fn get_uv_coordinates(&self, point: Vector) -> Result<UV, &'static str> {
        let a = self.a_obj.get_shape();
        let b = self.b_obj.get_shape();

        if a.is_on_surface(point) {
            a.get_uv_coordinates(point)
        } else if b.is_on_surface(point) {
            b.get_uv_coordinates(point)
        } else {
            Err("CSG's get_uv_coordinates called outside of a surface!")
        }
    }

    fn set_transformation(&mut self, transformation: MatrixTransformation) {
        self.transformation = transformation;
    }

    fn get_transformation(&self) -> &MatrixTransformation {
        &self.transformation
    }

    fn reverse_transform_ray(&self, ray: Ray) -> Ray {
        // CSG objects themselves do not have transformations.
        ray
    }

    fn clone_box(&self) -> Box<dyn MathShape> {
        Box::new(self.clone())
    }
}