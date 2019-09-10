use crate::raytracer::raytracer::RayTracer;

use super::ast_node::AstStatement;
use super::context::SceneContext;

use pest::Parser;
use pest::iterators::Pairs;
use pest_derive::Parser;
use std::fs::File;
use std::io::Read;

#[derive(Parser)]
#[grammar = "sceneparser/scene_grammar.pest"]
pub struct SceneParser;

const SCENE: &'static str = "
draw(sphere(<20, -5, 10>, 30, red, 0.5, 0.0))
a = sphere(<-15, -5, -10>, 30)
b = sphere(<-15, -5, -10>, 25)
draw(csg(a, b, 'difference', rgb(0.0, 1.0, 1.0), 0.0, 0.8))
";

pub fn load_scene(ray_tracer: &mut RayTracer) -> Result<(), pest::error::Error<Rule>> {
    let scene = File::open("globes.scene")
        .and_then(|mut file| {
            let mut scene = String::new();
            file.read_to_string(&mut scene)?;
            Ok(scene)
        })
        .unwrap_or(SCENE.to_string());

    let mut context = SceneContext::new(ray_tracer);
    let mut pairs: Pairs<Rule> = SceneParser::parse(Rule::scene, &scene)?;

    let statement_list = pairs.next().unwrap();
    assert_eq!(statement_list.as_rule(), Rule::statement_list);

    let eoi = pairs.next().unwrap();
    assert_eq!(eoi.as_rule(), Rule::EOI);

    let ast = AstStatement::from_pest(statement_list);
    ast.execute(&mut context);

    Ok(())
}
