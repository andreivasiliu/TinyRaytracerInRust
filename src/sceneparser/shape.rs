#[derive(Debug, Clone)]
pub struct Shape {
    pub color: Option<(f64, f64, f64, f64)>,
    pub reflectivity: f64,
    pub transparency: f64,
    pub kind: ShapeKind,
}

#[derive(Debug, Clone)]
pub enum ShapeKind {
    Sphere { radius: f64 },
    Cube { length: f64 },
    Plane { normal: (f64, f64, f64), distance: f64 },
    // CSG { operator,  }
}
