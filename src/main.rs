#![allow(dead_code)]

mod raytracer;
mod raydebugger;
mod sceneparser;

fn main() {
    raydebugger::gui::run_application();
}