use super::raytracer::RayTracer;
use super::color::{Color, ColorPixmap};
use super::math::abs;

type MarkerDelegate<'a> = &'a dyn Fn(usize, usize);
type SubPixels = Vec<Vec<Option<Color>>>;

struct AntiAliaser<'a> {
    ray_tracer: &'a RayTracer,
    // FIXME: Big source of dynamic dispatches; maybe change to RaytracerPixmap
    // Then copy results to wherever else; for now though, probably not easy
    source: &'a dyn ColorPixmap,
    anti_aliased_destination: &'a mut dyn ColorPixmap,
    threshold: f64,
    level: i32,
    size: usize,
}

impl<'a> AntiAliaser<'a> {
    pub fn new(
        ray_tracer: &'a RayTracer, source: &'a dyn ColorPixmap, destination: &'a mut dyn ColorPixmap,
        threshold: Option<f64>, level: Option<i32>,
    ) -> Self {
        let threshold = threshold.unwrap_or(0.1);
        let level = level.unwrap_or(3);
        let size= (1 << level) as usize + 1;

        AntiAliaser {
            ray_tracer,
            source,
            anti_aliased_destination: destination,
            threshold,
            level,
            size,
        }
    }

    pub fn set_threshold(&mut self, threshold: f64) {
        self.threshold = threshold;
    }

    // TODO: Find a proper name for these methods.
    pub fn anti_alias_all(&mut self, sub_pixels: &mut SubPixels, ray_counter: &mut i32) {
        for y in 0..(self.source.get_height() - 1) {
            self.anti_alias_line(y, sub_pixels, ray_counter);
        }
    }

    pub fn anti_alias_line(&mut self, y: usize, sub_pixels: &mut SubPixels, ray_counter: &mut i32) {
        let last_pixel = self.source.get_width() - 1;
        for x in 0..last_pixel {
            let color = self.get_anti_aliased_pixel(x, y, sub_pixels, ray_counter);
            self.anti_aliased_destination.set_pixel_color(x, y, color);
        }

        // Copy the last pixel.
        self.anti_aliased_destination.set_pixel_color(
            last_pixel, y, self.source.get_pixel_color(last_pixel, y),
        );
    }

    pub fn clear_matrices(&self, sub_pixels: &mut SubPixels) {
        assert_eq!(sub_pixels.len(), self.size);
        for y in 0..self.size {
            assert_eq!(sub_pixels[y].len(), self.size);
            for x in 0..self.size {
                sub_pixels[x][y] = None;
            }
        }
    }

    pub fn get_anti_aliased_pixel(
        &mut self, x: usize, y: usize, sub_pixels: &mut SubPixels, ray_counter: &mut i32
    ) -> Color {
        self.clear_matrices(sub_pixels);

        // Set up this pixel's subpixel matrix.
        // Note that which dimension is Y and which is X does not actually
        // matter; the result is the same.
        sub_pixels[0][0] = Some(self.source.get_pixel_color(x, y));
        sub_pixels[0][self.size - 1] = Some(self.source.get_pixel_color(x, x+1));
        sub_pixels[self.size - 1][0] = Some(self.source.get_pixel_color(x + 1, y));
        sub_pixels[self.size - 1][self.size - 1] = Some(self.source.get_pixel_color(x + 1, y + 1));

        let mut sub_renderer = |sub_x: usize, sub_y: usize| -> Color {
            if let Some(color) = sub_pixels[sub_x][sub_y] {
                return color;
            }

            *ray_counter += 1;

            let color = self.ray_tracer.get_pixel(
                x as f64 + (sub_x as f64 / self.size as f64),
                y as f64 + (sub_y as f64 / self.size as f64),
                &mut None,
            );
            sub_pixels[sub_x][sub_y] = Some(color);
            color
        };

        let final_color = self.get_sub_pixel_color(
            0, 0, self.size - 1, self.size - 1, self.level, &mut sub_renderer
        );

        final_color
    }

    fn get_sub_pixel_color<R>(
        &self, x1: usize, y1: usize, x2: usize, y2: usize, level: i32, sub_renderer: &mut R
    ) -> Color
        where R: FnMut(usize, usize) -> Color
    {
        let color1 = sub_renderer(x1, y1);
        let color2 = sub_renderer(x2, y1);
        let color3 = sub_renderer(x1, y2);
        let color4 = sub_renderer(x2, y2);

        let different = Self::pixels_are_different(color1, color2, self.threshold) ||
            Self::pixels_are_different(color1, color3, self.threshold) ||
            Self::pixels_are_different(color1, color4, self.threshold);

        if !different || level <= 0 {
            return Self::color_average(color1, color2, color3, color4);
        }

        let mid_x = x1 + (x2 - x1) / 2;
        let mid_y = y1 + (y2 - y1) / 2;
        assert!(x2 - x1 >= 2 && y2 - y1 >= 2);

        let color1 = self.get_sub_pixel_color(x1, y1, mid_x, mid_y, level - 1, sub_renderer);
        let color2 = self.get_sub_pixel_color(mid_x, y1, x2, mid_y, level - 1, sub_renderer);
        let color3 = self.get_sub_pixel_color(x1, mid_y, mid_x, y2, level - 1, sub_renderer);
        let color4 = self.get_sub_pixel_color(mid_x, mid_y, x2, y2, level - 1, sub_renderer);

        Self::color_average(color1, color2, color3, color4)
    }

    fn pixels_are_different(color1: Color, color2: Color, threshold: f64) -> bool {
        // Probably not the best color distance formula...
        (
            abs(color1.r - color2.r) +
            abs(color1.g - color2.g) +
            abs(color1.b - color2.b) +
            abs(color1.a - color2.a)
        ) / 4.0 > threshold
    }

    fn color_average(color1: Color, color2: Color, color3: Color, color4: Color) -> Color {
        let r = (color1.r + color2.r + color3.r + color4.r) / 4.0;
        let g = (color1.g + color2.g + color3.g + color4.g) / 4.0;
        let b = (color1.b + color2.b + color3.b + color4.b) / 4.0;
        let a = (color1.a + color2.a + color3.a + color4.a) / 4.0;

        Color::new(r, g, b, a)
    }

    pub fn mark_edge_pixels(threshold: f64, pixmap: &dyn ColorPixmap, mark: MarkerDelegate) {
        for x in 0..(pixmap.get_width() - 1) {
            for y in 0..(pixmap.get_height() - 1) {
                let color1 = pixmap.get_pixel_color(x, y);

                let pixel_is_different = |color2: Color| {
                    Self::pixels_are_different(color1, color2, threshold)
                };

                if pixel_is_different(pixmap.get_pixel_color(x, y + 1)) ||
                    pixel_is_different(pixmap.get_pixel_color(x + 1, y)) ||
                    pixel_is_different(pixmap.get_pixel_color(x + 1, y + 1)) {
                    mark(x, y);
                }
            }
        }
    }
}
