use crate::raytracer::color::{RaytracerPixmap, ColorPixmap, Color};

use lodepng;
use std::fmt::{Debug, Formatter, Error};
use std::rc::Rc;

#[derive(Clone)]
pub struct Texture {
    pixmap: Rc<RaytracerPixmap>,
    filename: String,
}

impl Debug for Texture {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Texture {{ filename: {:?} }}", self.filename)
    }
}

impl Texture {
    pub fn from_file(filename: &str) -> Self {
        // FIXME: No unwrap
        let png = lodepng::decode32_file(filename).unwrap();

        let mut pixmap = RaytracerPixmap::new(png.width, png.height);

        for x in 0..png.width {
            for y in 0..png.height {
                let pixel = png.buffer[y * png.width + x];
                let color = Color::new(
                    pixel.r as f64 / 255.0,
                    pixel.g as f64 / 255.0,
                    pixel.b as f64 / 255.0,
                    pixel.a as f64 / 255.0,
                );
                pixmap.set_pixel_color(x, y, color);
            }
        }

        Texture { pixmap: Rc::new(pixmap), filename: filename.to_owned() }
    }

    pub fn pixmap(&self) -> &RaytracerPixmap {
        &*self.pixmap
    }
}