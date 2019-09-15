use super::texture::Texture;
use super::vector::UV;
use super::color::Color;

pub trait Material: Send + Sync {
    fn get_color_at(&self, u: f64, v: f64) -> Color;
    fn get_reflectivity_at(&self, u: f64, v: f64) -> f64;
    fn get_transparency_at(&self, u: f64, v: f64) -> f64;

    fn get_color_at_uv(&self, uv_coordinates: UV) -> Color
    {
        return self.get_color_at(uv_coordinates.u, uv_coordinates.v);
    }

    fn get_reflectivity_at_uv(&self, uv_coordinates: UV) -> f64
    {
        return self.get_reflectivity_at(uv_coordinates.u, uv_coordinates.v);
    }

    fn get_transparency_at_uv(&self, uv_coordinates: UV) -> f64
    {
        return self.get_transparency_at(uv_coordinates.u, uv_coordinates.v);
    }

    fn clone_box(&self) -> Box<dyn Material>;
}

impl Clone for Box<dyn Material> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Clone)]
pub struct SolidColorMaterial {
    color: Color,
    reflectivity: f64,
    transparency: f64,
}

impl SolidColorMaterial {
    pub fn new(color: Color, reflectivity: f64, transparency: f64) -> Self {
        SolidColorMaterial {
            color,
            reflectivity,
            transparency,
        }
    }
}

impl Material for SolidColorMaterial {
    fn get_color_at(&self, _u: f64, _v: f64) -> Color {
        self.color
    }

    fn get_reflectivity_at(&self, _u: f64, _v: f64) -> f64 {
        self.reflectivity
    }

    fn get_transparency_at(&self, _u: f64, _v: f64) -> f64 {
        self.transparency
    }

    fn clone_box(&self) -> Box<dyn Material> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct TexturedMaterial {
    texture: Box<dyn Texture>,
    reflectivity: f64,
    transparency: f64,
}

impl TexturedMaterial {
    pub fn new(texture: Box<dyn Texture>, reflectivity: f64, transparency: f64) -> Self {
        TexturedMaterial {
            texture,
            reflectivity,
            transparency,
        }
    }
}

impl Material for TexturedMaterial {
    fn get_color_at(&self, u: f64, v: f64) -> Color {
        self.texture.get_color_at(UV { u, v })
    }

    fn get_reflectivity_at(&self, _u: f64, _v: f64) -> f64 {
        self.reflectivity
    }

    fn get_transparency_at(&self, _u: f64, _v: f64) -> f64 {
        self.transparency
    }

    fn clone_box(&self) -> Box<dyn Material> {
        Box::new(self.clone())
    }
}