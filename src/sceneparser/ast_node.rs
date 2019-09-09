use super::context::{SceneContext, Identifier};
use super::scene_loader::Rule;
use super::value::Value;
use super::shape::{Shape, ShapeKind};

use pest::iterators::Pair;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Function {
    id: String,
    param_list: Vec<String>,
    body: Rc<AstStatement>,
}

impl Function {
    pub fn call(&self, context: &mut SceneContext, value_list: Vec<Value>) {
        assert_eq!(self.param_list.len(), value_list.len());

        for (param_name, value) in self.param_list.iter().zip(value_list) {
            context.locals().insert(param_name.clone(), value);
        }

        self.body.execute(context)
    }
}

#[derive(Debug)]
pub enum AstStatement {
    StatementList(Vec<AstStatement>),
    Assignment { local: bool, id: Identifier, expression: AstExpression },
    Function(Function),
    CallFunction { id: Identifier, param_list: Vec<AstExpression> }
}

#[derive(Debug)]
pub enum AstExpression {
    Value(Value),
    Reference(Identifier),
    Object { name: String, param_list: Vec<AstExpression> },
}

pub fn expect_id(pair: Pair<Rule>) -> String {
    assert_eq!(pair.as_rule(), Rule::id);

    pair.as_str().to_string()
}

pub fn expect_param_list(pair: Pair<Rule>) -> Vec<AstExpression> {
    assert_eq!(pair.as_rule(), Rule::param_list);

    let mut param_list = Vec::new();
    for pair in pair.into_inner() {
        assert_eq!(pair.as_rule(), Rule::expression);
        param_list.push(AstExpression::from_pest(pair));
    }
    param_list
}

impl AstStatement {
    pub fn execute(&self, context: &mut SceneContext) {
        match self {
            AstStatement::StatementList(statement_list) => {
                for statement in statement_list {
                    statement.execute(context);
                }
            }
            AstStatement::Assignment { local, id, expression } => {
                let value = expression.evaluate(context);
                if *local {
                    context.locals().insert(id.to_string(), value);
                } else {
                    context.globals().insert(id.to_string(), value);
                }
            }
            AstStatement::Function(function) => {
                context.add_function(function.id.clone(), function.clone());
            }
            AstStatement::CallFunction { id, param_list } => {
                // FIXME: no unwrap
                let value_list: Vec<_> = param_list
                    .into_iter()
                    .map(|param| param.evaluate(context))
                    .collect();
                context.enter_call(id).call(value_list);
            }
        }
    }

    pub fn from_pest(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::statement_list => {
                let inner = pair.into_inner();
                let mut statement_list = Vec::new();

                for pair in inner {
                    statement_list.push(AstStatement::from_pest(pair));
                }

                AstStatement::StatementList(statement_list)
            }
            Rule::assignment_statement => {
                let mut inner = pair.into_inner();

                let local = if let Some("local") = inner.peek().map(|pair| pair.as_str()) {
                    true
                } else {
                    false
                };

                let id = inner.next().unwrap();
                let expr = inner.next().unwrap();

                assert_eq!(id.as_rule(), Rule::id);
                assert_eq!(expr.as_rule(), Rule::expression);

                AstStatement::Assignment {
                    local,
                    id: id.as_str().to_string(),
                    expression: AstExpression::from_pest(expr)
                }
            }
            Rule::function_statement => {
                let mut inner = pair.into_inner();

                // function <id> ( <id>* ) <statement_list> end

                assert_eq!(inner.next().unwrap().as_rule(), Rule::function_);

                let function_id = expect_id(inner.next().unwrap());
                let mut param_list = Vec::new();
                let statement_list;

                loop {
                    let pair = inner.next().unwrap();

                    if pair.as_rule() == Rule::id {
                        param_list.push(expect_id(pair));
                    } else if pair.as_rule() == Rule::statement_list {
                        statement_list = AstStatement::from_pest(pair);
                        break;
                    } else {
                        unreachable!()
                    }
                }

                assert_eq!(inner.next().unwrap().as_rule(), Rule::end_);
                AstStatement::Function(Function {
                    id: function_id,
                    param_list,
                    body: Rc::new(statement_list),
                })
            }
            Rule::call_statement => {
                let mut inner = pair.into_inner();

                // call <id> ( <param_list> )

                assert_eq!(inner.next().unwrap().as_rule(), Rule::call_);
                let id = expect_id(inner.next().unwrap());
                let param_list: Vec<AstExpression> = expect_param_list(inner.next().unwrap());

                AstStatement::CallFunction {
                    id,
                    param_list,
                }
            }
            rule => unimplemented!("Unknown statement rule {:?}", rule),
        }
    }
}

impl AstExpression {
    pub fn evaluate(&self, context: &mut SceneContext) -> Value {
        match self {
            AstExpression::Value(value) => value.clone(),
            AstExpression::Reference(id) => {
                if let Some(local) = context.locals().get(id) {
                    local.clone()
                } else if let Some(global) = context.globals().get(id) {
                    global.clone()
                } else {
                    // FIXME: no panic
                    unimplemented!("Didn't find variable, don't know how to error")
                }
            }
            AstExpression::Object { name, param_list } => {
                let value_list = param_list
                    .iter().map(|param| param.evaluate(context));

                match name.as_str() {
                    "sphere" => {
                        let mut radius = None;
                        let mut color = None;
                        let mut reflectivity = 0.0;
                        let mut transparency = 0.0;
                        let mut param_number = 0;

                        for value in value_list {
                            match value {
                                Value::Number(number) => {
                                    param_number += 1;

                                    match param_number {
                                        1 => radius = Some(number),
                                        2 => reflectivity = number,
                                        3 => transparency = number,
                                        // FIXME: No panic
                                        _ => panic!("Unknown sphere parameter!")
                                    }
                                }
                                Value::Color { r, g, b, a } => {
                                    color = Some((r, g, b, a));
                                }
                                _ => panic!("Unknown sphere parameter!")
                            }
                        }

                        // FIXME: No panic
                        let radius = match radius {
                            Some(radius) => radius,
                            None => panic!("No size given to sphere object!"),
                        };

                        Value::Object(Shape {
                            color,
                            reflectivity,
                            transparency,
                            kind: ShapeKind::Sphere {
                                radius,
                            },
                        })
                    }
                    _ => unimplemented!("Shape {} not yet implemented", name),
                }
            }
        }
    }

    pub fn from_pest(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::expression | Rule::mult_expression => {
                let mut inner = pair.into_inner();

                let expr_left = inner.next().unwrap();
                let operator = inner.next();

                if let Some(_operator) = operator {
                    let _expr_right = inner.next().unwrap();

                    unimplemented!()
                } else {
                    assert_eq!(inner.next(), None);
                    AstExpression::from_pest(expr_left)
                }
            }
            Rule::neg_expression => {
                let mut inner = pair.into_inner();
                let mut minus = false;

                let possibly_minus = inner.peek().map(|pair| pair.as_rule());
                if let Some(Rule::minus) = possibly_minus {
                    minus = true;
                    inner.next().unwrap();
                }

                let value = inner.next().unwrap();
                assert_eq!(inner.next(), None);
                assert_eq!(value.as_rule(), Rule::value);

                if minus {
                    unimplemented!()
                } else {
                    AstExpression::from_pest(value)
                }
            }
            Rule::value => {
                let mut inner = pair.into_inner();

                let expr = inner.next().unwrap();
                assert_eq!(inner.next(), None);

                AstExpression::from_pest(expr)
            }
            Rule::number_literal => {
                AstExpression::Value(Value::Number(pair.as_str().parse().unwrap()))
            }
            Rule::color_name => {
                let (r, g, b) = match pair.as_str() {
                    "red" => (1.0, 0.0, 0.0),
                    "orange" => (1.0, 0.5, 0.0),
                    "yellow" => (1.0, 1.0, 0.0),
                    "green" => (0.0, 1.0, 0.0),
                    "blue" => (0.0, 0.0, 1.0),
                    "purple" => (1.0, 0.0, 1.0),
                    "black" => (0.0, 0.0, 0.0),
                    "white" => (1.0, 1.0, 1.0),
                    color => panic!("Invalid color in pest grammar: '{}'", color)
                };

                AstExpression::Value(Value::Color { r, g, b, a: 1.0 })
            }
            Rule::id_reference => {
                AstExpression::Reference(pair.as_str().to_string())
            }
            Rule::object => {
                let mut inner = pair.into_inner();

                // obj_name ( <param_list> )

                let obj_name = inner.next().unwrap();
                assert_eq!(obj_name.as_rule(), Rule::obj_name);

                let param_list = expect_param_list(inner.next().unwrap());
                assert_eq!(inner.next(), None);

                AstExpression::Object { name: obj_name.as_str().to_string(), param_list }
            }
            rule => unimplemented!("Unknown expression rule {:?}", rule),
        }
    }
}