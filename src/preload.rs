use std::time::Duration;

use bevy::{asset::ChangeWatcher, prelude::*};

//start_loading(...) invokes the asset server loading folders and stuffs the handles in a resource.
//while_loading(...) drops handles from that resources as they finish loading and advances to the next state once all are done.

pub struct LoadScreenPlugin;
impl Plugin for LoadScreenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PendingHandles::default())
            .add_plugins((DefaultPlugins.set(AssetPlugin {
                watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                ..default()
            }),))
            .add_state::<GameState>()
            .add_systems(OnEnter(GameState::Loading), start_loading)
            .add_systems(Update, (while_loading).run_if(in_state(GameState::Loading)));
    }
}

//lifted nearly if not completely unchanged from Bevy's example code
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
}

#[derive(Deref, DerefMut, Default, Resource)]
pub struct PendingHandles(Vec<HandleUntyped>);

//Just slurps up the script handles and sticks them in pending handles
pub fn start_loading(asset_server: Res<AssetServer>, mut library: ResMut<PendingHandles>) {
    library.append(
        &mut (asset_server
            .load_folder("scripts/")
            .expect("script loading failed")),
    );
}
pub fn while_loading(
    mut library: ResMut<PendingHandles>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    library.as_mut().retain(|x| {
        let l = asset_server.get_load_state(x);
        matches!(l, bevy::asset::LoadState::Loaded)
    });

    if library.is_empty() {
        next_state.set(GameState::Playing);
    }
}
