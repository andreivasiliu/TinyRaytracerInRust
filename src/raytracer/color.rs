#[derive(Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const EMPTY: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Color { r, g, b, a }
    }

    // TODO: Maybe just change to clamp
    pub fn in_limit(x: f64, min: f64, max: f64) -> f64 {
        if x < min {
            min
        } else if x > max {
            max
        } else {
            x
        }
    }

    pub fn in_range(r: f64, g: f64, b: f64) -> Color {
        Color {
            r: Color::in_limit(r, 0.0, 1.0),
            g: Color::in_limit(g, 0.0, 1.0),
            b: Color::in_limit(b, 0.0, 1.0),
            a: 1.0,
        }
    }

    pub fn from_u8(r: u8, g: u8, b: u8, alpha: Option<u8>) -> Color {
        Color {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: alpha.unwrap_or(255) as f64 / 255.0,
        }
    }

    pub fn to_u8(self) -> (u8, u8, u8) {
        let r = (self.r * 255.0) as u8;
        let g = (self.g * 255.0) as u8;
        let b = (self.b * 255.0) as u8;
        (r, g, b)
    }

    pub fn intensify(self, intensity: f64) -> Color {
        Color::in_range(self.r * intensity, self.g * intensity, self.b * intensity)
    }
}

impl std::ops::Mul for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Color {
        Color::in_range(self.r * rhs.r, self.g * rhs.g, self.b * rhs.g)
    }
}

impl std::ops::Add for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Color {
        Color::in_range(self.r + rhs.r, self.g + rhs.g, self.b + rhs.g)
    }
}

pub trait ColorPixmap {
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;

    fn set_pixel_color(&mut self, x: usize, y: usize, color: Color);
    fn get_pixel_color(&self, x: usize, y: usize) -> Color;

    fn fill_with_color(&mut self, color: Color) {
        for x in 0..self.get_width() {
            for y in 0..self.get_height() {
                self.set_pixel_color(x, y, color);
            }
        }
    }
}

#[derive(Clone)]
pub struct RaytracerPixmap {
    width: usize,
    height: usize,
    color_pixmap: Vec<Color>,
}

impl RaytracerPixmap {
    pub fn new(width: usize, height: usize) -> Self {
        RaytracerPixmap {
            width,
            height,
            color_pixmap: vec![Color::EMPTY; width * height],
        }
    }

    pub fn from_color_pixmap<S: ColorPixmap>(source: &S) -> Self {
        let mut color_pixmap = Vec::with_capacity(
            source.get_height() * source.get_width()
        );

        for y in 0..source.get_height() {
            for x in 0..source.get_width() {
                color_pixmap.push(source.get_pixel_color(x, y));
            }
        }

        RaytracerPixmap {
            width: source.get_width(),
            height: source.get_height(),
            color_pixmap,
        }
    }
}

impl ColorPixmap for RaytracerPixmap {
    fn get_width(&self) -> usize {
        self.width
    }

    fn get_height(&self) -> usize {
        self.height
    }

    // TODO: Aka get_color_at
    fn get_pixel_color(&self, x: usize, y: usize) -> Color {
        self.color_pixmap[y * self.width + x]
    }

    // TODO: Aka set_color_at
    fn set_pixel_color(&mut self, x: usize, y: usize, color: Color) {
        self.color_pixmap[y * self.width + x] = color;
    }
}
