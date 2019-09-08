#![allow(dead_code)]

mod raytracer;
mod raydebugger;
mod sceneparser;

fn main() {
    if false {
        let scene = sceneparser::scene_loader::load_scene();
        match scene {
            Ok(_scene) => return,
            Err(err) => eprintln!("Parsing error: {}", err),
        }
        return;
    }

    raydebugger::gui::run_application();
}