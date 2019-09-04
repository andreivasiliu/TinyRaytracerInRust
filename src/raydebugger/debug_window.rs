use crate::raytracer::raytracer::RayTracer;
use crate::raytracer::color::ColorPixmap;
use super::easy_pixbuf::EasyPixbuf;

pub struct DebugWindow<'a> {
    ray_tracer: RayTracer,
    pixels: &'a mut [u8],
    width: usize,
    height: usize,
}

impl<'a> DebugWindow<'a> {
    pub fn new(width: usize, height: usize, pixels: &'a mut [u8]) -> Self {
        let mut ray_tracer = RayTracer::new_default(width, height);
        ray_tracer.add_test_objects();

        DebugWindow {
            ray_tracer,
            width,
            height,
            pixels,
        }
    }

    pub fn render_frame(&mut self) {
        // FIXME: ???
        let mut scene_pixbuf = EasyPixbuf::new(
            self.width, self.height, self.width * 4, 4, &mut self.pixels[..]
        );

        for y in 0..self.height {
            for x in 0..self.width {
                let color = self.ray_tracer.get_pixel(x as f64, y as f64, &mut None);
                scene_pixbuf.set_pixel_color(x, y, color);
            }
        }
    }
}
