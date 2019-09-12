use super::color::Color;
use super::vector::{Vector, UV, Ray};
use super::rt_object::RTObject;
use super::camera::{Camera, PerspectiveCamera};
use super::transformation::{TransformationStack, MatrixTransformation};
use super::point_light::PointLight;
use super::math::{PI, INFINITY, EPSILON, sqrt};

#[derive(Clone, Copy)]
pub enum RayType {
    NormalRay,
    ReflectionRay,
    TransmissionRay,
}

// Used by debuggers to show info about each ray
pub type RayDebuggerCallback<'a, 'b> = &'a mut Option<&'b mut dyn FnMut(
    i32, Ray, f64, Option<&RTObject>, &Color, &RayType
)>;

#[derive(Clone)]
pub struct RayTracer {
    transformation_stack: TransformationStack,
    camera: Box<dyn Camera>,
    top: f64,
    bottom: f64,
    left: f64,
    right: f64,
    width: usize,
    height: usize,
    max_depth: i32,

    objects: Vec<RTObject>,
    point_lights: Vec<PointLight>,
}

impl RayTracer {
    pub fn new_default(width: usize, height: usize) -> Self {
        RayTracer::new(
            Vector::new(0.0, 0.0, -100.0),
            60.0, -60.0, -80.0, 80.0,
            width, height,
        )
    }

    pub fn new(
        camera: Vector, top: f64, bottom: f64, left: f64, right: f64,
        width: usize, height: usize,
    ) -> Self {
        let transformation_stack = TransformationStack::new_with_identity();
        let camera = PerspectiveCamera::new(
            width, height, camera,
            None, None, None
        );

        RayTracer {
            transformation_stack,
            camera: Box::new(camera),
            top,
            bottom,
            left,
            right,
            width,
            height,
            max_depth: 10,

            objects: vec![],
            point_lights: vec![],
        }
    }

    pub fn add_test_objects(&mut self) {
        //use super::csg::{CSG, Operator};
        use super::math_shapes::MathPlane;
        //use super::math_shapes::MathSphere;
        //use super::math_shapes::MathCube;
        use super::material::SolidColorMaterial;
        //use super::transformation::MatrixTransformation;

        /*
        self.transformation_stack
            .push_transformation(MatrixTransformation::create_rotation_matrix(-0.3, 0.0, 0.0));

        self.transformation_stack
            .push_transformation(MatrixTransformation::create_rotation_matrix(0.0, 2.0, 0.0));
        */

        let t = self.transformation_stack
            .get_transformation()
            .expect("Expected transformation in stack");

        /*self.objects.push(RTObject::new(
            Box::new(CSG::new(
                t.clone(),
                RTObject::new_default(Box::new(MathCube::new(t.clone(), Vector::new(0.0, 0.0, 0.0), 40.0))),
                RTObject::new_default(Box::new(MathSphere::new(t.clone(), Vector::new(0.0, 0.0, 0.0), 24.0))),
                Operator::Intersection,
            )),
            Some(Box::new(SolidColorMaterial::new(Color::new(1.0, 0.0, 0.0, 1.0), 0.1, 0.7)))
        ));
        self.objects.push(RTObject::new(
            Box::new(MathCube::new(t.clone(), Vector::new(0.0, 0.0, 0.0), 40.0)),
            Some(Box::new(SolidColorMaterial::new(Color::new(1.0, 0.0, 0.0, 1.0), 0.5, 0.0))),
        ));*/
        /*self.objects.push(RTObject::new(
            Box::new(MathSphere::new(t.clone(), Vector::new(20.0, -5.0, 10.0), 30.0)),
            Some(Box::new(SolidColorMaterial::new(Color::new(1.0, 0.0, 0.0, 1.0), 0.5, 0.0))),
        ));
        self.objects.push(RTObject::new(
            Box::new(MathSphere::new(t.clone(), Vector::new(-15.0, -5.0, -10.0), 30.0)),
            Some(Box::new(SolidColorMaterial::new(Color::new(0.0, 1.0, 1.0, 1.0), 0.0, 0.8))),
        ));*/

        // FIXME: ...

        self.objects.push(RTObject::new(
            Box::new(MathPlane::new(t.clone(), 0.0, 1.0, 0.0, 20.0)),
            Some(Box::new(SolidColorMaterial::new(Color::new(0.5, 0.0, 0.5, 1.0), 0.2, 0.0)))
        ));

        self.point_lights.push(PointLight::new(
            Vector::new(-10.0, 30.0, -50.0),
            Color::in_range(0.5, 0.5, 0.5),
            100.0
        ));
    }

    pub fn get_ray_color(
        &self, ray: Ray, depth: i32, ray_type: Option<RayType>,
        ray_debugger_callback: RayDebuggerCallback
    ) -> Color {
        let ray_type = ray_type.unwrap_or(RayType::NormalRay);

        let mut nearest_distance = INFINITY;
        let mut nearest_object = None;

        for obj in self.objects.iter() {
            let mut add_intersection = |d: f64| {
                if d > EPSILON && d < nearest_distance {
                    nearest_distance = d;
                    nearest_object = Some(obj);
                }
            };

            obj.intersects(ray.clone(), &mut add_intersection);
        }

        let rt_object = match nearest_object {
            Some(rt_object) => rt_object,
            None => {
                if let Some(debugger) = ray_debugger_callback {
                    debugger(depth, ray, INFINITY, None, &Color::BLACK, &ray_type);
                }
                return Color::BLACK;
            }
        };

        let point = ray.point + ray.direction * nearest_distance;
        let normal = rt_object.get_shape().get_normal(point).normalized();

        let uv_coord = rt_object
            .get_shape()
            .get_uv_coordinates(point)
            .unwrap_or(UV { u: 0.0, v: 0.0 });

        let c = rt_object.get_material().get_color_at_uv(uv_coord);

        let ambient = c * Color::in_range(1.0, 1.0, 1.0).intensify(0.6);
        let mut final_light = ambient;

        for light in self.point_lights.iter() {
            let shadow_ray = Ray {
                point,
                direction: (*light.get_point() - point).normalized()
            };
            let distance_to_light = (*light.get_point() - point).length();
            let mut transparency = 1.0;
            use std::cell::RefCell;
            let mut cached_obj: Option<&RTObject> = None;
            // Share it betwen both the for loop and the closure.
            let cached_obj = RefCell::new(&mut cached_obj);

            let mut add_shadow_intersection = |d: f64| {
                if d > EPSILON && d < distance_to_light {
                    let cached_obj = cached_obj.borrow_mut().unwrap();
                    transparency *= cached_obj.get_material().get_transparency_at_uv(uv_coord);
                }
            };

            for obj in self.objects.iter() {
                cached_obj.borrow_mut().replace(obj);
                obj.intersects(shadow_ray.clone(), &mut add_shadow_intersection);
            }

            // Ignore this light, because there is an opaque object in the way.
            if transparency == 0.0 {
                continue;
            }

            let angle = Vector::angle(shadow_ray.direction, normal);

            if angle < 0.0 {
                panic!("Holy crap, negative angle!");
            }

            let angle = if angle >= PI / 2.0 {
                PI - angle
            } else {
                angle
            };

            let intensity = if angle < (PI / 2.0) && angle >= 0.0 {
                1.0 - (angle / (PI / 2.0))
            } else {
                0.0
            };

            let light_color = light
                .get_color()
                .intensify(intensity)
                .intensify(transparency);

            final_light = final_light + c * light_color;
        }

        let angle = Vector::angle( ray.direction *-1.0, normal);
        let (r1, r2, normal, inside_out) = if angle >= PI / 2.0 {
            (1.45, 1.0, normal * -1.0, true)
        } else {
            (1.0, 1.45, normal, false)
        };

        let transparency = rt_object.get_material().get_transparency_at_uv(uv_coord);
        let reflectivity = rt_object.get_material().get_reflectivity_at_uv(uv_coord);

        let mut total_internal_reflection = false;

        if depth < self.max_depth && transparency != 0.0 {
            let refracted_ray = Ray {
                point: ray.point + ray.direction * nearest_distance,
                direction: Self::get_refracted_ray_direction(
                    ray.direction, normal, r1 / r2, &mut total_internal_reflection
                ),
            };

            if !total_internal_reflection {
                let refracted_ray_color = self.get_ray_color(
                    refracted_ray, depth + 1, Some(RayType::TransmissionRay),
                    ray_debugger_callback
                );

                final_light = final_light.intensify(1.0 - transparency) +
                    refracted_ray_color.intensify(transparency);
            }
        }

        let reflectivity = if total_internal_reflection {
            reflectivity + (1.0 - reflectivity) * transparency
        } else {
            reflectivity
        };

        if depth < self.max_depth && reflectivity != 0.0 && (!inside_out || total_internal_reflection) {
            let reflected_ray = Ray {
                point: ray.point + ray.direction * nearest_distance,
                direction: Self::get_reflected_ray_direction(ray.direction, normal),
            };

            let reflected_ray_color = self.get_ray_color(
                reflected_ray, depth + 1, Some(RayType::ReflectionRay),
                ray_debugger_callback
            );

            final_light = final_light.intensify(1.0 - reflectivity) +
                reflected_ray_color.intensify(reflectivity);
        }

        if let Some(debugger) = ray_debugger_callback {
            debugger(depth, ray, nearest_distance, Some(rt_object), &final_light, &ray_type);
        }

        final_light
    }

    pub fn set_camera_from_vector(&mut self, center: Vector) {
        use super::transformation::Transformation;
        let center = self.transformation_stack
            .get_transformation()
            .expect("Expected a transformation in the stack!")
            .transform_vector(center);
        self.camera = Box::new(PerspectiveCamera::new(
            self.width, self.height, center,
            None, None, None
        ));
    }

    pub fn set_camera(&mut self, camera: Box<dyn Camera>) {
        self.camera = camera;
    }

    pub fn add_light(&mut self, light: PointLight) {
        self.point_lights.push(light);
    }

    pub fn add_object(&mut self, object: RTObject) {
        self.objects.push(object);
    }

    pub fn apply_current_transformation(&self, object: &mut RTObject) {
        object.get_shape_mut().set_transformation(
            self.transformation_stack
                .get_transformation()
                .expect("Expected transformation in stack!")
                .clone()
        );
    }

    pub fn transformation_stack_mut(&mut self) -> &mut TransformationStack {
        &mut self.transformation_stack
    }

    pub fn get_current_transformation(&self) -> &MatrixTransformation {
        self.transformation_stack
            .get_transformation()
            .expect("Expected transformation in stack!")
    }

    fn get_reflected_ray_direction(incident: Vector, normal: Vector) -> Vector {
        incident - (normal * 2.0 * (normal * incident))
    }

    fn get_refracted_ray_direction(
        incident: Vector, normal: Vector, r: f64, total_internal_reflection: &mut bool
    ) -> Vector {
        let cos_1 = (incident * -1.0) * normal;
        let v = 1.0 - r * r * (1.0 - cos_1 * cos_1);

        *total_internal_reflection = v < 0.0;

        if *total_internal_reflection {
            return Vector::new(0.0, 0.0, 0.0);
        }

        let cos_2 = sqrt(v);

        let result = incident * r + normal * (r * cos_1 - cos_2);

        result.normalized()
    }

    pub fn get_objects(&self) -> &Vec<RTObject> {
        &self.objects
    }

    pub fn get_pixel(
        &self, x: f64, y: f64, ray_debugger_callback: RayDebuggerCallback
    ) -> Color {
        self.camera.get_pixel_color(x, y, self, ray_debugger_callback)
    }
}
