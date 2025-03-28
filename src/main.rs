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
const MOVEMENT_TIMER: u64 = 150;

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

#[derive(Event)]
struct GrowthEvent;

#[derive(Resource, Default)]
struct LastTailPosition(Option<Position>);

#[derive(Event)]
struct GameOverEvent;

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
        .add_systems(FixedUpdate, (snake_movement.run_if(on_timer(Duration::from_millis(MOVEMENT_TIMER))), snake_eating.after(snake_movement)))
        .add_systems(FixedUpdate,food_spawner.run_if(on_timer(Duration::from_secs(FOOD_SPAWN_TIMER))))
        .add_systems(FixedUpdate, snake_movement_input.before(snake_movement))
        .add_systems(PostUpdate, (size_scaling, position_translation))
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPosition::default())
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        .add_systems(Update, snake_growth.after(snake_eating))
        .add_systems(Update, game_over.after(snake_movement))
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

fn snake_movement(
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Position>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_writer: EventWriter<GameOverEvent>,
) {
    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .0.iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();
        let mut head_pos = positions.get_mut(head_entity).unwrap();
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
        
        if head_pos.x < 0
            || head_pos.y < 0
            || head_pos.x as u32 >= ARENA_WIDTH
            || head_pos.y as u32 >= ARENA_HEIGHT
        {
            game_over_writer.send(GameOverEvent);
        }

        if segment_positions.contains(&head_pos) {
            game_over_writer.send(GameOverEvent);
        }
        
        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });
        *last_tail_position = LastTailPosition(Some(*segment_positions.last().unwrap()));
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

fn snake_eating(
    mut commands: Commands,
    mut growth_writer : EventWriter<GrowthEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
) {
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.entity(ent).despawn();
                growth_writer.send(GrowthEvent);
            }
        }
    }
}

fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
) {
    if growth_reader.read().next().is_some() {
        segments.0.push(spawn_segment(commands, last_tail_position.0.unwrap()));
    }
}

fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    segments_res: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>,
) {
    if reader.read().next().is_some() {
        for ent in food.iter().chain(segments.iter()) {
            commands.entity(ent).despawn();
        }
        spawn_snake(commands, segments_res);
    }
}