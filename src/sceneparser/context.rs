use std::collections::HashMap;

pub type Identifier = String;

#[derive(Default)]
pub struct SceneContext {
    stack: Vec<HashMap<Identifier, Value>>,
    globals: HashMap<Identifier, Value>,
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
}

pub enum Value {
    Number(f64),
}
