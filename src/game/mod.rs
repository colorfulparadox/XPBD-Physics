
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct BoidsConfig {
    #[inspector(min = 0.0, max = 1.0)]
    coherence: f32,
    #[inspector(min = 0.0, max = 1.0)]
    separation: f32,
    #[inspector(min = 0.0, max = 1.0)]
    alighment: f32,
    #[inspector(min = 0.0, max = 250.0)]
    visual_range: f32,
}

impl Default for BoidsConfig {
    fn default() -> Self {
        Self {
            coherence: 0.5,
            separation: 0.5,
            alighment: 0.5,
            visual_range: 75.
        }
    }
}

#[derive(Component)]
struct Boid {
    velo: Vec2,
    accel: Vec2
}

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<BoidsConfig>()
            .register_type::<BoidsConfig>()
            .add_plugins(ResourceInspectorPlugin::<BoidsConfig>::default());
    }   
}