use prestart::PauseFor;
use scoreboard::Scoreboard;
use wasm_bindgen::prelude::*;

use bevy::app::Events;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};
#[cfg(target_arch = "wasm32")]
use bevy_web_fullscreen::FullViewportPlugin;
mod prestart;
mod scoreboard;
const SPACING: f32 = 50.0;

struct Controls {
    up: KeyCode,
    down: KeyCode,
}

struct Paddle;
struct Ball;
enum Side {
    Left,
    Right,
}
struct Velocity(Vec2);

#[wasm_bindgen]
pub fn main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_plugin(prestart::PrestartPlugin)
        .add_plugin(scoreboard::ScoreboardPlugin);
    // when building for Web, use WebGL2 rendering
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin)
        .add_plugin(FullViewportPlugin);

    app.insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_startup_system(setup.system())
        .add_startup_system(window_resize_system.system())
        .add_system(window_resize_system.system())
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(prestart::run_if_unpaused.system())
                .with_system(ball_movement_system.system())
                .with_system(ball_collision_system.system()),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(prestart::run_if_poststart.system())
                .with_system(control_system.system()),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Res<Windows>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(30.0, 120.0)),
            ..Default::default()
        })
        .insert(Paddle)
        .insert(Controls {
            up: KeyCode::W,
            down: KeyCode::S,
        })
        .insert(Side::Left);

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(30.0, 120.0)),
            ..Default::default()
        })
        .insert(Paddle)
        .insert(Controls {
            up: KeyCode::Up,
            down: KeyCode::Down,
        })
        .insert(Side::Right);

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.9, 0.9, 0.9).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            ..Default::default()
        })
        .insert(Ball)
        .insert(Velocity(Vec2::from((0.5, 0.2)).normalize()));
}

fn window_resize_system(
    windows: Res<Windows>,
    mut query: Query<(&mut Transform, &Side, With<Paddle>)>,
) {
    let window = windows.get_primary().unwrap();
    for (mut transform, side, _) in query.iter_mut() {
        match side {
            Side::Left => {
                transform.translation.x = SPACING - window.width() as f32 / 2.0;
            }
            Side::Right => {
                transform.translation.x = window.width() as f32 / 2.0 - SPACING;
            }
        }
    }
}

fn control_system(
    time: Res<Time>,
    windows: Res<Windows>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Controls, &mut Transform, &Sprite, With<Paddle>)>,
) {
    for (controls, mut transform, sprite, _) in query.iter_mut() {
        const SPEED: f32 = 500.0;
        let direction = if keyboard_input.pressed(controls.up) {
            SPEED
        } else if keyboard_input.pressed(controls.down) {
            -SPEED
        } else {
            0.0
        };
        let translation = &mut transform.translation;
        translation.y += time.delta_seconds() * direction;
        // bound the paddle within the walls
        translation.y = translation
            .y
            .min(windows.get_primary().unwrap().height() / 2.0 - sprite.size.y / 2.0)
            .max(windows.get_primary().unwrap().height() / -2.0 + sprite.size.y / 2.0);
    }
}

fn vec2_to_vec3(v: Vec2) -> Vec3 {
    Vec3::new(v.x, v.y, 0.0)
}

fn ball_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity, With<Ball>)>,
) {
    const BALL_SPEED: f32 = 250.0;
    if let Ok((mut transform, velocity, _)) = query.single_mut() {
        let translation = &mut transform.translation;
        *translation += vec2_to_vec3(velocity.0 * time.delta_seconds() * BALL_SPEED);
    }
}
fn ball_collision_system(
    windows: Res<Windows>,
    mut query: QuerySet<(
        Query<(&mut Transform, &Sprite, &mut Velocity), With<Ball>>,
        Query<(&Transform, &Sprite), With<Paddle>>,
    )>,
    mut score: ResMut<Scoreboard>,
    mut pause: ResMut<PauseFor>,
) {
    let window = windows.get_primary().unwrap();
    let paddle_transforms = query
        .q1()
        .iter()
        .map(|(transform, _)| transform.clone())
        .collect::<Vec<_>>();
    let paddle_sprites = query
        .q1()
        .iter()
        .map(|(_, sprite)| sprite.clone())
        .collect::<Vec<_>>();
    let paddles = paddle_transforms.iter().zip(paddle_sprites.iter());
    if let Ok((mut transform, sprite, mut velocity)) = query.q0_mut().single_mut() {
        let translation = transform.translation;
        // Handle collision
        for (paddle_transform, paddle_sprite) in paddles {
            let collision = collide(
                translation,
                sprite.size,
                paddle_transform.translation,
                paddle_sprite.size,
            );

            if let Some(collision) = collision {
                match collision {
                    Collision::Left => {
                        velocity.0.x = -velocity.0.x;
                    }
                    Collision::Right => {
                        velocity.0.x = -velocity.0.x;
                    }
                    Collision::Top => {
                        velocity.0.y = -velocity.0.y;
                    }
                    Collision::Bottom => {
                        velocity.0.y = -velocity.0.y;
                    }
                }
            }
        }
        fn reset_ball(pause: &mut PauseFor, transform: &mut Transform) {
            *pause = PauseFor(1.0);
            *transform = Transform::from_xyz(0.0, 0.0, 0.0);
        }
        // Handle wall collision
        if translation.x < -window.width() as f32 / 2.0 + sprite.size.x / 2.0 {
            // Left side
            velocity.0.x = -velocity.0.x;
            score.1 += 1;
            reset_ball(&mut pause, &mut transform);
            *velocity = Velocity(Vec2::from((0.5, 0.2)).normalize());
        }
        if translation.x > window.width() as f32 / 2.0 - sprite.size.x / 2.0 {
            // Right side
            velocity.0.x = -velocity.0.x;
            score.0 += 1;
            reset_ball(&mut pause, &mut transform);
            *velocity = Velocity(Vec2::from((-0.5, -0.2)).normalize());
        }
        if translation.y < -window.height() as f32 / 2.0 + sprite.size.y / 2.0 {
            // Top
            velocity.0.y = -velocity.0.y;
        }
        if translation.y > window.height() as f32 / 2.0 - sprite.size.y / 2.0 {
            // Bottom
            velocity.0.y = -velocity.0.y;
        }
    }
}
