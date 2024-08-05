use bevy::{
    log::{Level, LogPlugin},
    math::AspectRatio,
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::prelude::*;
use bevy_web_asset::WebAssetPlugin;

const RESIZE_ANCHOR_AREA: u32 = 32;

#[derive(Debug, Component, Reflect)]
struct Size {
    pub dimensions: UVec2,
    pub aspect_ratio: AspectRatio,
}

#[derive(Debug, Component, Reflect)]
struct Selected;

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
        .add_systems(Update, (on_load, handle_deselection))
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

fn show_box_on_selection() {}

fn handle_deselection(
    mut deslect: EventReader<Pointer<Deselect>>,
    selected: Query<Entity, With<Selected>>,
    mut commands: Commands,
) {
    for entity in selected.iter() {
        commands.entity(entity).remove::<Selected>();
    }
}
