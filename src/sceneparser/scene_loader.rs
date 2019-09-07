use super::ast_node::AstStatement;
use super::context::SceneContext;

use pest::Parser;
use pest::iterators::Pairs;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "sceneparser/scene_grammar.pest"]
pub struct SceneParser;

const SCENE: &'static str = r###"a = 3
"###;

pub fn load_scene() -> Result<(), pest::error::Error<Rule>> {
    let pairs: Pairs<Rule> = SceneParser::parse(Rule::scene, SCENE)?;

    let mut context = SceneContext::new();

    for pair in pairs {
        if pair.as_rule() == Rule::EOI {
            continue;
        }

        let ast = AstStatement::from_pest(pair);
        println!("AST statement: {:?}", ast);
        ast.execute(&mut context);
    }

    Ok(())
}