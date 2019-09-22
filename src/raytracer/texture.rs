use super::vector::UV;
use super::color::{Color, RaytracerPixmap, ColorPixmap};

pub trait Texture: Send + Sync {
    fn get_color_at(&self, uv_coordinates: UV) -> Color;
    fn clone_box(&self) -> Box<dyn Texture>;
}

impl Clone for Box<dyn Texture> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Clone)]
pub struct PixmapTexture {
    pixmap: RaytracerPixmap,
}

impl PixmapTexture {
    pub fn from_pixmap(pixmap: RaytracerPixmap) -> Self {
        PixmapTexture { pixmap }
    }
}

impl Texture for PixmapTexture {
    fn get_color_at(&self, uv_coordinates: UV) -> Color {
        let width = self.pixmap.get_width();
        let height = self.pixmap.get_height();
        let x = uv_coordinates.u * (width - 1) as f64;
        let y = height as f64 - (uv_coordinates.v * (height - 1) as f64) - 1.0;

        self.pixmap.get_pixel_color(x as usize, y as usize)
    }

    fn clone_box(&self) -> Box<dyn Texture> {
        Box::new(self.clone())
    }
}
