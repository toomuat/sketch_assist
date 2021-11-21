use bevy::prelude::*;

mod draw;

use bevy_rapier2d::physics::{NoUserData, RapierPhysicsPlugin};
use draw::{clear_window, create_canvas, line_drawing_system, LineMaterial, MouseCoord};
use std::collections::VecDeque;

fn main() {
    let window_desc = WindowDescriptor {
        width: 1350.0,
        height: 700.0,
        title: "Sketch Assist".to_string(),
        ..Default::default()
    };

    let mut app = App::build();

    app.insert_resource(window_desc).add_plugins(DefaultPlugins);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(ClearColor(Color::rgb(0.7, 0.7, 0.7)))
        .add_startup_system(setup.system())
        .add_system(line_drawing_system.system())
        .add_system(clear_window.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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

    create_canvas(commands, asset_server, materials, windows);
}
