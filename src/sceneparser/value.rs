use super::shape::Shape;
use super::texture::Texture;
use crate::raytracer::vector::Vector;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    Color { r: f64, g: f64, b: f64, a: f64 },
    Vector { x: f64, y: f64, z: f64 },
    Object(Shape),
    Texture(Texture),
}

impl Value {
    pub fn to_number(&self) -> f64 {
        match self {
            Value::Number(number) => *number,
            // FIXME: no panic
            value => panic!("Cannot convert value to number: {:?}", value),
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            Value::Boolean(boolean) => *boolean,
            // FIXME: no panic
            value => panic!("Cannot convert value to boolean: {:?}", value),
        }
    }

    pub fn to_vector(&self) -> Vector {
        match self {
            Value::Vector { x, y, z } => Vector::new(*x, *y, *z),
            // FIXME: no panic
            value => panic!("Cannot convert value to vector: {:?}", value),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::String(string) => string.to_owned(),
            // FIXME: no panic
            value => panic!("Cannot convert value to string: {:?}", value),
        }
    }
}
