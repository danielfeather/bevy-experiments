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
    pointer::InputMove,
    prelude::{On, Pickable},
    DefaultPickingPlugins, PickableBundle,
};
use wasm_bindgen::prelude::*;

const PADDING: f32 = 35.0;
const CARD_SIZE: Vec2 = Vec2::new(100., 160.);

#[wasm_bindgen]
pub fn start() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(LogPlugin {
                    level: Level::DEBUG,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
            DefaultPickingPlugins,
            WorldInspectorPlugin::default(),
        ))
        .add_event::<ReorderChildren>()
        .insert_resource(DebugPickingMode::Normal)
        .add_systems(Startup, (spawn_camera, spawn_boxes))
        .add_systems(
            Update,
            (adjust_container, reorder_child_transforms, log_input_move),
        )
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

#[derive(Debug, Event)]
/// Triggers a reorder on children of the specified entity
struct ReorderChildren(Entity, Option<usize>);

fn adjust_container(
    mut elastibox: Query<(&Children, &mut Sprite), With<ElasticBox>>,
    mut overs: EventReader<Pointer<DragEnter>>,
    mut outs: EventReader<Pointer<DragLeave>>,
    mut drags: EventReader<Pointer<Drag>>,
    mut midpoints: Local<Vec<f32>>,
    mut current_elastibox: Local<Option<Entity>>,
    mut current_child: Local<usize>,
    global_transforms: Query<&GlobalTransform>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut reorder_children: EventWriter<ReorderChildren>,
) {
    let (camera, camera_transform) = camera.single();

    for over in overs.read() {
        let Ok((children, mut sprite)) = elastibox.get_mut(over.target) else {
            continue;
        };

        sprite.custom_size = Some(calculate_box_size(children.len() + 1, &CARD_SIZE, PADDING));

        *current_elastibox = Some(over.target);

        *midpoints = children
            .iter()
            .map(|child| {
                let Ok(transform) = global_transforms.get(*child) else {
                    return 0.0;
                };

                transform.translation().x + (CARD_SIZE.x / 2.0)
            })
            .collect();
    }

    for out in outs.read() {
        let Ok((children, mut sprite)) = elastibox.get_mut(out.target) else {
            continue;
        };

        sprite.custom_size = Some(calculate_box_size(children.len(), &CARD_SIZE, PADDING));

        reorder_children.send(ReorderChildren(out.target, None));

        *current_elastibox = None;
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

        for (i, pos) in midpoints.iter().enumerate() {
            if vec2.x < *pos {
                index = i;
                break;
            }
        }

        if *current_child == index {
            continue;
        }

        let Some(elastibox) = *current_elastibox else {
            continue;
        };

        *current_child = index;

        debug!("Reorder");

        reorder_children.send(ReorderChildren(elastibox, Some(index)));
    }
}

fn reorder_child_transforms(
    mut reorders: EventReader<ReorderChildren>,
    mut elastibox_children: Query<(&Children, &mut Sprite), With<ElasticBox>>,
    mut transforms: Query<&mut Transform>,
) {
    for ReorderChildren(elastibox, child_index) in reorders.read() {
        let Ok((children, ..)) = elastibox_children.get_mut(*elastibox) else {
            continue;
        };

        for (i, child) in children.iter().enumerate() {
            let new_translation = if let None = child_index {
                calculate_child_translation(children.len(), i)
            } else if child_index.is_some() && &i >= &child_index.unwrap() {
                calculate_child_translation(children.len() + 1, i + 1)
            } else {
                calculate_child_translation(children.len() + 1, i)
            };

            let Ok(mut child_transform) = transforms.get_mut(*child) else {
                continue;
            };

            child_transform.translation.x = new_translation.x;
        }
    }
}

fn calculate_child_translation(length: usize, index: usize) -> Vec2 {
    PADDING + ((CARD_SIZE.x + PADDING) * index as f32)
        - (calculate_box_size(length, &CARD_SIZE, PADDING) / 2.0)
}

fn log_input_move(mut moves: EventReader<InputMove>) {
    for moveEvent in moves.read() {
        debug!("Input Move: {:?}", moveEvent);
    }
}
