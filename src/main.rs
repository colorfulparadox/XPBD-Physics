use bevy::{prelude::*, window::PresentMode};
use bevy::window::{WindowResolution, WindowMode};
//use bevy::utils::Duration;

use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub const BACKGROUND_COLOR: Color = Color::hsl(200.,0.9, 0.04);

//https://github.com/laundmo/bevy_screen_diagnostics
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};
use boids::CorePlugin;

fn main() {
    let mut app = App::new();

    let asset_plugin = AssetPlugin {
        file_path: String::new(),
        processed_file_path: String::new(),
        watch_for_changes_override: Some(true),
        mode: AssetMode::Processed,
    };

    let window_plugin: WindowPlugin = WindowPlugin {
        primary_window: Some(Window {
            title: "Physics".to_string(),
            mode: WindowMode::Windowed,
            present_mode: PresentMode::AutoNoVsync,
            fit_canvas_to_parent: true,
            resolution: WindowResolution::new(1200., 850.),
            resizable: true,
            ..default()
        }),
        ..default()
    };

    //default bevy plugins
    app
        .insert_resource(ClearColor(
            BACKGROUND_COLOR,
        ))
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(asset_plugin)
                .set(window_plugin)
                .build(),
        );
    
    //external crates
    app
        .add_plugins((
            WorldInspectorPlugin::new(),
            ScreenDiagnosticsPlugin::default(),
            ScreenFrameDiagnosticsPlugin
    ));

    app.add_plugins(CorePlugin);

    app.add_systems(Startup, boids::quadtree::test_setup);
    app.add_systems(PreUpdate, (boids::quadtree::draw_quad_rects, boids::quadtree::draw_points));
    app.add_systems(FixedUpdate, (boids::quadtree::place_point, boids::quadtree::move_points, boids::quadtree::point_collision).chain());
    app.add_systems(PostUpdate, boids::quadtree::apply_constraint);

    app.run();
}
