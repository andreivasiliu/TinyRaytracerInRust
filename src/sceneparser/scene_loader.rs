use super::ast_node::AstStatement;
use super::context::SceneContext;

use pest::Parser;
use pest::iterators::Pairs;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "sceneparser/scene_grammar.pest"]
pub struct SceneParser;

const SCENE: &'static str = r###"a = red
function x(f)
    a = sphere(f, 2)
end
call x(green)
"###;

pub fn load_scene() -> Result<(), pest::error::Error<Rule>> {
    let mut pairs: Pairs<Rule> = SceneParser::parse(Rule::scene, SCENE)?;

    let mut context = SceneContext::new();

    let statement_list = pairs.next().unwrap();
    assert_eq!(statement_list.as_rule(), Rule::statement_list);

    let eoi = pairs.next().unwrap();
    assert_eq!(eoi.as_rule(), Rule::EOI);

    let ast = AstStatement::from_pest(statement_list);
    println!("AST statement: {:?}", ast);

    ast.execute(&mut context);
    println!("Result: a = {:?}", context.locals().get("a"));

    Ok(())
}