use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    prelude::*,
    reflect::TypeUuid,
    window::CursorMoved,
};
use tract_onnx::prelude::*;

pub struct Canvas;

pub enum ImageEvent {
    DrawPos(Vec2),
    Clear,
}

#[derive(TypeUuid)]
#[uuid = "ea2073f7-2a59-4983-85cd-6370ea9101a2"]
pub struct OnnxModelAsset {
    pub model: SimplePlan<
        TypedFact,
        Box<dyn TypedOp>,
        tract_onnx::prelude::Graph<TypedFact, Box<dyn TypedOp>>,
    >,
}

#[derive(Default)]
pub struct OnnxModelLoader;

impl AssetLoader for OnnxModelLoader {
    fn load<'a>(
        &'a self,
        mut bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let model = tract_onnx::onnx()
                .model_for_read(&mut bytes)
                .unwrap()
                .into_optimized()?
                .into_runnable()?;

            load_context.set_default_asset(LoadedAsset::new(OnnxModelAsset { model }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["onnx"]
    }
}

const WINDOW_WIDTH: f32 = 1350.;
const WINDOW_HEIGHT: f32 = 700.;

// Offset from left top corner
const OFFSET: f32 = WINDOW_HEIGHT / 14.;
const CANVAS_WIDTH: f32 = (WINDOW_WIDTH - OFFSET * 3.0) / 2.0;
const CANVAS_HEIGHT: f32 = WINDOW_HEIGHT - OFFSET * 2.0;

pub fn clear_canvas(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut image_events: EventWriter<ImageEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::C) {
        clear_inference(
            &mut commands,
            &mut materials,
            WINDOW_WIDTH / 2.0 - CANVAS_WIDTH / 2.0 - OFFSET,
        );

        image_events.send(ImageEvent::Clear);
    }
}

pub fn create_canvas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    create_canvas_(&mut commands, &mut materials, &asset_server);

    // Setup images on the right canvas

    let texture1 = asset_server.load("axe1.png");
    let texture2 = asset_server.load("axe2.png");
    let texture3 = asset_server.load("axe3.png");
    let texture4 = asset_server.load("axe4.png");

    // Upper left
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(CANVAS_WIDTH / 2., CANVAS_HEIGHT / 2.)),
        material: materials.add(texture1.into()),
        transform: Transform {
            translation: Vec3::new(
                WINDOW_WIDTH / 2. - OFFSET - CANVAS_WIDTH / 2. - CANVAS_WIDTH / 4.,
                WINDOW_HEIGHT / 2. - OFFSET - CANVAS_HEIGHT / 4.,
                0.,
            ),
            ..Default::default()
        },
        ..Default::default()
    });
    // Upper right
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(CANVAS_WIDTH / 2., CANVAS_HEIGHT / 2.)),
        material: materials.add(texture2.into()),
        transform: Transform {
            translation: Vec3::new(
                WINDOW_WIDTH / 2. - OFFSET - CANVAS_WIDTH / 4.,
                WINDOW_HEIGHT / 2. - OFFSET - CANVAS_HEIGHT / 4.,
                0.,
            ),
            ..Default::default()
        },
        ..Default::default()
    });
    // Lower left
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(CANVAS_WIDTH / 2., CANVAS_HEIGHT / 2.)),
        material: materials.add(texture3.into()),
        transform: Transform {
            translation: Vec3::new(
                WINDOW_WIDTH / 2. - OFFSET - CANVAS_WIDTH / 2. - CANVAS_WIDTH / 4.,
                -(WINDOW_HEIGHT / 2. - OFFSET - CANVAS_HEIGHT / 4.),
                0.,
            ),
            ..Default::default()
        },
        ..Default::default()
    });
    // Lower right
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(CANVAS_WIDTH / 2., CANVAS_HEIGHT / 2.)),
        material: materials.add(texture4.into()),
        transform: Transform {
            translation: Vec3::new(
                WINDOW_WIDTH / 2. - OFFSET - CANVAS_WIDTH / 4.,
                -(WINDOW_HEIGHT / 2. - OFFSET - CANVAS_HEIGHT / 4.),
                0.,
            ),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn create_canvas_(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    commands
        .spawn_bundle(ImageBundle {
            style: Style {
                size: Size::new(Val::Px(CANVAS_WIDTH), Val::Px(CANVAS_HEIGHT)),
                position: Rect {
                    left: Val::Px(OFFSET),
                    top: Val::Px(OFFSET),
                    ..Default::default()
                },
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            material: materials.add(asset_server.load("empty.png").into()),
            ..Default::default()
        })
        .insert(Canvas)
        .insert(Interaction::None);

    clear_inference(
        commands,
        materials,
        WINDOW_WIDTH / 2.0 - CANVAS_WIDTH / 2.0 - OFFSET,
    );
}

// Area to show images on right side
fn clear_inference(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    x_offset: f32,
) {
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(CANVAS_WIDTH, CANVAS_HEIGHT)),
        material: materials.add(Color::WHITE.into()),
        transform: Transform {
            translation: Vec3::new(x_offset, 0., 0.),
            ..Default::default()
        },
        ..Default::default()
    });
}

pub fn mouse_draw(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut image_events: EventWriter<ImageEvent>,
    mut last_mouse_position: Local<Option<Vec2>>,
    drawable: Query<(&Interaction, &GlobalTransform, &Style), With<Canvas>>,
) {
    for (interaction, transform, style) in drawable.iter() {
        if let Interaction::Hovered = interaction {
            // println!("Hovered");
        }
        if let Interaction::Clicked = interaction {
            // println!("Clicked");
            // dbg!(style);
            // dbg!(transform);

            let width = if let Val::Px(x) = style.size.width {
                x
            } else {
                0.
            };
            let height = if let Val::Px(x) = style.size.height {
                x
            } else {
                0.
            };

            // dbg!(transform.translation);
            // [examples/ui/ui.rs:89] transform.translation = Vec3(
            //     400.0,
            //     320.0,
            //     0.001,
            // )

            for event in cursor_moved_events.iter() {
                // info!("{:?}", event.position);

                if let Some(last_mouse_position) = *last_mouse_position {
                    // dbg!(last_mouse_position);
                    // dbg!(last_mouse_position.distance(event.position));

                    let steps =
                        (last_mouse_position.distance(event.position) as u32 / 400 + 1) * 10;

                    for i in 0..steps {
                        let lerped =
                            last_mouse_position.lerp(event.position, i as f32 / steps as f32);
                        let x = lerped.x - transform.translation.x + width / 2.;
                        let y = lerped.y - transform.translation.y + height / 2.;

                        // let y = 400 as f32 - y;

                        // println!("{}, {}", x, y);

                        image_events.send(ImageEvent::DrawPos(Vec2::new(x, y)));
                    }
                } else {
                    let x = event.position.x - transform.translation.x + width / 2.;
                    let y = event.position.y - transform.translation.y + height / 2.;
                    image_events.send(ImageEvent::DrawPos(Vec2::new(x, y)));
                }

                *last_mouse_position = Some(event.position);
            }
        } else {
            // println!("None");
        }
    }
}

pub fn update_canvas(
    mut image_events: EventReader<ImageEvent>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut textures: ResMut<Assets<Texture>>,
    mut canvas: Query<(&bevy::ui::Node, &mut Handle<ColorMaterial>), With<Canvas>>,
) {
    for event in image_events.iter() {
        let (_node, mat) = canvas.iter_mut().next().unwrap();
        let material = materials.get_mut(mat.clone()).unwrap();
        let texture = textures
            .get_mut(material.texture.as_ref().unwrap())
            .unwrap();

        match event {
            ImageEvent::DrawPos(pos) => {
                let x_scale = texture.size.width as f32 / CANVAS_WIDTH;
                let y_scale = texture.size.height as f32 / CANVAS_HEIGHT;
                let line_scale = 5;
                let line_radius = 5;

                for i in -line_radius..=line_radius {
                    for j in -line_radius..=line_radius {
                        let target_point = Vec2::new(pos.x + i as f32, pos.y + j as f32);
                        if pos.distance(target_point) < line_radius as f32 {
                            for i in 0..=line_scale {
                                for j in 0..=line_scale {
                                    let x = target_point.x * x_scale;
                                    let y = (CANVAS_HEIGHT - target_point.y) * y_scale;

                                    set_pixel(x as i32 + i, y as i32 + j, Color::BLACK, texture);
                                }
                            }
                        }
                    }
                }
            }
            ImageEvent::Clear => {
                for x in 0..texture.size.width as i32 {
                    for y in 0..texture.size.height as i32 {
                        set_pixel(x, y, Color::WHITE, texture);
                    }
                }
            }
        }
    }
}

fn set_pixel(x: i32, y: i32, color: Color, texture: &mut Texture) {
    if x < 0 || texture.size.width as i32 - 1 < x {
        return;
    }
    if y < 0 || texture.size.height as i32 - 1 < y {
        return;
    }

    let x = x as usize;
    let h = (y as u32 * texture.size.width) as usize;
    let offset = (x + h) * 4;

    texture.data[offset] = (color.r() * 255.) as u8;
    texture.data[offset + 1] = (color.g() * 255.) as u8;
    texture.data[offset + 2] = (color.b() * 255.) as u8;
}

#[allow(unused)]
pub fn print_mouse_events_system(
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    for event in mouse_button_input_events.iter() {
        info!("mouse_button_input_events: {:?}", event);
    }

    for event in mouse_motion_events.iter() {
        info!("mouse_motion_events: {:?}", event);
    }

    for event in cursor_moved_events.iter() {
        info!("cursor_moved_events: {:?}", event);
        // info!("cursor_moved_events: {:?}", event.position);
        // dbg!(event.position.x, event.position.y);
    }

    for event in mouse_wheel_events.iter() {
        info!("mouse_wheel_events: {:?}", event);
    }
}
