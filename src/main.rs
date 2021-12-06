use bevy::prelude::*;

mod draw;
mod model;

use draw::{clear_canvas, create_canvas, mouse_draw, update_canvas};
use model::{infer_sketch, infer_timer, OnnxModelAsset, OnnxModelLoader};

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

    app.insert_resource(ClearColor(Color::SILVER))
        .add_asset::<OnnxModelAsset>()
        .init_asset_loader::<OnnxModelLoader>()
        .init_resource::<model::State>()
        .add_event::<draw::ImageEvent>()
        .add_startup_system(setup.system())
        .add_system(mouse_draw.system())
        .add_system(update_canvas.system())
        .add_system(clear_canvas.system())
        .add_system(infer_sketch.system())
        .add_system(infer_timer.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(UiCameraBundle::default());

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn().insert(Timer::from_seconds(3.0, true));

    create_canvas(commands, asset_server, materials);
}
