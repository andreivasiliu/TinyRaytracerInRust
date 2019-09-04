use super::math::{cos, sin};
use super::vector::{Vector, Ray};

// Just a normal stack, except that it knows how to multiply transformation
// matrices.

pub struct TransformationStack {
    stack: Vec<MatrixTransformation>,
}

impl TransformationStack {
    pub fn new_with_identity() -> Self {
        let stack = vec![MatrixTransformation::create_identity_matrix()];

        TransformationStack {
            stack,
        }
    }

    pub fn push_transformation(&mut self, transformation: MatrixTransformation) {
        let new_transformation = if let Some(last_transformation) = self.stack.last() {
            transformation.compose_with(last_transformation)
        } else {
            transformation
        };
        self.stack.push(new_transformation);
    }

    pub fn pop_transformation(&mut self) -> MatrixTransformation {
        self.stack.pop().expect("Trying to pop from an empty TransformationStack!")
    }

    pub fn get_transformation(&self) -> Option<&MatrixTransformation> {
        self.stack.last()
    }
}

pub trait Transformation {
    fn transform_vector(&self, vector: Vector) -> Vector;
    fn reverse_transform_vector(&self, vector: Vector) -> Vector;
    fn transform_direction_vector(&self, vector: Vector) -> Vector;
    fn reverse_transform_direction_vector(&self, vector: Vector) -> Vector;
    fn reverse_transform_ray(&self, ray: Ray) -> Ray;
}

#[derive(Clone)]
pub struct MatrixTransformation {
    matrix: [[f64; 4]; 4],
    inverse_matrix: [[f64; 4]; 4],
}

fn transform_vector(vector: Vector, matrix: [[f64; 4]; 4]) -> Vector {
    let a = matrix[0][0] * vector.x + matrix[0][1] * vector.y + matrix[0][2] * vector.z + matrix[0][3];
    let b = matrix[1][0] * vector.x + matrix[1][1] * vector.y + matrix[1][2] * vector.z + matrix[1][3];
    let c = matrix[2][0] * vector.x + matrix[2][1] * vector.y + matrix[2][2] * vector.z + matrix[2][3];

    Vector::new(a, b, c)
}

impl Transformation for MatrixTransformation {
    fn transform_vector(&self, vector: Vector) -> Vector {
        transform_vector(vector, self.matrix)
    }

    fn reverse_transform_vector(&self, vector: Vector) -> Vector {
        transform_vector(vector, self.inverse_matrix)
    }

    fn transform_direction_vector(&self, vector: Vector) -> Vector {
        let transformed_origin = transform_vector(
            Vector::new(0.0, 0.0, 0.0),
            self.matrix,
        );

        transform_vector(vector, self.matrix) - transformed_origin
    }

    fn reverse_transform_direction_vector(&self, vector: Vector) -> Vector {
        let transformed_origin = transform_vector(
            Vector::new(0.0, 0.0, 0.0),
            self.inverse_matrix,
        );

        transform_vector(vector, self.inverse_matrix) - transformed_origin
    }

    fn reverse_transform_ray(&self, ray: Ray) -> Ray {
        Ray {
            point: self.reverse_transform_vector(ray.point),
            direction: self.reverse_transform_direction_vector(ray.direction),
        }
    }
}

impl MatrixTransformation {
    pub fn new(matrix: [[f64; 4]; 4], inverse_matrix: [[f64; 4]; 4]) -> Self {
        MatrixTransformation {
            matrix,
            inverse_matrix,
        }
    }

    pub fn create_identity_matrix() -> Self {
        let matrix = [
            [ 1.0, 0.0, 0.0, 0.0 ],
            [ 0.0, 1.0, 0.0, 0.0 ],
            [ 0.0, 0.0, 1.0, 0.0 ],
            [ 0.0, 0.0, 0.0, 1.0 ],
        ];

        MatrixTransformation::new(matrix, matrix)
    }

    pub fn create_rotation_matrix(x: f64, y: f64, z: f64) -> Self {
        fn x_rotation_matrix(angle: f64) -> [[f64; 4]; 4] {
            let csa = cos(angle);
            let sna = sin(angle);
            [
                [ 1.0, 0.0, 0.0, 0.0 ],
                [ 0.0, csa,-sna, 0.0 ],
                [ 0.0, sna, csa, 0.0 ],
                [ 0.0, 0.0, 0.0, 1.0 ],
            ]
        }

        fn y_rotation_matrix(angle: f64) -> [[f64; 4]; 4] {
            let csa = cos(angle);
            let sna = sin(angle);
            [
                [ csa, 0.0,-sna, 0.0 ],
                [ 0.0, 1.0, 0.0, 0.0 ],
                [ sna, 0.0, csa, 0.0 ],
                [ 0.0, 0.0, 0.0, 1.0 ],
            ]
        }

        fn z_rotation_matrix(angle: f64) -> [[f64; 4]; 4] {
            let csa = cos(angle);
            let sna = sin(angle);
            [
                [ csa,-sna, 0.0, 0.0 ],
                [ sna, csa, 0.0, 0.0 ],
                [ 0.0, 0.0, 1.0, 0.0 ],
                [ 0.0, 0.0, 0.0, 1.0 ],
            ]
        }

        let matrix_1 = x_rotation_matrix(x);
        let inverse_matrix_1 = x_rotation_matrix(-x);
        let matrix_2 = y_rotation_matrix(y);
        let inverse_matrix_2 = y_rotation_matrix(-y);
        let matrix_3 = z_rotation_matrix(z);
        let inverse_matrix_3 = z_rotation_matrix(-z);

        let temp = multiply_matrices(matrix_1, matrix_2);
        let final_matrix = multiply_matrices(temp, matrix_3);
        let temp = multiply_matrices(inverse_matrix_1, inverse_matrix_2);
        let final_inverse_matrix = multiply_matrices(temp, inverse_matrix_3);

        MatrixTransformation::new(final_matrix, final_inverse_matrix)
    }

    pub fn create_translation_matrix(x: f64, y: f64, z: f64) -> Self {
        let matrix = [
            [ 1.0, 0.0, 0.0, x ],
            [ 0.0, 1.0, 0.0, y ],
            [ 0.0, 0.0, 1.0, z ],
            [ 0.0, 0.0, 0.0, 1.0 ],
        ];

        let inverse_matrix = [
            [ 1.0, 0.0, 0.0, -x ],
            [ 0.0, 1.0, 0.0, -y ],
            [ 0.0, 0.0, 1.0, -z ],
            [ 0.0, 0.0, 0.0, 1.0 ],
        ];

        MatrixTransformation::new(matrix, inverse_matrix)
    }

    pub fn create_scaling_matrix(x: f64, y: f64, z: f64) -> Self {
        let matrix = [
            [ x, 0.0, 0.0, 0.0 ],
            [ 0.0, y, 0.0, 0.0 ],
            [ 0.0, 0.0, z, 0.0 ],
            [ 0.0, 0.0, 0.0, 1.0 ],
        ];

        let inverse_matrix = [
            [ 1.0 / x, 0.0, 0.0, 0.0 ],
            [ 0.0, 1.0 / y, 0.0, 0.0 ],
            [ 0.0, 0.0, 1.0 / z, 0.0 ],
            [ 0.0, 0.0, 0.0, 1.0 ],
        ];

        MatrixTransformation::new(matrix, inverse_matrix)
    }

    pub fn compose_with(&self, other: &MatrixTransformation) -> MatrixTransformation {
        let new_matrix = multiply_matrices(self.matrix, other.matrix);
        let new_inverse_matrix = multiply_matrices(self.inverse_matrix, other.inverse_matrix);

        MatrixTransformation::new(new_matrix, new_inverse_matrix)
    }
}

fn multiply_matrices(matrix1: [[f64; 4]; 4], matrix2: [[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let mut result: [[f64; 4]; 4] = Default::default();

    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                result[i][j] += matrix1[i][k] * matrix2[k][j];
            }
        }
    }

    result
}
