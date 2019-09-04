pub use std::f64::consts::PI;
pub const EPSILON: f64 = 10e-7;
pub use std::f64::INFINITY;
pub use std::f64::NEG_INFINITY;

pub fn sin(x: f64) -> f64 {
    x.sin()
}

pub fn cos(x: f64) -> f64 {
    x.cos()
}

pub fn acos(x: f64) -> f64 {
    x.acos()
}

pub fn abs(x: f64) -> f64 {
    x.abs()
}

pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}
