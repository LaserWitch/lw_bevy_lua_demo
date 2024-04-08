use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::*,
};

//Contains the rust-side components and systems,
// also contains the commented out original 'target' system mod target;
mod target;
use target::*;

//Contains systems for waiting for all assets to load before going into update state
//mod preload;
//use preload::*;
//use ons_preload::*;
//Contains all our scripting stuff, naturally.
//mod lua;
use ons_gamestates::*;
use ons_lua::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_plugins(DefaultPlugins)
        .add_plugins(TargetGameplay)
        .add_plugins((GameStatesPlugin, LuaPlugin))
        .run();
}

fn setup(mut commands: Commands) {
    //Bloom makes for pretty tests so we specify an HDR camera and some bloom settings
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        BloomSettings::default(),
    ));
}
