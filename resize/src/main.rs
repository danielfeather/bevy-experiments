use bevy::{
    color::palettes::{css::WHITE, tailwind::SKY_400},
    log::{Level, LogPlugin},
    math::AspectRatio,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::prelude::*;
use bevy_web_asset::WebAssetPlugin;

const RESIZE_ANCHOR_AREA: f32 = 10.;

#[derive(Debug, Component, Reflect)]
struct Size {
    pub dimensions: UVec2,
    pub aspect_ratio: AspectRatio,
}

#[derive(Debug, Component, Reflect)]
struct Selected;

#[derive(Debug, Component, Reflect)]
struct SelectionHandles;

#[derive(Debug, Component, Reflect)]
struct SelectionHandle;

fn main() {
    App::new()
        .add_plugins((WebAssetPlugin::default(), DefaultPickingPlugins))
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    level: Level::DEBUG,
                    ..default()
                }),
            WorldInspectorPlugin::default(),
        ))
        .register_type::<Size>()
        .register_type::<Selected>()
        .insert_resource(DebugPickingMode::Normal)
        .add_systems(Startup, spawn_image)
        .add_systems(
            Update,
            (
                on_load,
                // handle_deselection,
                show_box_on_selection,
                move_on_drag,
                handle_resize,
            ),
        )
        .run();
}

fn spawn_image(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load::<Image>("forest.webp"),
            ..default()
        },
        PickableBundle::default(),
        On::<Pointer<Click>>::target_commands_mut(|_, commands| {
            commands.insert(Selected);
        }),
    ));
}

fn move_on_drag(
    mut drags: EventReader<Pointer<Drag>>,
    mut handles: Query<&mut Transform, Or<(With<SelectionHandles>, With<Selected>)>>,
) {
    for drag in drags.read() {
        if handles.get(drag.target).is_err() {
            continue;
        }

        for mut transform in handles.iter_mut() {
            transform.translation.x = transform.translation.x + drag.delta.x;
            transform.translation.y = transform.translation.y - drag.delta.y;
        }
    }
}

/// Set the size and aspect ratio of the component
fn on_load(
    mut reader: EventReader<AssetEvent<Image>>,
    handles: Query<(Entity, &Handle<Image>)>,
    mut commands: Commands,
    assets: Res<Assets<Image>>,
) {
    for asset_event in reader.read() {
        for (entity, handle) in handles.iter() {
            if !asset_event.is_loaded_with_dependencies(handle.id()) {
                continue;
            }

            let Some(image) = assets.get(handle.id()) else {
                continue;
            };

            let size = Size {
                dimensions: image.size(),
                aspect_ratio: AspectRatio::from(image.size_f32()),
            };

            commands.entity(entity).insert(size);
        }
    }
}

fn show_box_on_selection(
    selected: Query<(&Transform, &Size), Added<Selected>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (transform, size) in selected.iter() {
        let rect = Rect::from_center_size(
            Vec2::new(transform.translation.x, transform.translation.y),
            size.dimensions.as_vec2(),
        );

        let corners: &[Vec2; 4] = &[
            rect.min + Vec2::new(0_f32, rect.height()),
            rect.max,
            rect.max - Vec2::new(0_f32, rect.height()),
            rect.min,
        ];

        let sides: &[Rectangle; 4] = &[
            Rectangle::new(corners[0].distance(corners[1]), 5.0),
            Rectangle::new(5.0, corners[1].distance(corners[2])),
            Rectangle::new(corners[2].distance(corners[3]), 5.0),
            Rectangle::new(5.0, corners[3].distance(corners[0])),
        ];

        let background =
            Mesh2dHandle(meshes.add(Ellipse::new(RESIZE_ANCHOR_AREA, RESIZE_ANCHOR_AREA)));
        let foreground = Mesh2dHandle(meshes.add(Ellipse::new(
            RESIZE_ANCHOR_AREA / 1.3,
            RESIZE_ANCHOR_AREA / 1.3,
        )));

        let material_sky_blue = materials.add(Color::Srgba(SKY_400));
        let material_white = materials.add(Color::Srgba(WHITE));

        commands
            .spawn((SpatialBundle::default(), SelectionHandles))
            .with_children(|parent| {
                for (i, _) in corners.iter().enumerate() {
                    // let rect = Mesh2dHandle(meshes.add(sides[i]));

                    let corner = if (i + 1) < corners.len() {
                        corners[i + 1]
                    } else {
                        corners[0]
                    };

                    let current = corners[i];

                    // let midpoint = if (i + 1) < corners.len() {
                    //     current.midpoint(corners[i + 1])
                    // } else {
                    //     current.midpoint(corners[0])
                    // };

                    // parent.spawn(MaterialMesh2dBundle {
                    //     mesh: rect,
                    //     material: material_sky_blue.clone(),
                    //     transform: Transform::from_xyz(midpoint.x, midpoint.y, i as f32 + 1.0),
                    //     ..default()
                    // });

                    parent.spawn((
                        MaterialMesh2dBundle {
                            mesh: background.clone(),
                            material: material_sky_blue.clone(),
                            transform: Transform::from_xyz(corner.x, corner.y, i as f32 + 2.0),
                            ..default()
                        },
                        SelectionHandle,
                    ));

                    parent.spawn((
                        MaterialMesh2dBundle {
                            mesh: foreground.clone(),
                            material: material_white.clone(),
                            transform: Transform::from_xyz(corner.x, corner.y, i as f32 + 3.0),
                            ..default()
                        },
                        SelectionHandle,
                    ));
                }
            });
    }
}

// fn handle_deselection(
//     mut deslection: EventReader<Pointer<Deselect>>,
//     selected: Query<Entity, With<Selected>>,
//     handles: Query<Entity, With<SelectionHandles>>,
//     mut commands: Commands,
// ) {
//     for deselect in deslection.read() {
//         for entity in selected.iter() {
//             commands.entity(entity).remove::<Selected>();
//             commands.entity(handles.single()).despawn_recursive();
//         }
//     }
// }

fn handle_resize(
    mut drags: EventReader<Pointer<Drag>>,
    handles: Query<Entity, With<SelectionHandle>>,
    mut images: Query<(&mut Sprite, &Handle<Image>)>,
    image_store: Res<Assets<Image>>,
) {
    for drag in drags.read() {
        let Ok(_) = handles.get(drag.target) else {
            debug!("Running");
            continue;
        };

        let (mut sprite, handle) = images.single_mut();

        let dimensions = match sprite.custom_size {
            Some(mut dimensions) => {
                dimensions.x = dimensions.x + drag.delta.x;
                dimensions.y = dimensions.y - drag.delta.y;

                dimensions
            }
            None => {
                let Some(image) = image_store.get(handle.id()) else {
                    continue;
                };

                let mut dimensions = image.size_f32();

                dimensions.x = dimensions.x + drag.delta.x;
                dimensions.y = dimensions.y - drag.delta.y;

                dimensions
            }
        };

        sprite.custom_size = Some(dimensions);
    }
}
