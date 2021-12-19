use crate::prestart::run_if_unpaused;
use bevy::prelude::*;
pub struct ScoreboardPlugin;

#[derive(Debug, Clone)]
pub struct Scoreboard(pub u8, pub u8);

impl Plugin for ScoreboardPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Scoreboard(0, 0))
            .add_startup_system(setup.system())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_unpaused.system())
                    .with_system(update_scoreboard.system()),
            );
    }
}

struct ScoreKeeper;

fn generate_score(score: &Scoreboard) -> String {
    format!("{} {}", score.0, score.1)
}

fn update_scoreboard(score: Res<Scoreboard>, mut query: Query<&mut Text, With<ScoreKeeper>>) {
    for mut text in &mut query.iter_mut() {
        text.sections[0].value = generate_score(&score);
        text.sections[0].style.color.set_a(1.0);
    }
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
                            value: generate_score(&Scoreboard(0, 0)),
                            style: TextStyle {
                                font_size: 90.0,
                                color: Color::rgba(1.0, 1.0, 1.0, 0.0),
                                font: asset_server.load("PressStart2P-Regular.ttf"),
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
                            top: Val::Px(18.0),
                            ..Default::default()
                        },
                        align_self: AlignSelf::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(ScoreKeeper);
        });
}
