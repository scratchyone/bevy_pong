use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;

pub struct GameStarted(pub bool);
pub struct PreGameOnly;

pub struct PrestartPlugin;

pub struct PauseFor(pub f32);

impl Plugin for PrestartPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(PauseFor(0.0))
            .add_startup_system(setup.system())
            .add_system(pause_counter.system())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_before_start.system())
                    .with_system(start_game.system())
                    .with_system(animate_text_out_system.system())
                    .with_system(animate_text_in_system.system()),
            );
    }
}

pub fn run_if_unpaused(mode: Res<GameStarted>, paused: Res<PauseFor>) -> ShouldRun {
    match *mode {
        GameStarted(true) if paused.0 == 0.0 => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}
pub fn run_if_poststart(mode: Res<GameStarted>, paused: Res<PauseFor>) -> ShouldRun {
    match *mode {
        GameStarted(true) => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}

pub fn run_before_start(mode: Res<GameStarted>) -> ShouldRun {
    match *mode {
        GameStarted(false) => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}
fn pause_counter(mut pause: ResMut<PauseFor>, time: Res<Time>) {
    if pause.0 > 0.0 {
        pause.0 -= time.delta_seconds();
        pause.0 = pause.0.max(0.0);
    }
}

fn start_game(
    mut game_started: ResMut<GameStarted>,
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<Entity, With<PreGameOnly>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        game_started.0 = true;

        for entity in &mut query.iter() {
            commands.entity(entity).despawn();
        }
    }
}
struct AnimateOutAfter(f32);
struct AnimateIn(TextAnimationMetadata);

#[derive(Clone)]
struct TextAnimationMetadata {
    start_time: f32,
    duration: f32,
    elapsed: f32,
    end_pos: f32,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: "W/S and Up/Down to control paddles".to_string(),
                            style: TextStyle {
                                font_size: 50.0,
                                color: Color::rgba(1.0, 1.0, 1.0, 0.0),
                                font: asset_server.load("FiraSans-Regular.ttf"),
                            },
                        }],
                        alignment: TextAlignment {
                            vertical: VerticalAlign::Center,
                            horizontal: HorizontalAlign::Center,
                        },
                    },
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: Rect {
                            top: Val::Px(10.0),
                            ..Default::default()
                        },
                        align_self: AlignSelf::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(PreGameOnly)
                .insert(AnimateOutAfter(2.5))
                .insert(AnimateIn(TextAnimationMetadata {
                    start_time: 0.5,
                    duration: 0.5,
                    elapsed: 0.0,
                    end_pos: 10.0,
                }));

            parent
                .spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: "Press space to start".to_string(),
                            style: TextStyle {
                                font_size: 70.0,
                                color: Color::rgba(1.0, 1.0, 1.0, 0.0),
                                font: asset_server.load("FiraSans-Regular.ttf"),
                            },
                        }],
                        alignment: TextAlignment {
                            vertical: VerticalAlign::Center,
                            horizontal: HorizontalAlign::Center,
                        },
                    },
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: Rect {
                            top: Val::Px(5.0),
                            ..Default::default()
                        },
                        align_self: AlignSelf::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(PreGameOnly)
                .insert(AnimateIn(TextAnimationMetadata {
                    start_time: 3.5,
                    duration: 0.5,
                    elapsed: 0.0,
                    end_pos: 5.0,
                }));
        })
        .insert(PreGameOnly);

    commands.insert_resource(GameStarted(false));
}

fn animate_text_out_system(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut Style, &mut AnimateOutAfter), With<PreGameOnly>>,
) {
    for (mut text, mut style, mut pause) in &mut query.iter_mut() {
        if pause.0 > 0.0 {
            pause.0 -= time.delta_seconds();
            debug!("Pausing animation for {}", pause.0);
        } else {
            pause.0 = 0.0;
            let count = text
                .sections
                .iter()
                .filter(|section| section.style.color.a() > 0.0)
                .count();
            if count > 0 {
                debug!("Text bundle has {} sections currently animating", count);
            }
            for mut section in &mut text.sections {
                section.style.color = Color::rgba(
                    section.style.color.r(),
                    section.style.color.g(),
                    section.style.color.b(),
                    f32::max(0.0, section.style.color.a() - 0.05),
                );
                style.position.top = Val::Px(
                    (match style.position.top {
                        Val::Px(x) => x + 0.5,
                        _ => 0.0,
                    } + 0.1)
                        .max(0.0),
                );
            }
        }
    }
}

fn gradient(start: f32, end: f32, elapsed: f32, duration: f32) -> f32 {
    if elapsed == 0.0 {
        start
    } else {
        start + (end - start) * (elapsed / duration)
    }
}

fn animate_text_in_system(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut Style, &mut AnimateIn), With<PreGameOnly>>,
) {
    for (mut text, mut style, mut animation) in &mut query.iter_mut() {
        animation.0.elapsed += time.delta_seconds();
        let meta = &mut animation.0;
        if meta.elapsed < meta.start_time {
            debug!("Pausing animation for {}", meta.start_time - meta.elapsed);
        } else if (meta.elapsed - meta.start_time) <= meta.duration {
            debug!(
                "Text bundle has {} sections currently animating",
                text.sections.len(),
            );
            for mut section in &mut text.sections {
                section.style.color = Color::rgba(
                    section.style.color.r(),
                    section.style.color.g(),
                    section.style.color.b(),
                    gradient(0.0, 1.0, meta.elapsed - meta.start_time, meta.duration),
                );
                style.position.top = Val::Px(gradient(
                    meta.end_pos + 13.0,
                    meta.end_pos,
                    meta.elapsed - meta.start_time,
                    meta.duration / 0.8,
                ));
            }
        }
    }
}
