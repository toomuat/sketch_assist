use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
};
use image::{imageops::FilterType, ImageBuffer, RgbImage};
use std::time::{Duration, Instant};
use tract_ndarray::Array;
use tract_onnx::prelude::*;
use wasm_bindgen::prelude::*;

use crate::draw::{ImageEvent, TestCanvas};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
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

impl FromWorld for State {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        State {
            inference_state: InferenceState::Wait,
            model: asset_server.load("resnet50.onnx"),
        }
    }
}

pub fn infer_sketch(
    mut image_events: EventReader<ImageEvent>,
    keyboard_input: Res<Input<KeyCode>>,
    materials: ResMut<Assets<ColorMaterial>>,
    textures: Res<Assets<Texture>>,
    models: Res<Assets<OnnxModelAsset>>,
    mut state: ResMut<State>,
    drawable: Query<&Handle<ColorMaterial>, With<TestCanvas>>,
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
            println!("Save image");

            let material = materials.get(mat).unwrap();
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

            #[cfg(not(target_arch = "wasm32"))]
            img.save("image.png").unwrap();

            // Imagenet mean and standard deviation
            let mean = Array::from_shape_vec((1, 3, 1, 1), vec![0.485, 0.456, 0.406]).unwrap();
            let std = Array::from_shape_vec((1, 3, 1, 1), vec![0.229, 0.224, 0.225]).unwrap();

            let resized = image::imageops::resize(&img, 224, 224, FilterType::Triangle);
            let tensor_image: Tensor =
                ((tract_ndarray::Array4::from_shape_fn((1, 3, 224, 224), |(_, c, y, x)| {
                    resized[(x as _, y as _)][c] as f32 / 255.0
                }) - mean)
                    / std)
                    .into();

            let start = Instant::now();

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

                let duration = start.elapsed();
                println!("Inference time: {:?}", duration);

                #[cfg(target_arch = "wasm32")]
                console_log!("{} {}", score, class);
                #[cfg(target_arch = "wasm32")]
                console_log!("Inference time: {:?}", duration);
            }

            state.inference_state = InferenceState::Wait;
        }
    }
}

pub fn infer_timer(time: Res<Time>, mut state: ResMut<State>, mut query: Query<&mut Timer>) {
    for mut timer in query.iter_mut() {
        if timer.tick(time.delta()).finished() {
            info!("Entity timer just finished");

            state.inference_state = InferenceState::Infer;
        }
    }
}
