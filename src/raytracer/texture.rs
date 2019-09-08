use super::vector::UV;
use super::color::{Color, RaytracerPixmap, ColorPixmap};

pub trait Texture: Send {
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
        let x = uv_coordinates.u as usize * (width - 1);
        let y = height - (uv_coordinates.v as usize * (height - 1)) - 1;

        self.pixmap.get_pixel_color(x, y)
    }

    fn clone_box(&self) -> Box<dyn Texture> {
        Box::new(self.clone())
    }
}
