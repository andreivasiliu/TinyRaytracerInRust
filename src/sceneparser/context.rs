use super::value::Value;
use super::ast_node::Function;

use std::collections::HashMap;

pub type Identifier = String;

#[derive(Default)]
pub struct SceneContext {
    stack: Vec<HashMap<Identifier, Value>>,
    globals: HashMap<Identifier, Value>,
    functions: HashMap<Identifier, Function>,
}

impl SceneContext {
    pub fn new() -> Self {
        Default::default()
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

    pub fn add_function(&mut self, id: Identifier, function: Function) {
        self.functions.insert(id, function);
    }

    pub fn enter_call(&mut self, id: &Identifier) -> Call {
        // No unwrap
        let function = self.functions.get(id).unwrap().clone();
        self.stack.push(HashMap::new());

        Call {
            function,
            context: self
        }
    }
}

pub struct Call<'a> {
    function: Function,
    context: &'a mut SceneContext,
}

impl Call<'_> {
    pub fn call(&mut self, value_list: Vec<Value>) {
        self.function.call(self.context, value_list);
    }
}

impl Drop for Call<'_> {
    fn drop(&mut self) {
        self.context.stack.pop();
    }
}
