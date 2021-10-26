use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    prelude::*,
    window::CursorMoved,
};
use bevy_rapier2d::{physics::RigidBodyBundle, prelude::*};
use itertools::Itertools;
use std::collections::VecDeque;

pub struct MouseCoord {
    pub mouse_coord: VecDeque<Vec2>,
    pub camera_entity: Entity,
}

#[derive(Default)]
pub struct LineMaterial(pub Handle<ColorMaterial>);

pub fn line_drawing_system(
    mut commands: Commands,
    mut state: ResMut<MouseCoord>,
    line_material: Res<LineMaterial>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: Res<Windows>,
    transforms: Query<&Transform>,
) {
    let camera_transform = transforms.get(state.camera_entity).unwrap();

    if mouse_button_input.pressed(MouseButton::Left) {
        for event in cursor_moved_events.iter() {
            // state.mouse_coord.push_front(event.position);
            state.mouse_coord.push_front(screen_to_world(
                event.position,
                &camera_transform,
                &windows,
            ));
        }
    } else {
        state.mouse_coord.clear();
    }

    let new_line_segments = state.pop_line_segments();

    // Get two x, y coordinate pair of mouse transition
    // and connect them to create a line.
    for (p1, p2) in new_line_segments.into_iter() {
        spawn_line_segment(p1, p2, line_material.0.clone(), &mut commands, &windows);
    }
}

const SEGMENT_LENGTH: f32 = 15.0;

impl MouseCoord {
    fn pop_line_segments(&mut self) -> Vec<(Vec2, Vec2)> {
        // Downsample the cursor curve by length.
        let mut line_segments = Vec::new();
        let mut segment_start = if let Some(back) = self.mouse_coord.back() {
            *back
        } else {
            return line_segments;
        };

        let mut curve_length = 0.0;
        let mut segment_points = 0;
        let mut confirmed_segment_points = 0;
        for (p1, p2) in self.mouse_coord.iter().rev().tuple_windows() {
            segment_points += 1;

            let diff = *p2 - *p1;
            curve_length += diff.length();
            if curve_length >= SEGMENT_LENGTH {
                if segment_start != *p2 {
                    line_segments.push((segment_start, *p2));
                }
                segment_start = *p2;
                confirmed_segment_points += segment_points;
                curve_length = 0.0;
                segment_points = 0;
            }
        }

        // Remove the points belonging to the segments we've gathered.
        self.mouse_coord
            .truncate(self.mouse_coord.len() - confirmed_segment_points);

        line_segments
    }
}

const LINE_THICKNESS: f32 = 3.0;

fn spawn_line_segment(
    mut p1: Vec2,
    mut p2: Vec2,
    material: Handle<ColorMaterial>,
    commands: &mut Commands,
    windows: &Windows,
) {
    let window = windows.get_primary().unwrap();
    let (width, height) = (window.width(), window.height());
    let a = height / 14.0;
    let canvas_width = (width - a * 3.0) / 2.0;
    let canvas_height = height - a * 2.0;
    let left_down = Vec2::new(-width / 2.0, -height / 2.0);
    let crop_left = left_down.x + a;
    let crop_down = left_down.y + a;
    let crop_right = left_down.x + a + canvas_width;
    let crop_up = left_down.y + a + canvas_height;

    if p1.x <= crop_left {
        p1.x = crop_left;
    }
    if p2.x <= crop_left {
        p2.x = crop_left;
    }
    if p1.x >= crop_right {
        p1.x = crop_right;
    }
    if p2.x >= crop_right {
        p2.x = crop_right;
    }
    if p1.y <= crop_down {
        p1.y = crop_down;
    }
    if p2.y <= crop_down {
        p2.y = crop_down;
    }
    if p1.y >= crop_up {
        p1.y = crop_up;
    }
    if p2.y >= crop_up {
        p2.y = crop_up;
    }

    let midpoint = (p1 + p2) / 2.0;
    let diff = p2 - p1;
    let length = diff.length();
    let angle = Vec2::new(1.0, 0.0).angle_between(diff);
    let x = midpoint.x;
    let y = midpoint.y;

    commands
        .spawn_bundle(SpriteBundle {
            material,
            sprite: Sprite {
                size: Vec2::new(length, LINE_THICKNESS),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle(RigidBodyBundle {
            body_type: RigidBodyType::Static,
            position: (Vec2::new(x, y), angle).into(),
            ..Default::default()
        })
        .insert(RigidBodyPositionSync::Discrete);
}

pub fn create_canvas(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Res<Windows>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let window = windows.get_primary().unwrap();
    let (width, height) = (window.width(), window.height());

    let a = height / 14.0;

    let canvas_width = (width - a * 3.0) / 2.0;
    let canvas_height = height - a * 2.0;

    // Area of sketch canvas
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(canvas_width, canvas_height)),
        material: materials.add(Color::WHITE.into()),
        transform: Transform {
            translation: Vec3::new(-(width / 2.0 - canvas_width / 2.0 - a), 0., 0.),
            ..Default::default()
        },
        ..Default::default()
    });

    // Area to show images
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::new(canvas_width, canvas_height)),
        material: materials.add(Color::WHITE.into()),
        transform: Transform {
            translation: Vec3::new(width / 2.0 - canvas_width / 2.0 - a, 0., 0.),
            ..Default::default()
        },
        ..Default::default()
    });
}

pub fn screen_to_world(p: Vec2, camera_transform: &Transform, windows: &Windows) -> Vec2 {
    let w = windows.get_primary().unwrap();
    let resolution = Vec2::new(w.width() as f32, w.height() as f32);
    let p_ndc = p - resolution / 2.0;
    let p_world = *camera_transform * p_ndc.extend(0.0);

    p_world.truncate()
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
