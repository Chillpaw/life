use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::prelude::*;
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Paused(true))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
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
        .add_systems(Update, (update_cell_color, toggle_pause, handle_cell_click))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
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

fn handle_cell_click(
    mut commands: Commands,
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<(Entity, &Transform, &Position, Option<&Alive>)>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    if let Some(cursor_pos) = window
        .cursor_position()
        .and_then(|pos| camera.viewport_to_world(camera_transform, pos))
        .map(|ray| ray.origin.truncate())
    {
        for (entity, transform, _pos, alive) in query.iter_mut() {
            let cell_pos = transform.translation.truncate();
            let half_size = CELL_SIZE / 2.0;

            let in_bounds = cursor_pos.x >= cell_pos.x - half_size.x
                && cursor_pos.x <= cell_pos.x + half_size.x
                && cursor_pos.y >= cell_pos.y - half_size.y
                && cursor_pos.y <= cell_pos.y + half_size.y;
        }
    }
}
