use bevy::{
    color::palettes::css::{BLUE, GREEN, RED, WHITE},
    log::{Level, LogPlugin},
    prelude::*,
    sprite::Anchor,
};
use bevy_mod_picking::{
    debug::DebugPickingMode,
    events::{Drag, DragEnter, DragLeave, Pointer},
    DefaultPickingPlugins, PickableBundle,
};

const PADDING: f32 = 35.0;
const CARD_SIZE: Vec2 = Vec2::new(100., 160.);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                level: Level::DEBUG,
                ..default()
            }),
            DefaultPickingPlugins,
        ))
        .insert_resource(DebugPickingMode::Normal)
        .add_systems(Startup, (spawn_camera, spawn_boxes))
        .add_systems(Update, adjust_container)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

/// Calculates the size of the box considering the number of items and padding
fn calculate_box_size(items: usize, item_size: &Vec2, padding: f32) -> Vec2 {
    let x = (item_size.x + padding) * items as f32 + padding;
    let y = item_size.y + (padding * 2.);

    Vec2::new(x, y)
}

const COLORS: &'static [Color] = &[Color::Srgba(RED), Color::Srgba(GREEN), Color::Srgba(BLUE)];

#[derive(Debug, Component)]
struct ElasticBox;

#[derive(Debug, Component)]
struct Card;

fn spawn_boxes(mut commands: Commands) {
    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::Srgba(WHITE),
                    custom_size: Some(calculate_box_size(3_usize, &CARD_SIZE, PADDING)),
                    ..default()
                },
                ..default()
            },
            PickableBundle::default(),
        ))
        .with_children(|parent| {
            for i in 0..3 {
                parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: COLORS[i],
                        custom_size: Some(CARD_SIZE),
                        anchor: Anchor::CenterLeft,
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        PADDING + ((CARD_SIZE.x + PADDING) * i as f32)
                            - (calculate_box_size(3_usize, &CARD_SIZE, PADDING).x / 2.0),
                        0.0,
                        1.0,
                    ),
                    ..default()
                });
            }
        });
}

fn adjust_container(
    elastibox: Query<&Children, With<ElasticBox>>,
    mut overs: EventReader<Pointer<DragEnter>>,
    mut outs: EventReader<Pointer<DragLeave>>,
    mut drags: EventReader<Pointer<Drag>>,
    mut midpoints: Local<Vec<Vec2>>,
    transforms: Query<&Transform>,
) {
    for over in overs.read() {
        let Ok(elastiboxes) = elastibox.get(over.target) else {
            continue;
        };

        *midpoints = elastiboxes
            .iter()
            .map(|child| {
                let Ok(transform) = transforms.get(*child) else {
                    return Vec2::ZERO;
                };

                Vec2::new(transform.translation.x, transform.translation.y)
            })
            .collect();

        // midpoints(value)
    }

    for out in outs.read() {
        let Ok(_) = elastibox.get(out.target) else {
            continue;
        };

        midpoints.drain(0..);
    }

    if midpoints.is_empty() {
        return;
    }

    for drag in drags.read() {
        // When a drag over happens, need to loop through all the children and find the closest midpoint and then the index of
        // that midpoint
        debug!("{:#?}", drag);
    }
}
