use bevy::prelude::*;

mod draw;

use draw::{line_drawing_system, print_mouse_events_system};

fn main() {
    let mut window_desc = WindowDescriptor::default();
    window_desc.width = 1300.0;
    window_desc.height = 600.0;
    window_desc.title = "Sketch Assist".to_string();

    App::new()
        .insert_resource(window_desc)
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.7, 0.7, 0.7)))
        // .add_startup_system(add_people.system())
        // .add_system(hello_world.system())
        // .add_system(greet_people.system())
        // .add_system(line_drawing_system.system())
        .add_system(print_mouse_events_system)
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}

#[allow(dead_code)]
fn hello_world() {
    println!("hello world!");
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

#[allow(dead_code)]
fn add_people(mut commands: Commands) {
    commands
        .spawn()
        .insert(Person)
        .insert(Name("Elaina Proctor".to_string()));
    commands
        .spawn()
        .insert(Person)
        .insert(Name("Renzo Hume".to_string()));
    commands
        .spawn()
        .insert(Person)
        .insert(Name("Zayna Nieves".to_string()));
}

#[allow(dead_code)]
fn greet_people(query: Query<&Name, With<Person>>) {
    for name in query.iter() {
        println!("hello {}!", name.0);
    }
}
