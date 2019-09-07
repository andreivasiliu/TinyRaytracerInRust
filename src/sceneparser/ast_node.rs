use super::context::{SceneContext, Identifier, Value};
use super::scene_loader::Rule;

use pest::iterators::Pair;

#[derive(Debug)]
pub enum AstStatement {
    Assignment(bool, Identifier, AstExpression),
}

#[derive(Debug)]
pub enum AstExpression {
    Number(f64),
}

impl AstStatement {
    pub fn execute(&self, context: &mut SceneContext) {
        use AstStatement::*;

        match self {
            Assignment(local, id, expr) => {
                let value = expr.evaluate(context);
                if *local {
                    context.locals().insert(id.to_string(), value);
                } else {
                    context.globals().insert(id.to_string(), value);
                }
            }
        }
    }

    pub fn from_pest(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
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

                AstStatement::Assignment(
                    local, id.as_str().to_string(),
                    AstExpression::from_pest(expr)
                )
            },
            rule => unimplemented!("Unknown statement rule {:?}", rule),
        }
    }
}

impl AstExpression {
    pub fn evaluate(&self, _context: &mut SceneContext) -> Value {
        use AstExpression::*;

        match self {
            Number(number) => Value::Number(*number),
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
                AstExpression::Number(pair.as_str().parse().unwrap())
            }
            rule => unimplemented!("Unknown expression rule {:?}", rule),
        }
    }
}