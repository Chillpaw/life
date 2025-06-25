use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::collections::HashSet;

const CELL_COLOR_DEAD: Color = Color::srgb(0.2, 0.2, 0.2);
const CELL_COLOR_ALIVE: Color = Color::srgb(1.0, 1.0, 1.0);
const BACKGROUND_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);
const CELL_SIZE: Vec2 = Vec2::splat(10.0);
const CELL_PADDING: isize = 15;
const GRID_SIZE: isize = 10;

#[derive(Component, PartialEq, Eq, Debug, Hash, Copy, Clone)]
struct Position {
    x: isize,
    y: isize,
}

#[derive(Component)]
struct Alive;

#[derive(Component)]
struct NextAlive;

#[derive(Resource)]
struct Paused(bool);

#[derive(Component)]
struct MainCamera;

#[derive(Resource)]
struct CursorWorldPos(Option<Vec2>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Paused(true))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(CursorWorldPos(None))
        .insert_resource(Time::<Fixed>::from_seconds(0.25))
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                simulate.run_if(not(paused)),
                apply_next_state.run_if(not(paused)),
                clear_next_state.run_if(not(paused)),
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                update_cell_color,
                toggle_pause,
                (get_cursor_world_pos, handle_cell_click).chain(),
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
    build_grid(commands, GRID_SIZE);
}

fn paused(paused: Res<Paused>) -> bool {
    paused.0
}

fn not(f: impl Fn(Res<Paused>) -> bool + Copy) -> impl Fn(Res<Paused>) -> bool + Copy {
    move |r| !f(r)
}

fn toggle_pause(mut paused: ResMut<Paused>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) {
        paused.0 = !paused.0;
        println!("Paused: {}", paused.0);
    }
}

fn build_grid(mut commands: Commands, size: isize) {
    for cell_x in -size..size {
        for cell_y in -size..size {
            let pos_x = (cell_x * CELL_PADDING) as f32;
            let pos_y = (cell_y * CELL_PADDING) as f32;

            commands.spawn((
                Sprite::from_color(CELL_COLOR_DEAD, CELL_SIZE),
                Transform {
                    translation: Vec3::new(pos_x, pos_y, 0.0),
                    ..default()
                },
                Position {
                    x: cell_x,
                    y: cell_y,
                },
            ));
        }
    }
}

fn assign_sample_lives(mut commands: Commands, query: Query<(Entity, &Position)>) {
    let sample_array = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 1, 1, 1, 0, 0, 0, 0, 0],
        [0, 0, 1, 0, 1, 0, 0, 0, 0, 0],
        [0, 0, 1, 1, 1, 0, 0, 0, 0, 0],
        [0, 0, 1, 1, 1, 0, 0, 0, 0, 0],
        [0, 0, 1, 1, 1, 0, 0, 0, 0, 0],
        [0, 0, 1, 1, 1, 0, 0, 0, 0, 0],
        [0, 0, 1, 0, 1, 0, 0, 0, 0, 0],
        [0, 0, 1, 1, 1, 0, 0, 0, 0, 0],
    ];

    for (y, row) in sample_array.iter().enumerate() {
        for (x, &value) in row.iter().enumerate() {
            if value == 1 {
                for (entity, pos) in query.iter() {
                    if pos.x == x as isize && pos.y == y as isize {
                        commands.entity(entity).insert(Alive);
                        break;
                    }
                }
            }
        }
    }
}

fn update_cell_color(mut query: Query<(&mut Sprite, &Position, Option<&Alive>)>) {
    for (mut cell, _pos, alive) in query.iter_mut() {
        if let Some(_) = alive {
            cell.color = CELL_COLOR_ALIVE;
        } else {
            cell.color = CELL_COLOR_DEAD;
        }
    }
}

fn simulate(mut commands: Commands, query: Query<(Entity, &Position, Option<&Alive>)>) {
    let alive_positions: HashSet<Position> = query
        .iter()
        .filter_map(|(_, pos, alive)| alive.map(|_| *pos))
        .collect();

    for (cell, pos, alive) in query.iter() {
        let mut neighbours = 0;

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let neighbour_x = pos.x as isize + dx;
                let neighbour_y = pos.y as isize + dy;

                let neighbour_pos = Position {
                    x: neighbour_x,
                    y: neighbour_y,
                };

                if alive_positions.contains(&neighbour_pos) {
                    neighbours += 1;
                }
            }
        }

        if let Some(_) = alive {
            if neighbours < 2 || neighbours > 3 {
                commands.entity(cell).remove::<NextAlive>();
                println!(
                    "Changed cell: {:?} to dead because alive with {} neighbours.",
                    pos, neighbours
                );
            } else {
                commands.entity(cell).insert(NextAlive);
                println!(
                    "Kept cell: {:?} alive because {} neighbours",
                    pos, neighbours
                );
            }
        } else if neighbours == 3 {
            commands.entity(cell).insert(NextAlive);
            println!("Changed cell: {:?} to dead.", pos);
        }
    }
}

fn apply_next_state(
    mut commands: Commands,
    query: Query<(Entity, Option<&Alive>, Option<&NextAlive>)>,
) {
    for (cell, alive, next_alive) in query.iter() {
        match (alive.is_some(), next_alive.is_some()) {
            (false, true) => {
                commands.entity(cell).insert(Alive);
                //println!("Changed cell: {} to alive", cell);
            }
            (true, false) => {
                commands.entity(cell).remove::<Alive>();
                //println!("Changed cell: {} to not alive", cell);
            }
            _ => {}
        }
    }
}

fn clear_next_state(mut commands: Commands, query: Query<Entity, With<NextAlive>>) {
    for cell in query.iter() {
        commands.entity(cell).remove::<NextAlive>();
    }
}

fn get_cursor_world_pos(
    mut cursor_world_pos: ResMut<CursorWorldPos>,
    primary_window: Single<&Window, With<PrimaryWindow>>,
    q_camera: Single<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = *q_camera;

    cursor_world_pos.0 = primary_window.cursor_position().and_then(|cursor_pos| {
        camera
            .viewport_to_world_2d(camera_transform, cursor_pos)
            .ok()
    });
}

fn handle_cell_click(
    cursor_world_pos: Res<CursorWorldPos>,
    query: Query<(Entity, &Position, Option<&Alive>)>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor_pos) = cursor_world_pos.0 else {
        return;
    };

    let grid_x = (cursor_pos.x / CELL_PADDING as f32).floor() as isize;
    let grid_y = (cursor_pos.y / CELL_PADDING as f32).floor() as isize;

    for (cell, pos, alive) in query.iter() {
        if pos.x == grid_x && pos.y == grid_y {
            if alive.is_some() {
                commands.entity(cell).remove::<Alive>();
            } else {
                commands.entity(cell).insert(Alive);
            }
            break;
        }
    }
}
