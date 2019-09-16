use crate::raytracer::color::Color;
use crate::raytracer::vector::Vector;
use crate::raytracer::transformation::MatrixTransformation;
use super::context::{SceneContext, Identifier};
use super::scene_loader::Rule;
use super::value::Value;
use super::shape::{Shape, ShapeKind, CSGOperator};

use pest::iterators::Pair;
use std::rc::Rc;
use std::collections::VecDeque;

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
    CallFunction { id: Identifier, param_list: Vec<AstExpression> },
    Draw { param_list: Vec<AstExpression> },
    Transformation {
        x: Box<AstExpression>, y: Box<AstExpression>, z: Box<AstExpression>,
        transformation: Transformation,
        statement: Box<AstStatement>,
    },
}

#[derive(Debug)]
pub enum AstExpression {
    Value(Value),
    Reference(Identifier),
    Vector { x: Box<AstExpression>, y: Box<AstExpression>, z: Box<AstExpression> },
    Rgb { r: Box<AstExpression>, g: Box<AstExpression>, b: Box<AstExpression> },
    Object { name: String, param_list: Vec<AstExpression> },
    Minus(Box<AstExpression>),
    BinaryOperation { a: Box<AstExpression>, operator: BinaryOperator, b: Box<AstExpression> },
}

#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    LessThan,
    GreaterThan,
}

#[derive(Debug)]
pub enum Transformation {
    Translate,
    Rotate,
    Scale,
}

pub fn expect_id(pair: Pair<Rule>) -> String {
    assert_eq!(pair.as_rule(), Rule::id);

    pair.as_str().to_string()
}

pub fn expect_param_list(pair: Pair<Rule>) -> Vec<AstExpression> {
    assert_eq!(pair.as_rule(), Rule::param_list);

    let mut param_list = Vec::new();
    for pair in pair.into_inner() {
        param_list.push(expect_expression(pair));
    }
    param_list
}

pub fn expect_expression(pair: Pair<Rule>) -> AstExpression {
    assert_eq!(pair.as_rule(), Rule::expression);

    AstExpression::from_pest(pair)
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
                let value_list: Vec<_> = param_list
                    .into_iter()
                    .map(|param| param.evaluate(context))
                    .collect();
                context.enter_call(id).call(value_list);
            }
            AstStatement::Draw { param_list } => {
                let value_list: Vec<_> = param_list
                    .into_iter()
                    .map(|param| param.evaluate(context))
                    .collect();

                assert_eq!(value_list.len(), 1);
                let object = value_list.into_iter().next().unwrap();

                if let Value::Object(shape) = object {
                    context.ray_tracer().add_object(shape.to_rt_object());
                } else {
                    // FIXME: no assert
                    panic!("Didn't get an object on draw!");
                }
            }
            AstStatement::Transformation {
                x, y, z,
                transformation,
                statement,
            } => {
                let x = x.evaluate(context).to_number();
                let y = y.evaluate(context).to_number();
                let z = z.evaluate(context).to_number();

                let matrix_transformation = match transformation {
                    Transformation::Translate => MatrixTransformation::create_translation_matrix(x, y, z),
                    Transformation::Rotate => MatrixTransformation::create_rotation_matrix(x, y, z),
                    Transformation::Scale => MatrixTransformation::create_scaling_matrix(x, y, z),
                };

                // FIXME: RAII
                context
                    .ray_tracer()
                    .transformation_stack_mut()
                    .push_transformation(matrix_transformation);

                statement.execute(context);

                context
                    .ray_tracer()
                    .transformation_stack_mut()
                    .pop_transformation();
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


                let local = if let Some(Rule::local_) = inner.peek().map(|pair| pair.as_rule()) {
                    inner.next().unwrap();
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
                assert_eq!(inner.next(), None);

                AstStatement::CallFunction {
                    id,
                    param_list,
                }
            }
            Rule::command_statement => {
                let mut inner = pair.into_inner();

                // <command> ( <param_list> )

                let command_name = inner.next().unwrap();
                let param_list: Vec<AstExpression> = expect_param_list(inner.next().unwrap());
                assert_eq!(inner.next(), None);

                match command_name.as_str() {
                    "draw" => {
                        AstStatement::Draw { param_list }
                    }
                    "display" | "append" => unimplemented!(),
                    cmd => panic!("Unknown command in grammar: {}", cmd),
                }
            }
            Rule::transformation_statement => {
                let mut inner = pair.into_inner();

                let transformation = inner.next().unwrap();
                assert_eq!(transformation.as_rule(), Rule::transformation_);

                let x = expect_expression(inner.next().unwrap());
                let y = expect_expression(inner.next().unwrap());
                let z = expect_expression(inner.next().unwrap());

                let statement = inner.next().unwrap();

                let transformation = match transformation.as_str() {
                    "translate" => Transformation::Translate,
                    "scale" => Transformation::Scale,
                    "rotate" => Transformation::Rotate,
                    transformation => panic!("Unknown transformation '{}'", transformation),
                };

                AstStatement::Transformation {
                    x: Box::new(x),
                    y: Box::new(y),
                    z: Box::new(z),
                    transformation,
                    statement: Box::new(AstStatement::from_pest(statement)),
                }
            }
            Rule::do_statement => {
                let mut inner = pair.into_inner();

                let do_ = inner.next().unwrap();
                assert_eq!(do_.as_rule(), Rule::do_);

                let statement_list = inner.next().unwrap();
                assert_eq!(statement_list.as_rule(), Rule::statement_list);

                let end_ = inner.next().unwrap();
                assert_eq!(end_.as_rule(), Rule::end_);

                AstStatement::from_pest(statement_list)
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
            AstExpression::Vector { x, y, z } => {
                let x = x.evaluate(context).to_number();
                let y = y.evaluate(context).to_number();
                let z = z.evaluate(context).to_number();

                Value::Vector { x, y, z }
            }
            AstExpression::Rgb { r, g, b } => {
                let r = r.evaluate(context).to_number();
                let g = g.evaluate(context).to_number();
                let b = b.evaluate(context).to_number();

                Value::Color { r, g, b, a: 1.0 }
            }
            AstExpression::Object { name, param_list } => {
                let value_list = param_list
                    .iter().map(|param| param.evaluate(context));

                let mut numbers = VecDeque::new();
                let mut strings = VecDeque::new();
                let mut vectors = VecDeque::new();
                let mut objects = VecDeque::new();
                let mut colors = VecDeque::new();

                for value in value_list {
                    match value {
                        Value::Number(number) => numbers.push_back(number),
                        Value::String(string) => strings.push_back(string),
                        Value::Color { r, g, b, a } => {
                            colors.push_back(Color::new(r, g, b, a))
                        },
                        Value::Vector { x, y, z } => {
                            vectors.push_back(Vector::new(x, y, z))
                        },
                        Value::Object(shape) => objects.push_back(shape),
                        // FIXME: No panic
                        Value::Texture(_) => panic!("Unexpected argument type: texture"),
                        Value::Boolean(_) => panic!("Unexpected argument type: boolean"),
                    };
                }

                let shape_kind = match name.as_str() {
                    "sphere" => ShapeKind::Sphere {
                        center: vectors.pop_front().unwrap_or(Vector::new(0.0, 0.0, 0.0)),
                        radius: numbers.pop_front().unwrap_or(1.0),
                    },
                    "cube" => ShapeKind::Cube {
                        center: vectors.pop_front().unwrap_or(Vector::new(0.0, 0.0, 0.0)),
                        length: numbers.pop_front().unwrap_or(1.0),
                    },
                    "plane" => ShapeKind::Plane {
                        normal: vectors.pop_front().unwrap_or(Vector::new(0.0, 1.0, 0.0)),
                        distance: numbers.pop_front().unwrap_or(1.0),
                    },
                    "csg" => {
                        let operator = strings.pop_front();
                        let operator = operator
                            .as_ref()
                            .map(|string| string.as_str())
                            .unwrap_or("union");
                        ShapeKind::CSG {
                            operator: match operator {
                                "union" => CSGOperator::Union,
                                "intersection" => CSGOperator::Intersection,
                                "difference" => CSGOperator::Difference,
                                // FIXME: No panic
                                operator => panic!("Unknown CSG operator: {}", operator),
                            },
                            // FIXME: No expect
                            a: Box::new(objects.pop_front().expect("Expected object 1!")),
                            b: Box::new(objects.pop_front().expect("Expected object 2!")),
                        }
                    },
                    kind => panic!("Unknown shape type in grammar: {}", kind),
                };

                let transformation =
                    context.ray_tracer().get_current_transformation().clone();

                let object = Shape {
                    color: colors.pop_front().unwrap_or(Color::BLACK),
                    reflectivity: numbers.pop_front().unwrap_or(0.0),
                    transparency: numbers.pop_front().unwrap_or(0.0),
                    kind: shape_kind,
                    transformation,
                };

                // FIXME: No assert
                assert!(numbers.pop_front().is_none());
                assert!(strings.pop_front().is_none());
                assert!(vectors.pop_front().is_none());
                assert!(objects.pop_front().is_none());
                assert!(colors.pop_front().is_none());

                Value::Object(object)
            }
            AstExpression::Minus(expression) => {
                match expression.evaluate(context) {
                    Value::Number(number) => Value::Number(-number),
                    Value::Vector { x, y, z } => {
                        Value::Vector { x: -x, y: -y, z: -z }
                    },
                    // FIXME: No panic
                    value => panic!("Cannot apply - to {:?}", value),
                }
            }
            AstExpression::BinaryOperation { a, operator, b } => {
                let a = a.evaluate(context);
                let b = b.evaluate(context);

                match operator {
                    BinaryOperator::Add => Value::Number(a.to_number() + b.to_number()),
                    BinaryOperator::Subtract => Value::Number(a.to_number() - b.to_number()),
                    BinaryOperator::Multiply => {
                        match (a, b) {
                            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
                            (Value::Color { r, g, b, a }, Value::Number(x))
                            | (Value::Number(x), Value::Color { r, g, b, a }) => {
                                Value::Color { r: r * x, g: g * x, b: b * x, a: a * x }
                            }
                            (Value::Vector { x, y, z }, Value::Number(b))
                            | (Value::Number(b), Value::Vector { x, y, z }) => {
                                Value::Vector { x: x * b, y: y * b, z: z * b }
                            }
                            // FIXME: No panic
                            (x, y) => panic!("Cannot multiply {:?} and {:?}", x, y),
                        }
                    }
                    operator => unimplemented!("Operator {:?} not yet implemented", operator),
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

                if let Some(operator) = operator {
                    let expr_right = inner.next().unwrap();

                    let operator = match operator.as_str() {
                        "+" => BinaryOperator::Add,
                        "-" => BinaryOperator::Subtract,
                        "*" => BinaryOperator::Multiply,
                        "/" => BinaryOperator::Divide,
                        "%" => BinaryOperator::Modulo,
                        ">" => BinaryOperator::GreaterThan,
                        "<" => BinaryOperator::LessThan,
                        operator => panic!("Unknown operator '{}' in the grammar", operator),
                    };

                    AstExpression::BinaryOperation {
                        a: Box::new(AstExpression::from_pest(expr_left)),
                        operator,
                        b: Box::new(AstExpression::from_pest(expr_right)),
                    }
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
                    AstExpression::Minus(Box::new(AstExpression::from_pest(value)))
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
            Rule::vector => {
                let mut inner = pair.into_inner();

                let x = expect_expression(inner.next().unwrap());
                let y = expect_expression(inner.next().unwrap());
                let z = expect_expression(inner.next().unwrap());
                assert_eq!(inner.next(), None);

                AstExpression::Vector {
                    x: Box::new(x),
                    y: Box::new(y),
                    z: Box::new(z),
                }
            }
            Rule::color => {
                let mut inner = pair.into_inner();

                let r = expect_expression(inner.next().unwrap());
                let g = expect_expression(inner.next().unwrap());
                let b = expect_expression(inner.next().unwrap());
                assert_eq!(inner.next(), None);

                AstExpression::Rgb {
                    r: Box::new(r),
                    g: Box::new(g),
                    b: Box::new(b),
                }
            }
            Rule::string_literal => {
                let string_with_quotes = pair.as_str();
                let string = &string_with_quotes[1..string_with_quotes.len()-1];

                AstExpression::Value(Value::String(string.to_string()))
            }
            _ => unimplemented!("Unimplemented rule: {}", pair)
        }
    }
}
