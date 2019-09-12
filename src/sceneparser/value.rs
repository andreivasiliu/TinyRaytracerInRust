use super::shape::Shape;
use super::texture::Texture;

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
}
