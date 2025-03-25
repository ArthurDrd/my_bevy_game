use std::time::Duration;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy::window::WindowResolution;
use rand::random;

const SNAKE_HEAD_COLOR: Color = Color::srgb(0.7, 0.7, 0.7);
const FOOD_COLOR: Color = Color::srgb(1.0, 0.0, 1.0); 

const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;

const FOOD_SPAWN_TIMER: u64 = 1;

#[derive(Component)]
struct SnakeHead;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}
impl Size {
    pub fn square(x: f32) -> Self {
        Size {
            width: x,
            height: x,
        }
    }
}

#[derive(Component)]
struct Food;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.04)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snake!".to_string(),
                resolution: WindowResolution::new(500., 500.),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, snake_movement)
        .add_systems(Update,food_spawner.run_if(on_timer(Duration::from_secs(FOOD_SPAWN_TIMER))))
        .add_systems(PostUpdate, (size_scaling, position_translation))
        .run();
}

fn setup(
    mut  commands: Commands
) {
    // camera 2D
    commands.spawn(Camera2d);
    spawn_snake(commands);
}

fn spawn_snake(
    mut commands: Commands
) {
    commands
        .spawn((
            Sprite {
                color: SNAKE_HEAD_COLOR,
                ..default() },
            Transform {
                scale: Vec3::new(10.0, 10.0, 1.0),
                ..default()
            },
        ))
        .insert(SnakeHead)
        .insert(Position { x: 3, y: 3 })
        .insert(Size::square(0.8));
}

fn snake_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut head_positions: Query<&mut Position, With<SnakeHead>>
) {
    for mut pos in head_positions.iter_mut() {
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            pos.x -= 1;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            pos.x += 1;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            pos.y -= 1;
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            pos.y += 1;
        }
    }
}

fn size_scaling(
    windows: Query<&Window>,
    mut entity: Query<(&Size, &mut Transform)>
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    
    for (sprite_size, mut transform) in entity.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width(),
            sprite_size.height / ARENA_HEIGHT as f32 * window.height(),
            1.0,
        );
    }
}

fn position_translation(
    windows: Query<&Window>,
    mut entity: Query<(&Position, &mut Transform)>
) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    for (pos, mut transform) in entity.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width(), ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height(), ARENA_HEIGHT as f32),
            0.0,
        );
    }
}

fn food_spawner(
    mut commands: Commands
)  {
    commands
        .spawn((
            Sprite {
                color: FOOD_COLOR,
                ..default() },
        ))
        .insert(Food)
        .insert(Position {
            x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
            y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
        })
        .insert(Size::square(0.8));
}