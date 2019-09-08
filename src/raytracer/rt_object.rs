use super::vector::Ray;
use super::color::Color;
use super::material::{Material, SolidColorMaterial};
use super::math_shapes::{MathShape, AddIntersection};

#[derive(Clone)]
pub struct RTObject {
    shape: Box<dyn MathShape>,
    material: Box<dyn Material>,
}

impl RTObject {
    pub fn new(shape: Box<dyn MathShape>, material: Option<Box<dyn Material>>) -> Self {
        RTObject {
            shape,
            material: material.unwrap_or_else(|| Box::new(SolidColorMaterial::new(
                Color::BLACK, 0.0, 0.0
            ))),
        }
    }

    pub fn new_default(shape: Box<dyn MathShape>) -> Self {
        RTObject::new(shape, Some(Box::new(SolidColorMaterial::new(
            Color::new(0.0, 1.0, 1.0, 1.0), 0.0, 0.0
        ))))
    }

    pub fn intersects(&self, ray: Ray, add_intersection: AddIntersection) {
        let transformed_ray = self.shape.reverse_transform_ray(ray);
        self.shape.intersects(transformed_ray, add_intersection);
    }

    pub fn get_material(&self) -> &Box<dyn Material> {
        &self.material
    }

    pub fn get_shape(&self) -> &Box<dyn MathShape> {
        &self.shape
    }

    pub fn get_shape_mut(&mut self) -> &mut Box<dyn MathShape> {
        &mut self.shape
    }

    pub fn get_color(&self) -> Color {
        self.material.get_color_at(0.0, 0.0)
    }
}

