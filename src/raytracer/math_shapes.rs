use super::math::{PI, EPSILON, INFINITY, NEG_INFINITY, sin, sqrt, abs, acos};
use super::transformation::{MatrixTransformation, Transformation};
use super::vector::{Vector, UV, Ray};

pub type AddIntersection<'a> = &'a mut dyn FnMut(f64);

pub trait MathShape {
    fn intersects(&self, ray: Ray, add_intersection: AddIntersection);
    fn get_normal(&self, surface_point: Vector) -> Vector;
    fn is_inside(&self, point: Vector) -> bool;
    fn is_on_surface(&self, point: Vector) -> bool;
    fn get_uv_coordinates(&self, point: Vector) -> Result<UV, &'static str>;
    fn set_transformation(&mut self, transformation: MatrixTransformation);
    fn get_transformation(&self) -> &MatrixTransformation;

    fn reverse_transform_ray(&self, ray: Ray) -> Ray {
        self.get_transformation().reverse_transform_ray(ray)
    }
}

pub struct MathSphere {
    transformation: MatrixTransformation,
    center: Vector,
    radius: f64,
}

impl MathSphere {
    pub fn new(transformation: MatrixTransformation, center: Vector, radius: f64) -> Self {
        MathSphere { transformation, center, radius }
    }
}

impl MathShape for MathSphere {
    fn intersects(&self, ray: Ray, add_intersection: AddIntersection) {
        let v = ray.point - self.center;
        let d = ray.direction.normalized();

        let scale = 1.0 / ray.direction.length();
        let r = self.radius;

        let vd = v * d;
        let sum = vd * vd - (v * v - r * r);
        if sum < 0.0 {
            return;
        }

        let first = (-vd + sqrt(sum)) * scale;
        let second = (-vd - sqrt(sum)) * scale;

        // Some might be behind the camera, but objects behind the camera might
        // be of interest as well (on an orthogonal view, for example).
        add_intersection(first);
        add_intersection(second);
    }

    fn get_normal(&self, surface_point: Vector) -> Vector {
        let surface_point = self.transformation.reverse_transform_vector(surface_point);
        let normal = surface_point - self.center;
        self.transformation.transform_direction_vector(normal).normalized()
    }

    fn is_inside(&self, point: Vector) -> bool {
        let point = self.transformation.reverse_transform_vector(point);

        (point - self.center).length() <= self.radius + EPSILON
    }

    fn is_on_surface(&self, point: Vector) -> bool {
        let point = self.transformation.reverse_transform_vector(point);

        abs((point - self.center).length() - self.radius) < EPSILON
    }

    fn get_uv_coordinates(&self, point: Vector) -> Result<UV, &'static str> {
        let point = self.transformation.reverse_transform_vector(point - self.center);
        let point = point.normalized() * (1.0 - EPSILON);

        let up = Vector::new(0.0, 1.0, 0.0);
        let u_zero = Vector::new(0.0, 0.0, -1.0);
        let u_qrtr = Vector::new(-1.0, 0.0, 0.0);

        let phi = acos(-(up * point));
        let phi = if phi.is_nan() {
            eprintln!("MathSphere::get_uv_coordinates: phi was NaN!");
            0.0
        } else {
            phi
        };

        let theta = (acos((point * u_zero) / sin(phi))) / (2.0 * PI);
        let theta = if theta.is_nan() {
            eprintln!("MathSphere::get_uv_coordinates: theta was NaN!");
            0.0
        } else {
            theta
        };

        let v = phi / PI;
        let u = if u_qrtr * point > 0.0 {
            1.0 - theta
        } else {
            theta
        };

        Ok(UV { u, v })
    }

    fn set_transformation(&mut self, transformation: MatrixTransformation) {
        self.transformation = transformation;
    }

    fn get_transformation(&self) -> &MatrixTransformation {
        &self.transformation
    }
}

pub struct MathPlane {
    transformation: MatrixTransformation,
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    normal: Vector,
}

impl MathPlane {
    pub fn new(transformation: MatrixTransformation, a: f64, b: f64, c: f64, d: f64) -> Self {
        let normal = Vector::new(a, b, c).normalized();
        let normal = MathPlane::transformed_normal(normal, &transformation);

        MathPlane {
            transformation,
            a,
            b,
            c,
            d,
            normal,
        }
    }

    pub fn from_normal(transformation: MatrixTransformation, normal: Vector, distance: f64) -> Self {
        MathPlane::new(transformation, normal.x, normal.y, normal.z, distance)
    }

    fn transformed_normal(normal: Vector, transformation: &MatrixTransformation) -> Vector {
        transformation.transform_direction_vector(normal).normalized()
    }

    fn is_transformed_point_on_surface(&self, point: Vector) -> bool {
        abs(self.a * point.x + self.b * point.y + self.c * point.z + self.d) < EPSILON
    }
}

impl MathShape for MathPlane {
    fn intersects(&self, ray: Ray, add_intersection: AddIntersection) {
        let p_n = Vector::new(self.a, self.b, self.c).normalized();
        let r_0 = ray.point;
        let r_d = ray.direction;
        let v_d = p_n * r_d;

        if v_d != 0.0 {
            let t = -(p_n * r_0 + self.d) * (1.0 / v_d);
            if t >= 0.0 {
                add_intersection(t);
            }
        }
    }

    fn get_normal(&self, _surface_point: Vector) -> Vector {
        self.normal
    }

    fn is_inside(&self, _point: Vector) -> bool {
        false
    }

    fn is_on_surface(&self, point: Vector) -> bool {
        self.is_transformed_point_on_surface(
            self.transformation.reverse_transform_vector(point)
        )
    }

    fn get_uv_coordinates(&self, _point: Vector) -> Result<UV, &'static str> {
        Err("UV not implemented for MathPlane!")
    }

    fn set_transformation(&mut self, transformation: MatrixTransformation) {
        self.normal = MathPlane::transformed_normal(self.normal, &transformation);
        self.transformation = transformation;
    }

    fn get_transformation(&self) -> &MatrixTransformation {
        &self.transformation
    }
}

pub struct MathCube {
    transformation: MatrixTransformation,
    p1: MathPlane,
    p2: MathPlane,
    p3: MathPlane,
    p4: MathPlane,
    p5: MathPlane,
    p6: MathPlane,
    center: Vector,
    length: f64,
}

impl MathCube {
    pub fn new(transformation: MatrixTransformation, center: Vector, length: f64) -> Self {
        let length = length / 2.0;
        let t = transformation;

        MathCube {
            p1: MathPlane::new(t.clone(), 0.0, 0.0, 1.0, -(center.z + length)),
            p6: MathPlane::new(t.clone(), 0.0, 0.0, -1.0, center.z + -length / 2.0),
            p2: MathPlane::new(t.clone(), 0.0, 1.0, 0.0, -(center.y + length / 2.0)),
            p5: MathPlane::new(t.clone(), 0.0, -1.0, 0.0, center.y + -length / 2.0),
            p3: MathPlane::new(t.clone(), 1.0, 0.0, 0.0, -(center.x + length / 2.0)),
            p4: MathPlane::new(t.clone(), -1.0, 0.0, 0.0, center.x + -length / 2.0),

            transformation: t,
            center,
            length,
        }
    }
}

impl MathShape for MathCube {
    fn intersects(&self, ray: Ray, add_intersection: AddIntersection) {
        let mut t_near = NEG_INFINITY;
        let mut t_far = INFINITY;

        let ray_direction_v = [ray.direction.x, ray.direction.y, ray.direction.z];
        let ray_point_v = [ray.point.x, ray.point.y, ray.point.z];
        let center_v = [self.center.x, self.center.y, self.center.z];

        // X planes
        for i in 0..3 {
            if ray_direction_v[i] == 0.0 {
                if ray_point_v[i] < center_v[i] - self.length ||
                    ray_point_v[i] > center_v[i] + self.length {
                    return;
                }
                // ?
                continue;
            }

            let t1 = (center_v[i] - self.length - ray_point_v[i]) / ray_direction_v[i];
            let t2 = (center_v[i] + self.length - ray_point_v[i]) / ray_direction_v[i];

            let (t1, t2) = if t1 > t2 {
                (t2, t1)
            } else {
                (t1, t2)
            };

            if t1 > t_near {
                t_near = t1;
            }
            if t2 < t_far {
                t_far = t2;
            }

            if t_near > t_far || t_far < 0.0 {
                return;
            }
        }

        add_intersection(t_near);
        add_intersection(t_far);
    }

    fn get_normal(&self, surface_point: Vector) -> Vector {
        // TODO: This could be greatly improved, since we should already know
        // which surface was intersected.

        let surface_point = self.transformation.reverse_transform_vector(surface_point);

        let planes = [
            &self.p1,
            &self.p2,
            &self.p3,
            &self.p4,
            &self.p5,
            &self.p6,
        ];

        for plane in planes.iter() {
            if plane.is_transformed_point_on_surface(surface_point) {
                return plane.get_normal(surface_point);
            }
        }

        panic!("Get normal for MathCube failed!")
    }

    fn is_inside(&self, point: Vector) -> bool {
        let point = self.transformation.reverse_transform_vector(point);

        point.x <= (self.center.x + self.length) &&
            point.x >= (self.center.x - self.length) &&
            point.y <= (self.center.y + self.length) &&
            point.y >= (self.center.y - self.length) &&
            point.z <= (self.center.z + self.length) &&
            point.z >= (self.center.z - self.length)
    }

    fn is_on_surface(&self, point: Vector) -> bool {
        let point = self.transformation.reverse_transform_vector(point);

        fn is_between(x: f64, start: f64, end: f64) -> bool {
            (start..=end).contains(&x)
        }

        let center = self.center;
        let length = self.length;

        if is_between(point.y, center.y - length - EPSILON, center.y + length + EPSILON) &&
            is_between(point.x, center.x - length - EPSILON, center.x + length + EPSILON) &&
            (self.p1.is_transformed_point_on_surface(point) || self.p6.is_transformed_point_on_surface(point)) {
            return true;
        } else if is_between(point.z, center.z - length - EPSILON, center.z + length + EPSILON) &&
            is_between(point.x, center.x - length - EPSILON, center.x + length + EPSILON) &&
            (self.p2.is_transformed_point_on_surface(point) || self.p5.is_transformed_point_on_surface(point)) {
            return true;
        } else if is_between(point.y, center.y - length - EPSILON, center.y + length + EPSILON) &&
            is_between(point.z, center.z - length - EPSILON, center.z + length + EPSILON) &&
            (self.p3.is_transformed_point_on_surface(point) || self.p4.is_transformed_point_on_surface(point)) {
            return true;
        } else {
            return false;
        }
    }

    fn get_uv_coordinates(&self, _point: Vector) -> Result<UV, &'static str> {
        Err("UV not implemented for MathCube!")
    }

    fn set_transformation(&mut self, transformation: MatrixTransformation) {
        self.p1.set_transformation(transformation.clone());
        self.p2.set_transformation(transformation.clone());
        self.p3.set_transformation(transformation.clone());
        self.p4.set_transformation(transformation.clone());
        self.p5.set_transformation(transformation.clone());
        self.p6.set_transformation(transformation.clone());
    }

    fn get_transformation(&self) -> &MatrixTransformation {
        &self.transformation
    }
}