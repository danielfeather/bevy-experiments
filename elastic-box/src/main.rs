use bevy::{
    color::palettes::css::{BLUE, GREEN, MAGENTA, RED, WHITE},
    log::{Level, LogPlugin},
    prelude::*,
    sprite::Anchor,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::{
    debug::DebugPickingMode,
    events::{Drag, DragEnter, DragLeave, Pointer},
    prelude::{On, Pickable},
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
            WorldInspectorPlugin::default(),
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
            ElasticBox,
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
                parent.spawn((
                    Card,
                    PickableBundle {
                        pickable: Pickable::IGNORE,
                        ..default()
                    },
                    SpriteBundle {
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
                    },
                ));
            }
        });

    commands.spawn((
        Card,
        PickableBundle {
            pickable: Pickable {
                should_block_lower: false,
                ..default()
            },
            ..default()
        },
        SpriteBundle {
            sprite: Sprite {
                color: Color::Srgba(MAGENTA),
                custom_size: Some(CARD_SIZE),
                ..default()
            },
            transform: Transform::from_xyz(
                0.0,
                -calculate_box_size(3_usize, &CARD_SIZE, PADDING).y,
                2.0,
            ),
            ..default()
        },
        On::<Pointer<Drag>>::target_component_mut::<Transform>(|e, transform| {
            transform.translation.x = transform.translation.x + e.delta.x;
            transform.translation.y = transform.translation.y - e.delta.y;
        }),
    ));
}

fn adjust_container(
    elastibox: Query<&Children, With<ElasticBox>>,
    mut overs: EventReader<Pointer<DragEnter>>,
    mut outs: EventReader<Pointer<DragLeave>>,
    mut drags: EventReader<Pointer<Drag>>,
    mut midpoints: Local<Vec<f32>>,
    mut current_child: Local<usize>,
    transforms: Query<&GlobalTransform>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut transform: Query<&mut Transform>,
) {
    let (camera, camera_transform) = camera.single();

    for over in overs.read() {
        let Ok(elastiboxes) = elastibox.get(over.target) else {
            continue;
        };

        *midpoints = elastiboxes
            .iter()
            .map(|child| {
                let Ok(transform) = transforms.get(*child) else {
                    return 0.0;
                };

                transform.translation().x + (CARD_SIZE.x / 2.0)
            })
            .collect();

        debug!("MidPoints: {:#?}", midpoints);

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
        let Some(vec2) =
            camera.viewport_to_world_2d(camera_transform, drag.pointer_location.position)
        else {
            continue;
        };

        let mut index = midpoints.len();

        debug!("Cursor Pos: {}", vec2);

        for (i, pos) in midpoints.iter().enumerate() {
            if vec2.x < *pos {
                index = i;
                break;
            }
        }

        if *current_child == index {
            continue;
        }

        // Reorder children
        // Get all the transforms
    }
}

fn reorder_child_transforms() {}
