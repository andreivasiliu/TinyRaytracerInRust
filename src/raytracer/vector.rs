use super::math::{sqrt, acos};

#[derive(Clone)]
pub struct Ray {
    pub point: Vector,
    pub direction: Vector,
}

#[derive(Clone, Copy)]
pub struct UV {
    pub u: f64,
    pub v: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Vector {
        Vector { x, y, z }
    }

    pub fn axis(&self, axis: usize) -> f64 {
        match axis {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            x => panic!("Wrong axis {} given to Vector!", x),
        }
    }

    pub fn axis_mut(&mut self, axis: usize) -> &mut f64 {
        match axis {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            x => panic!("Wrong axis {} given to Vector!", x),
        }
    }

    pub fn normalized(self) -> Vector {
        self * (1.0 / self.length())
    }

    pub fn length(self) -> f64 {
        sqrt(self * self)
    }

    pub fn distance(a: Vector, b: Vector) -> f64 {
        (a - b).length()
    }

    pub fn angle(a: Vector, b: Vector) -> f64 {
        acos((a * b) / (a.length() * b.length()))
    }

    pub fn cross_product(a: Vector, b: Vector) -> Vector {
        Vector::new(
            a.y * b.z - a.z * b.y,
            a.x * b.z - a.z * b.x,
            a.x * b.y - a.y * b.x,
        )
    }
}

impl std::ops::Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Vector {
        Vector::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
        )
    }
}

impl std::ops::Sub for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Vector {
        Vector::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
        )
    }
}

impl std::ops::Mul for Vector {
    type Output = f64;

    fn mul(self, rhs: Vector) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl std::ops::Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Vector {
        Vector::new(
            self.x * rhs,
            self.y * rhs,
            self.z * rhs,
        )
    }
}

impl std::ops::Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Vector {
        Vector::new(-self.x, -self.y, -self.z)
    }
}
