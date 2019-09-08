use super::raytracer::RayTracer;
use super::color::{Color, ColorPixmap};
use super::math::abs;

type SubPixels = Vec<Vec<Option<Color>>>;

pub struct AntiAliaser<'a> {
    ray_tracer: &'a RayTracer,
    threshold: f64,
    level: i32,
    size: usize,
}

impl<'a> AntiAliaser<'a> {
    pub fn new(
        ray_tracer: &'a RayTracer, threshold: Option<f64>, level: Option<i32>,
    ) -> Self {
        let threshold = threshold.unwrap_or(0.1);
        let level = level.unwrap_or(3);
        let size= (1 << level) as usize + 1;

        AntiAliaser {
            ray_tracer,
            threshold,
            level,
            size,
        }
    }

    pub fn set_threshold(&mut self, threshold: f64) {
        self.threshold = threshold;
    }

    // TODO: Find a proper name for these methods.
    pub fn anti_alias_line<S: ColorPixmap, D: ColorPixmap>(
        &mut self, y: usize, sub_pixels: &mut SubPixels, ray_counter: &mut i32,
        source: &S, destination: &mut D,
    ) {
        let last_pixel = source.get_width() - 1;
        for x in 0..last_pixel {
            let color = self.get_anti_aliased_pixel(
                x, y, sub_pixels, ray_counter, source
            );
            destination.set_pixel_color(x, y, color);
        }

        // Copy the last pixel.
        destination.set_pixel_color(
            last_pixel, y, source.get_pixel_color(last_pixel, y),
        );
    }

    pub fn anti_alias_line_vec<S: ColorPixmap>(
        &self, y: usize, sub_pixels: &mut SubPixels, ray_counter: &mut i32,
        source: &S,
    ) -> Vec<Color> {
        let mut line = Vec::with_capacity(source.get_width());

        let last_pixel = source.get_width() - 1;
        for x in 0..last_pixel {
            let color = self.get_anti_aliased_pixel(
                x, y, sub_pixels, ray_counter, source
            );
            line.push(color);
        }

        // Copy the last pixel.
        line.push(source.get_pixel_color(last_pixel, y));

        line
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

    pub fn create_sub_pixel_buffer(&self) -> SubPixels {
        vec![vec![None; self.size]; self.size]
    }

    pub fn get_anti_aliased_pixel<S: ColorPixmap>(
        &self, x: usize, y: usize, sub_pixels: &mut SubPixels, ray_counter: &mut i32,
        source: &S,
    ) -> Color {
        self.clear_matrices(sub_pixels);

        // Set up this pixel's subpixel matrix.
        // Note that which dimension is Y and which is X does not actually
        // matter; the result is the same.
        sub_pixels[0][0] = Some(source.get_pixel_color(x, y));
        sub_pixels[0][self.size - 1] = Some(source.get_pixel_color(x, y + 1));
        sub_pixels[self.size - 1][0] = Some(source.get_pixel_color(x + 1, y));
        sub_pixels[self.size - 1][self.size - 1] = Some(source.get_pixel_color(x + 1, y + 1));

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

    pub fn mark_edge_pixels<F: FnMut(usize, usize)>(
        threshold: f64, pixmap: &dyn ColorPixmap, mut mark_pixel: F
    ) {
        for x in 0..(pixmap.get_width() - 1) {
            for y in 0..(pixmap.get_height() - 1) {
                let color1 = pixmap.get_pixel_color(x, y);

                let pixel_is_different = |color2: Color| {
                    Self::pixels_are_different(color1, color2, threshold)
                };

                if pixel_is_different(pixmap.get_pixel_color(x, y + 1)) ||
                    pixel_is_different(pixmap.get_pixel_color(x + 1, y)) ||
                    pixel_is_different(pixmap.get_pixel_color(x + 1, y + 1)) {
                    mark_pixel(x, y);
                }
            }
        }
    }
}
