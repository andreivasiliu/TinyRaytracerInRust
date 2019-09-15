use super::color::Color;
use super::vector::{Vector, Ray};
use super::raytracer::{RayType, RayTracer, RayDebuggerCallback};

pub trait Camera: Send + Sync {
    fn get_pixel_color(&self, x: f64, y: f64, ray_tracer: &RayTracer, ray_debugger_callback: RayDebuggerCallback) -> Color;
    fn create_ray(&self, x: f64, y: f64) -> Ray;
    fn clone_box(&self) -> Box<dyn Camera>;
}

impl Clone for Box<dyn Camera> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Clone)]
pub struct PerspectiveCamera {
    width: usize,
    height: usize,
    center: Vector,
    look_at: Vector,
    up: Vector,
    right: Vector,
    direction: Vector,
    aspect_ratio: f64,
}

impl PerspectiveCamera {
    pub fn new(width: usize, height: usize, center: Vector, look_at: Option<Vector>, up: Option<Vector>, right: Option<Vector>) -> Self {
        let look_at = look_at.unwrap_or(Vector::new(0.0, 0.0, 0.0));
        let up = up.unwrap_or(Vector::new(0.0, 1.0, 0.0));
        let right = right.unwrap_or(Vector::new(0.0, 0.0, 0.0));
        let direction = (look_at - center).normalized();
        let aspect_ratio = width as f64 / height as f64;

        let right = if right.length() == 0.0 {
            // FIXME: Remove the negation after switching to a proper coordinate system
            -Vector::cross_product(direction, up)
        } else {
            right
        };

        PerspectiveCamera {
            width,
            height,
            center,
            look_at,
            up,
            right,
            direction,
            aspect_ratio,
        }
    }
}

impl Camera for PerspectiveCamera {
    fn get_pixel_color(&self, x: f64, y: f64, ray_tracer: &RayTracer, ray_debugger_callback: RayDebuggerCallback) -> Color {
        let ray = self.create_ray(x, y);
        ray_tracer.get_ray_color(
            ray, 0, Some(RayType::NormalRay), ray_debugger_callback
        )
    }

    fn create_ray(&self, x: f64, y: f64) -> Ray {
        // Get coordinates in the range -0.5 .. 0.5
        let sx = ((x / self.width as f64) - 0.5) * self.aspect_ratio;
        let sy = (self.height as f64 - 1.0 - y) / self.height as f64 - 0.5;

        Ray {
            direction: self.direction + self.right * sx + self.up * sy,
            point: self.center,
        }
    }

    fn clone_box(&self) -> Box<dyn Camera> {
        Box::new(self.clone())
    }
}

// TODO: A way to specify cross/parallel viewing
#[derive(Clone)]
pub struct StereoscopicCamera {
    left_camera: PerspectiveCamera,
    right_camera: PerspectiveCamera,
    width: usize,
}

impl StereoscopicCamera {
    pub fn new(
        width: usize, height: usize, center: Vector, eye_distance: f64,
        look_at: Option<Vector>, up: Option<Vector>, right: Option<Vector>,
    ) -> Self {
        let look_at = look_at.unwrap_or(Vector::new(0.0, 0.0, 0.0));
        let up = up.unwrap_or(Vector::new(0.0, 1.0, 0.0));
        let right = right.unwrap_or(Vector::new(0.0, 0.0, 0.0));
        let width = width / 2;

        let right = if right.length() == 0.0 {
            let direction = (look_at - center).normalized();

            // FIXME: Remove the negation after switching to a proper coordinate system
            -Vector::cross_product(direction, up)
        } else {
            right
        };

        let left_eye = center - right * (eye_distance / 2.0);
        let right_eye = center + right * (eye_distance / 2.0);

        let left_camera = PerspectiveCamera::new(width, height, left_eye, Some(look_at), Some(up), None);
        let right_camera = PerspectiveCamera::new(width, height, right_eye, Some(look_at), Some(up), None);

        StereoscopicCamera {
            left_camera,
            right_camera,
            width: width,
        }
    }
}

impl Camera for StereoscopicCamera {
    fn get_pixel_color(&self, x: f64, y: f64, ray_tracer: &RayTracer, ray_debugger_callback: RayDebuggerCallback) -> Color {
        let (x, camera) = if x >= self.width as f64 {
            (x - self.width as f64, &self.left_camera)
        } else {
            (x, &self.right_camera)
        };

        let ray = camera.create_ray(x, y);
        ray_tracer.get_ray_color(ray, 0, Some(RayType::NormalRay), ray_debugger_callback)
    }

    fn create_ray(&self, x: f64, y: f64) -> Ray {
        self.left_camera.create_ray(x, y)
    }

    fn clone_box(&self) -> Box<dyn Camera> {
        Box::new(self.clone())
    }
}

// TODO: Color masks for each eye
#[derive(Clone)]
pub struct AnaglyphCamera {
    left_camera: PerspectiveCamera,
    right_camera: PerspectiveCamera,
    width: usize,
}

impl AnaglyphCamera {
    fn new(
        width: usize, height: usize, center: Vector, eye_distance: f64,
        look_at: Option<Vector>, up: Option<Vector>, right: Option<Vector>,
    ) -> Self {
        let look_at = look_at.unwrap_or(Vector::new(0.0, 0.0, 0.0));
        let up = up.unwrap_or(Vector::new(0.0, 1.0, 0.0));
        let right = right.unwrap_or(Vector::new(0.0, 0.0, 0.0));

        let right = if right.length() == 0.0 {
            let direction = (look_at - center).normalized();

            // FIXME: Remove the negation after switching to a proper coordinate system
            -Vector::cross_product(direction, up)
        } else {
            right
        };

        let left_eye = center - right * (eye_distance / 2.0);
        let right_eye = center + right * (eye_distance / 2.0);

        let left_camera = PerspectiveCamera::new(width, height, left_eye, Some(look_at), Some(up), None);
        let right_camera = PerspectiveCamera::new(width, height, right_eye, Some(look_at), Some(up), None);

        AnaglyphCamera {
            left_camera,
            right_camera,
            width,
        }
    }
}

impl Camera for AnaglyphCamera {
    fn get_pixel_color(&self, x: f64, y: f64, ray_tracer: &RayTracer, ray_debugger_callback: RayDebuggerCallback) -> Color {
        // FIXME: Rust's re-borrowing is not smart enough for options of mutable references
        let color1 = ray_tracer.get_ray_color(
            self.left_camera.create_ray(x, y), 0,
            Some(RayType::NormalRay), ray_debugger_callback
        );
        let color2 = ray_tracer.get_ray_color(
            self.right_camera.create_ray(x, y), 0,
            Some(RayType::NormalRay), ray_debugger_callback
        );

        return Color::new(color1.r, color2.g, color2.b, 1.0);
    }

    fn create_ray(&self, x: f64, y: f64) -> Ray {
        self.left_camera.create_ray(x, y)
    }

    fn clone_box(&self) -> Box<dyn Camera> {
        Box::new(self.clone())
    }
}

// TODO:
// - PanoramicCamera
// - OrthogonalCamera