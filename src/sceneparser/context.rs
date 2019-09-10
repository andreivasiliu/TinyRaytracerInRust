use crate::raytracer::raytracer::RayTracer;
use super::value::Value;
use super::ast_node::Function;

use std::collections::HashMap;

pub type Identifier = String;

pub struct SceneContext<'a> {
    stack: Vec<HashMap<Identifier, Value>>,
    globals: HashMap<Identifier, Value>,
    functions: HashMap<Identifier, Function>,
    ray_tracer: &'a mut RayTracer,
}

impl<'r> SceneContext<'r> {
    pub fn new(ray_tracer: &'r mut RayTracer) -> SceneContext<'r> {
        Self {
            stack: Default::default(),
            globals: Default::default(),
            functions: Default::default(),
            ray_tracer,
        }
    }

    pub fn locals(&mut self) -> &mut HashMap<Identifier, Value> {
        if let Some(variables) = self.stack.last_mut() {
            variables
        } else {
            &mut self.globals
        }
    }

    pub fn globals(&mut self) -> &mut HashMap<Identifier, Value> {
        &mut self.globals
    }

    pub fn ray_tracer(&mut self) -> &mut RayTracer {
        &mut self.ray_tracer
    }

    pub fn add_function(&mut self, id: Identifier, function: Function) {
        self.functions.insert(id, function);
    }

    pub fn enter_call<'a>(&'a mut self, id: &Identifier) -> Call<'a, 'r> {
        // No unwrap
        let function = self.functions.get(id).unwrap().clone();
        self.stack.push(HashMap::new());

        Call {
            function,
            context: self
        }
    }
}

pub struct Call<'a, 'r> {
    function: Function,
    context: &'a mut SceneContext<'r>,
}

impl Call<'_, '_> {
    pub fn call(&mut self, value_list: Vec<Value>) {
        self.function.call(self.context, value_list);
    }
}

impl Drop for Call<'_, '_> {
    fn drop(&mut self) {
        self.context.stack.pop();
    }
}
