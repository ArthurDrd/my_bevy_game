use std::time::Duration;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy::window::WindowResolution;
use rand::random;

const SNAKE_HEAD_COLOR: Color = Color::srgb(0.7, 0.7, 0.7);
const SNAKE_SEGMENT_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
const FOOD_COLOR: Color = Color::srgb(1.0, 0.0, 1.0);

const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;

const FOOD_SPAWN_TIMER: u64 = 1;
const MOVEMENT_TIMER: u64 = 200;

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

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

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}
impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}


#[derive(Component)]
struct SnakeSegment;

#[derive(Resource, Default)]
struct SnakeSegments(Vec<Entity>);

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
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, spawn_snake)
        .add_systems(FixedUpdate, snake_movement.run_if(on_timer(Duration::from_millis(MOVEMENT_TIMER))))
        .add_systems(FixedUpdate,food_spawner.run_if(on_timer(Duration::from_secs(FOOD_SPAWN_TIMER))))
        .add_systems(FixedUpdate, snake_movement_input.before(snake_movement))
        .add_systems(PostUpdate, (size_scaling, position_translation))
        .insert_resource(SnakeSegments::default())
        .run();
}

fn setup_camera(
    mut  commands: Commands
) {
    commands.spawn(Camera2d);
}

fn spawn_snake(
    mut commands: Commands,
    mut segments: ResMut<SnakeSegments>
) {
    *segments = SnakeSegments(vec![
    commands
        .spawn((
            Sprite {
                color: SNAKE_HEAD_COLOR,
                ..default() },
        ))
        .insert(SnakeHead {
            direction: Direction::Up
        })
        .insert(SnakeSegment)
        .insert(Position { x: 3, y: 3 })
        .insert(Size::square(0.8))
        .id(), 
    spawn_segment(commands, Position { x: 3, y: 2 }), 
    ]);
}

fn snake_movement_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut heads: Query<&mut SnakeHead>
) {
    if let Some(mut head) = heads.iter_mut().next() {
        let dir: Direction = if keyboard_input.pressed(KeyCode::ArrowLeft) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::ArrowUp) {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::ArrowRight) {
            Direction::Right
        } else {
            head.direction
        };
        if dir != head.direction.opposite() {
            head.direction = dir;
        }
    }
}

fn snake_movement(mut heads: Query<(&mut Position, &SnakeHead)>) {
    if let Some((mut head_pos, head)) = heads.iter_mut().next() {
        match &head.direction {
            Direction::Left => {
                head_pos.x -= 1;
            }
            Direction::Right => {
                head_pos.x += 1;
            }
            Direction::Up => {
                head_pos.y += 1;
            }
            Direction::Down => {
                head_pos.y -= 1;
            }
        };
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

fn spawn_segment(
    mut commands: Commands,
    position: Position
) -> Entity {
    commands
        .spawn((
            Sprite {
                color: SNAKE_SEGMENT_COLOR,
                ..default() 
            },
        ))
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.65))
        .id()
}