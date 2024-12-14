pub mod core;
pub mod game;
pub mod quadtree;

use bevy::{
    prelude::*,
    //core_pipeline::bloom::{BloomSettings, BloomPrefilterSettings}
};

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<AppState>()
            .add_systems(Startup, setup_camera)
            .add_systems(PostUpdate, upd_lifetimes)

            .add_plugins(core::spring::SpringPlugin);
    }   
}

/*
    -------------------------------------
        DEFAULT COMPONENTS / STATES
    -------------------------------------
*/

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
}

#[derive(Component)]
pub struct Lifetime {
    pub timer: Timer
}

#[derive(Component)]
pub struct MainCamera;

/*
    -------------------------------------
        FUNCTIONS
    -------------------------------------
*/

fn upd_lifetimes(
    mut commands: Commands,
    mut lifetimes: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>
) {
    for (entity, mut lifetime) in &mut lifetimes {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn setup_camera(mut commands: Commands) {
    let mut trans: Transform = Transform::from_scale(Vec3::new(0.6, 0.6, 1.)); //0.5
    trans.translation = Vec3::new(0.,0.0,999.9);

    let mut proj: OrthographicProjection = OrthographicProjection::default();
    proj.scale = 1.7;

    commands.spawn((Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            projection: proj,
            transform: trans,
            ..default()
        },
        
    /*BloomSettings {
        intensity: 0.3,
        prefilter_settings: BloomPrefilterSettings {
            threshold: 0.6,
            threshold_softness: 0.4,
        },
        ..default()
    },*/
        core::spring::SpringVec::new(Vec2::new(500.0,-375.0), 25., 8.),
        Name::new("MainCamera"),
        MainCamera
    ));
}