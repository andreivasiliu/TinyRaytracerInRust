use crate::raytracer::rt_object::RTObject;
use crate::raytracer::material::SolidColorMaterial;
use crate::raytracer::color::Color;
use crate::raytracer::vector::Vector;
use crate::raytracer::math_shapes::MathSphere;
use crate::raytracer::transformation::MatrixTransformation;

#[derive(Debug, Clone)]
pub struct Shape {
    pub color: Option<(f64, f64, f64, f64)>,
    pub reflectivity: f64,
    pub transparency: f64,
    pub kind: ShapeKind,
}

#[derive(Debug, Clone)]
pub enum ShapeKind {
    Sphere { center: (f64, f64, f64), radius: f64 },
    Cube { length: f64 },
    Plane { normal: (f64, f64, f64), distance: f64 },
    // CSG { operator,  }
}

impl Shape {
    pub fn to_rt_object(&self, transformation: MatrixTransformation) -> RTObject {
        match self.kind {
            ShapeKind::Sphere { center, radius } => {
                let center = Vector::new(center.0, center.1, center.2);
                let color = self.color.unwrap_or((0.0, 0.0, 0.0, 1.0));
                let color = Color::new(color.0, color.1, color.2, color.3);

                RTObject::new(
                    Box::new(MathSphere::new(transformation, center, radius)),
                    Some(Box::new(SolidColorMaterial::new(color, self.reflectivity, self.transparency)))
                )
            }
            ShapeKind::Cube { .. } => unimplemented!(),
            ShapeKind::Plane { .. } => unimplemented!(),
        }
    }
}
