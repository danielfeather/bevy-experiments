use bevy::{
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
        .add_systems(Update, (on_load, handle_deselection, show_box_on_selection))
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
        On::<Pointer<Drag>>::target_component_mut::<Transform>(move_on_drag),
        On::<Pointer<Click>>::target_commands_mut(|_, commands| {
            commands.insert(Selected);
        }),
    ));
}

fn move_on_drag(drag: &mut ListenerInput<Pointer<Drag>>, transform: &mut Transform) {
    transform.translation.x = transform.translation.x + drag.delta.x;
    transform.translation.y = transform.translation.y - drag.delta.y;
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

        commands
            .spawn((SpatialBundle::default(), SelectionHandles))
            .with_children(|parent| {
                let shape =
                    Mesh2dHandle(meshes.add(Ellipse::new(RESIZE_ANCHOR_AREA, RESIZE_ANCHOR_AREA)));

                for (i, corner) in corners.iter().enumerate() {
                    let color = Color::BLACK;
                    parent.spawn(MaterialMesh2dBundle {
                        mesh: shape.clone(),
                        material: materials.add(color),
                        transform: Transform::from_xyz(corner.x, corner.y, 1.0),
                        ..default()
                    });

                    let rect = Mesh2dHandle(meshes.add(sides[i]));

                    parent.spawn(MaterialMesh2dBundle {
                        mesh: shape.clone(),
                        material: materials.add(color),
                        transform: Transform::from_xyz(corner.x, corner.y, 1.0),
                        ..default()
                    });

                    parent.spawn(MaterialMesh2dBundle {
                        mesh: rect,
                        material: materials.add(color),
                        transform: Transform::from_xyz(corner.x, corner.y, 1.0),
                        ..default()
                    });
                }
            });
    }
}

fn handle_deselection(
    mut deslection: EventReader<Pointer<Deselect>>,
    selected: Query<Entity, With<Selected>>,
    handles: Query<Entity, With<SelectionHandles>>,
    mut commands: Commands,
) {
    for deselect in deslection.read() {
        for entity in selected.iter() {
            commands.entity(entity).remove::<Selected>();
            commands.entity(handles.single()).despawn_recursive();
        }
    }
}
