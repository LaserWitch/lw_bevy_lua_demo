use bevy::{
    app::{App, Plugin},
    ecs::schedule::States,
};

pub struct GameStatesPlugin;
impl Plugin for GameStatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();
    }
}

//lifted nearly if not completely unchanged from Bevy's example code
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    CollectScripts,
    SetupScripts,
    Loading,
    Playing,
    LoadGame,
    Menus,
    LoadLevel,
    Level,
}
