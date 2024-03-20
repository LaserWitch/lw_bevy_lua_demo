use bevy::prelude::*;
use bevy_mod_scripting::{ core::event::ScriptLoaded, prelude::* };
use bevy_script_api::lua::RegisterForeignLuaType;

//We need to be state aware so we don't try to start setting up scripts before all are loaded.
use crate::preload::GameState;

//The custom script host that contains the most significant code implementing our desired behavior.
mod host;
use host::*;
// home of the asset and loader types that accomodate Fennel loading.
mod lf_file;
use lf_file::*;
//API contains our API provider.
// Besides a helper function fairly standard
// The one open question for API implementation is if/how we can augment the BevyLuaAPI world without replacing it entirely.
mod api;
use api::*;

pub struct LuaPlugin;

impl Plugin for LuaPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ScriptingPlugin)
            .add_script_host::<OurScriptHost<()>>(PostUpdate)
            .add_api_provider::<OurScriptHost<()>>(Box::new(OurAPI))
            .add_api_provider::<OurScriptHost<()>>(Box::new(LuaBevyAPIProvider))
            .add_script_handler::<OurScriptHost<()>, 0, 0>(PostUpdate)
            .add_systems(OnEnter(GameState::Playing), load_startup_scripts)
            .add_systems(Update, do_update)
            .register_foreign_lua_type::<Entity>();
        //            .update_documentation::<bevy_mod_scripting_lua::LuaScriptHost<()>>();
    }
}
fn load_startup_scripts(
    server: Res<AssetServer>,
    script_assets: ResMut<Assets<LuaFennel>>,
    mut commands: Commands
) {
    info!("load_startup_scripts");
    let mut scripts = Vec::new();
    for (id, _) in script_assets.iter() {
        let h = server.get_id_handle(id).expect("msg");

        // GFZ: make_strong no longer exists
        //h.make_strong(&script_assets);

        let path = server.get_path(id).expect("msg");
        let n = path.path().to_str().unwrap().to_string();
        let s = Script::<LuaFennel>::new(n, h);

        info!("{:#?}", s);
        scripts.push(s);
    }
    commands.spawn(ScriptCollection::<LuaFennel> { scripts });
}
//I wanted to call script files directly but screw it, I'm using the event system that's in there
fn do_update(mut load_events: EventReader<ScriptLoaded>, mut w: PriorityEventWriter<LuaEvent<()>>) {
    // GFZ: this fires after load_script and setup_script
    for load_event in load_events.read() {
        let on_load = LuaEvent {
            hook_name: "on_load".to_string(),
            args: (),
            recipients: Recipients::ScriptID(load_event.sid),
        };
        w.send(on_load, 0);
    }

    w.send(
        LuaEvent {
            hook_name: "on_level".to_string(),
            args: (),
            recipients: Recipients::All,
        },
        0
    );
}
