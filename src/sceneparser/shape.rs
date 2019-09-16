use crate::raytracer::rt_object::RTObject;
use crate::raytracer::material::SolidColorMaterial;
use crate::raytracer::color::Color;
use crate::raytracer::vector::Vector;
use crate::raytracer::math_shapes::{MathSphere, MathCube, MathPlane};
use crate::raytracer::transformation::MatrixTransformation;
use crate::raytracer::csg::{CSG, Operator};

#[derive(Debug, Clone)]
pub struct Shape {
    pub color: Color,
    pub reflectivity: f64,
    pub transparency: f64,
    pub kind: ShapeKind,
    pub transformation: MatrixTransformation,
}

#[derive(Debug, Clone)]
pub enum ShapeKind {
    Sphere { center: Vector, radius: f64 },
    Cube { center: Vector, length: f64 },
    Plane { normal: Vector, distance: f64 },
    CSG { operator: CSGOperator, a: Box<Shape>, b: Box<Shape> },
}

#[derive(Debug, Clone)]
pub enum CSGOperator {
    Intersection,
    Union,
    Difference,
}

impl Shape {
    pub fn to_rt_object(&self) -> RTObject {
        RTObject::new(
            match self.kind {
                ShapeKind::Sphere { center, radius } => {
                    Box::new(MathSphere::new(
                        self.transformation.clone(), center, radius
                    ))
                }
                ShapeKind::Cube { center, length } => {
                    Box::new(MathCube::new(
                        self.transformation.clone(), center, length
                    ))
                },
                ShapeKind::Plane { normal, distance } => {
                    Box::new(MathPlane::from_normal(
                        self.transformation.clone(), normal, distance
                    ))
                },
                ShapeKind::CSG { ref operator, ref a, ref b } => {
                    let a = a.to_rt_object();
                    let b = b.to_rt_object();

                    let operator = match operator {
                        CSGOperator::Intersection => Operator::Intersection,
                        CSGOperator::Union => Operator::Union,
                        CSGOperator::Difference => Operator::Difference,
                    };

                    Box::new(CSG::new(
                        self.transformation.clone(), a, b, operator
                    ))
                }
            },
            Some(Box::new(SolidColorMaterial::new(
                self.color, self.reflectivity, self.transparency
            ))),
        )
    }
}
