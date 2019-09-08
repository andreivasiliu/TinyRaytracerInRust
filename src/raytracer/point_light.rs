use super::color::Color;
use super::vector::Vector;

#[derive(Clone)]
pub struct PointLight {
    point: Vector,
    color: Color,
    fade_distance: f64,
}

impl PointLight {
    pub fn new(point: Vector, color: Color, fade_distance: f64) -> Self {
        PointLight {
            point,
            color,
            fade_distance,
        }
    }

    pub fn get_point(&self) -> &Vector {
        &self.point
    }

    pub fn get_color(&self) -> &Color {
        &self.color
    }

    pub fn fade_distance(&self) -> f64 {
        self.fade_distance
    }

    /// Fade power
    pub fn intensity(&self, distance: f64) -> f64 {
        if distance >= self.fade_distance {
            0.0
        } else {
            distance / self.fade_distance
        }
    }
}