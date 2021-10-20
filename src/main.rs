use bevy::prelude::*;

mod draw;

use draw::{line_drawing_system, MouseCoord};
use std::collections::VecDeque;

fn main() {
    let mut window_desc = WindowDescriptor::default();
    window_desc.width = 1300.0;
    window_desc.height = 600.0;
    window_desc.title = "Sketch Assist".to_string();

    App::build()
        .insert_resource(window_desc)
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.7, 0.7, 0.7)))
        .add_startup_system(setup.system())
        .add_system(line_drawing_system.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}

fn setup(mut commands: Commands) {
    commands.insert_resource(MouseCoord {
        mouse_coord: VecDeque::new(),
    });
}
