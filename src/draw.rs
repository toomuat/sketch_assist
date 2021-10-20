use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    prelude::*,
    window::CursorMoved,
};

pub fn line_drawing_system(mouse_button_input: Res<Input<MouseButton>>) {
    if mouse_button_input.pressed(MouseButton::Left) {
        info!("left mouse currently pressed");
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        info!("left mouse just pressed");
    }
}

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
