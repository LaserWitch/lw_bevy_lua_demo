use bevy::{ asset::LoadedFolder, prelude::* };

//start_loading(...) invokes the asset server loading folders and stuffs the handles in a resource.
//while_loading(...) drops handles from that resources as they finish loading and advances to the next state once all are done.

pub struct LoadScreenPlugin;
impl Plugin for LoadScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins)
            //.add_state::<GameState>()
            .init_state::<GameState>()
            .add_systems(OnEnter(GameState::Loading), start_loading)
            .add_systems(Update, while_loading.run_if(in_state(GameState::Loading)));
    }
}

//lifted nearly if not completely unchanged from Bevy's example code
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
}

// GFZ: LoadedFolder field "handles" contains a Vec<UntypedHandle, Global> for the requested folder
#[derive(Default, Resource)]
pub struct PendingHandles(Handle<LoadedFolder>);

//Just slurps up the script handles and sticks them in pending handles
pub fn start_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(PendingHandles(asset_server.load_folder("scripts/")));
}

pub fn while_loading(
    mut events: EventReader<AssetEvent<LoadedFolder>>,
    handles: Res<PendingHandles>,
    mut next_state: ResMut<NextState<GameState>>
) {
    // GFZ: this won't work if there is more than one load_folder
    // when the folder has an update, it fires AssetEvent<LoadedFolder>
    // we use the handles in LoadedFolder to check if recursive deps are met
    for event in events.read() {
        if event.is_loaded_with_dependencies(&handles.0) {
            next_state.set(GameState::Playing);
        }
    }
}
