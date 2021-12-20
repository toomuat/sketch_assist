use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
};
use image::{imageops::FilterType, ImageBuffer, RgbImage};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tract_ndarray::Array;
use tract_onnx::prelude::*;
use wasm_bindgen::prelude::*;

use crate::draw::{
    Canvas, ImageEvent, TestCanvas, CANVAS_HEIGHT, CANVAS_WIDTH, OFFSET, WINDOW_HEIGHT,
    WINDOW_WIDTH,
};

const INPUT_IMG_SIZE: u32 = 128;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn time(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn timeEnd(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
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

#[derive(PartialEq)]
pub enum InferenceState {
    Wait,
    Infer,
}

pub struct State {
    pub model: Handle<OnnxModelAsset>,
    pub inference_state: InferenceState,
}

enum ImageClass {
    Rabbit = 1,
    Axe,
    SmileyFace,
}

impl FromWorld for State {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        State {
            inference_state: InferenceState::Wait,
            model: asset_server.load("cnn_sketch_3class.onnx"),
        }
    }
}

pub fn infer_sketch(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut image_events: EventReader<ImageEvent>,
    keyboard_input: Res<Input<KeyCode>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    textures: Res<Assets<Texture>>,
    models: Res<Assets<OnnxModelAsset>>,
    mut state: ResMut<State>,
    drawable: Query<&Handle<ColorMaterial>, With<Canvas>>,
) {
    // If canvas is cleared and nothing drawed then return without inference
    for event in image_events.iter() {
        match event {
            ImageEvent::Clear => return,
            _ => (),
        }
    }

    if keyboard_input.just_pressed(KeyCode::B) && state.inference_state == InferenceState::Infer {
        for mat in drawable.iter() {
            let material = &materials.get(mat).unwrap();
            let texture = textures.get(material.texture.as_ref().unwrap()).unwrap();

            let mut img: RgbImage = ImageBuffer::new(texture.size.width, texture.size.height);

            // Copy image data from texture to ImageBuffer
            for i in 0..texture.size.width {
                for j in 0..texture.size.height {
                    let idx = (i as usize + (j as u32 * texture.size.width) as usize) * 4;
                    let r = texture.data[idx];
                    let g = texture.data[idx + 1];
                    let b = texture.data[idx + 2];
                    img[(i as u32, j as u32)] = image::Rgb([r, g, b]);
                }
            }

            // #[cfg(not(target_arch = "wasm32"))]
            // img.save("image.png").unwrap();

            let resized =
                image::imageops::resize(&img, INPUT_IMG_SIZE, INPUT_IMG_SIZE, FilterType::Triangle);

            // let tensor_image = tract_ndarray::Array4::from_shape_fn(
            //     (1, 1, INPUT_IMG_SIZE as usize, INPUT_IMG_SIZE as usize),
            //     |(_, _, y, x)| resized[(x as _, y as _)][0] as f32,
            // );
            // println!("tensor_image shape: {:?}", tensor_image.shape());
            // for i in 0..10 {
            //     for j in 0..10 {
            //         let a = resized.get_pixel(i, j);
            //         print!("({} {} {}),", a[0], a[1], a[2]);
            //     }
            //     println!();
            // }

            let tensor_image: Tensor = ((tract_ndarray::Array3::from_shape_fn(
                (1, INPUT_IMG_SIZE as usize, INPUT_IMG_SIZE as usize),
                |(_, y, x)| {
                    // Convert RGB to gray scale value
                    // let r = resized[(x as _, y as _)][0] as f32;
                    // let g = resized[(x as _, y as _)][1] as f32;
                    // let b = resized[(x as _, y as _)][2] as f32;
                    // (r * 0.3 + g * 0.59 + b * 0.11) / 255.0
                    resized[(x as _, y as _)][0] as f32 / 255.0
                },
            ) - 0.5)
                / 0.5)
                .into();

            #[cfg(not(target_arch = "wasm32"))]
            resized.save("resized.png").unwrap();

            // dbg!(&tensor_image.shape());
            // dbg!(&tensor_image);
            for i in 0..10 {
                for j in 0..10 {
                    // print!("{:?} ", resized[(i, j)]);
                    // dbg!(resized[(i, j)]);

                    // let a = resized.get_pixel(i, j);
                    // print!("[({} {} {}),", a[0], a[1], a[2]);
                }
                // println!();
            }

            #[cfg(not(target_arch = "wasm32"))]
            let start = Instant::now();

            // #[cfg(target_arch = "wasm32")]
            // console_log!("aaaaaaa");
            // return;

            #[cfg(target_arch = "wasm32")]
            time("infer");

            if let Some(model) = models.get(state.model.as_weak::<OnnxModelAsset>()) {
                let result = model.model.run(tvec!(tensor_image)).unwrap();

                // find and display the max value with its index
                let best = result[0]
                    .to_array_view::<f32>()
                    .unwrap()
                    .iter()
                    .cloned()
                    .zip(1..)
                    .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                let (score, class) = best.unwrap();

                println!("{} {}", score, class);

                // let a = result[0].to_array_view::<f32>().unwrap();
                // println!("{}", a);

                #[cfg(not(target_arch = "wasm32"))]
                let duration = start.elapsed();
                #[cfg(not(target_arch = "wasm32"))]
                println!("Inference time: {:?}", duration);

                #[cfg(target_arch = "wasm32")]
                console_log!("{} {}", score, class);
                #[cfg(target_arch = "wasm32")]
                timeEnd("infer");

                show_infer_result(&mut commands, &asset_server, &mut materials, class);
            }

            state.inference_state = InferenceState::Wait;
        }
    }
}

fn show_infer_result(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    class: u32,
) {
    let path = match class {
        1 => "rabbit",
        2 => "axe",
        3 => "smile",
        _ => "err",
    };
    let path = path.to_string();

    let texture1 = asset_server.load(PathBuf::from(path.clone() + "1.png"));
    let texture2 = asset_server.load(PathBuf::from(path.clone() + "2.png"));
    let texture3 = asset_server.load(PathBuf::from(path.clone() + "3.png"));
    let texture4 = asset_server.load(PathBuf::from(path + "4.png"));

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

pub fn infer_timer(time: Res<Time>, mut state: ResMut<State>, mut query: Query<&mut Timer>) {
    for mut timer in query.iter_mut() {
        if timer.tick(time.delta()).finished() {
            // info!("Entity timer just finished");

            state.inference_state = InferenceState::Infer;
        }
    }
}
