use bevy::prelude::*;

mod draw;

use draw::{line_drawing_system, LineMaterial, MouseCoord};
use std::collections::VecDeque;

fn main() {
    let window_desc = WindowDescriptor {
        width: 1300.0,
        height: 600.0,
        title: "Sketch Assist".to_string(),
        ..Default::default()
    };

    App::build()
        .insert_resource(window_desc)
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.7, 0.7, 0.7)))
        .add_startup_system(setup.system())
        .add_system(line_drawing_system.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Res<Windows>,
) {
    let camera_entity = commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .id();

    commands.insert_resource(MouseCoord {
        mouse_coord: VecDeque::new(),
        camera_entity,
    });
    commands.insert_resource(LineMaterial(
        materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
    ));
}
