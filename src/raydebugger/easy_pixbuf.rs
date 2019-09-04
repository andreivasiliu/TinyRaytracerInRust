use crate::raytracer::color::{Color, ColorPixmap};

pub struct EasyPixbuf<'a> {
    width: usize,
    height: usize,

    row_stride: usize,
    n_channels: usize,
    pixels: &'a mut [u8],
}

impl<'a> EasyPixbuf<'a> {
    pub fn new(
        width: usize, height: usize, row_stride: usize, n_channels: usize, pixels: &'a mut [u8]
    ) -> Self {
        EasyPixbuf {
            width,
            height,

            row_stride,
            n_channels,
            pixels,
        }
    }

    fn get_pixel_slice(&self, x: usize, y: usize) -> &[u8] {
        let pos = y * self.row_stride + x * self.n_channels;
        &self.pixels[pos..pos+self.n_channels]
    }

    fn get_pixel_slice_mut(&mut self, x: usize, y: usize) -> &mut [u8] {
        let pos = y * self.row_stride + x * self.n_channels;
        &mut self.pixels[pos..pos+self.n_channels]
    }
}

impl ColorPixmap for EasyPixbuf<'_> {
    fn get_width(&self) -> usize {
        self.width
    }

    fn get_height(&self) -> usize {
        self.height
    }

    fn set_pixel_color(&mut self, x: usize, y: usize, color: Color) {
        let pixel = self.get_pixel_slice_mut(x, y);

        pixel[2] = (color.r * 255.0) as u8;
        pixel[1] = (color.g * 255.0) as u8;
        pixel[0] = (color.b * 255.0) as u8;
    }

    fn get_pixel_color(&self, x: usize, y: usize) -> Color {
        let pixel = self.get_pixel_slice(x, y);

        Color::new(
            pixel[2] as f64 / 255.0,
            pixel[1] as f64 / 255.0,
            pixel[0] as f64 / 255.0,
            1.0
        )
    }
}
