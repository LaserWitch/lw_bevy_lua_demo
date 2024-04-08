use std::any::TypeId;

use bevy::asset::LoadedFolder;
use bevy_asset_loader::prelude::*;

use crate::{api::OurAPIBase, asset::LuaFennel, host::OurScriptHost};
use bevy::{
    app::{App, Plugin, PostUpdate, Update},
    asset::{embedded_asset, embedded_path, AssetServer, Assets, Handle, UntypedHandle},
    ecs::{
        entity::Entity,
        event::EventReader,
        schedule::{
            //apply_deferred,
            common_conditions::in_state,
            IntoSystemConfigs,
            NextState,
            OnExit,
        },
        system::{Commands, Res, ResMut, Resource},
    },
    log::info,
    prelude::{Deref, DerefMut},
};
use bevy_mod_scripting::core::{
    events::PriorityEventWriter,
    hosts::{Recipients, Script, ScriptCollection},
    AddScriptApiProvider, AddScriptHost, AddScriptHostHandler, ScriptingPlugin,
};
use bevy_mod_scripting::prelude::LuaBevyAPIProvider;
use bevy_mod_scripting::prelude::LuaCoreBevyAPIProvider;
use bevy_mod_scripting_lua::LuaEvent;
use bevy_script_api::lua::RegisterForeignLuaType;
use ons_gamestates::GameState;

#[derive(AssetCollection, Resource, Default)]
struct ScriptAssets {
    #[asset(path = "", collection)]
    all: Vec<UntypedHandle>,
    _handle: Option<Handle<LoadedFolder>>,
}

pub struct LuaPlugin;

impl Plugin for LuaPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ScriptingPlugin)
            .insert_resource(ScriptAssets::default())
            .add_loading_state(
                LoadingState::new(GameState::CollectScripts)
                    .continue_to_state(GameState::SetupScripts)
                    .load_collection::<ScriptAssets>(),
            )
            .insert_resource(FrameCount::default())
            .add_systems(PostUpdate, tick_frames)
            .add_script_host::<OurScriptHost<()>>(PostUpdate)
            .add_api_provider::<OurScriptHost<()>>(Box::new(LuaCoreBevyAPIProvider))
            .add_api_provider::<OurScriptHost<()>>(Box::new(LuaBevyAPIProvider))
            .add_api_provider::<OurScriptHost<()>>(Box::new(OurAPIBase))
            .add_script_handler::<OurScriptHost<()>, 0, 0>(OnExit(GameState::SetupScripts))
            .add_script_handler::<OurScriptHost<()>, 0, 0>(PostUpdate)
            .add_systems(
                Update,
                (script_host_setup, load_script_modules)
                    .chain()
                    .run_if(in_state(GameState::SetupScripts)),
            )
            .add_systems(
                Update,
                (script_asset_monitor, do_update).run_if(in_state(GameState::Playing)),
            )
            .register_foreign_lua_type::<Entity>();
        let p = embedded_path!("src/", "lua/our_tools.lua");
        info!("{:#?}", p);
        embedded_asset!(app, "src/", "lua/our_tools.lua");

        embedded_asset!(app, "src/", "lua/fennel.lua");
        //            .update_documentation::<bevy_mod_scripting_lua::LuaScriptHost<()>>();
    }
}
fn load_script_modules(
    server: Res<AssetServer>,
    mut all_scripts: ResMut<ScriptAssets>,
    mut host: ResMut<OurScriptHost<()>>,
    scripts: Res<Assets<LuaFennel>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut w: PriorityEventWriter<LuaEvent<()>>,
    mut load_events: EventReader<bevy::asset::AssetEvent<LuaFennel>>,
) {
    if host.ready() {
        for ut_h in &all_scripts.all {
            if ut_h.type_id() == TypeId::of::<LuaFennel>() {
                let h = &((ut_h.clone()).typed::<LuaFennel>());
                let i = h.id();
                let name: String = server.get_path(i).unwrap().to_string();
                if let Some(x) = scripts.get(i) {
                    let r = host.run_asset_module(&name, h, x);
                    host.print_result(&r, true);
                }
            }
        }
        host.activate_pending_modules().expect("msg");

        if let Some(handle) = &host.loader_asset {
            let script = Script::<LuaFennel>::new("our_tools".into(), handle.clone());
            commands.spawn(ScriptCollection::<LuaFennel> {
                scripts: vec![script],
            });
        }
        w.send(
            LuaEvent {
                hook_name: "on_game_start".to_string(),
                args: (),
                recipients: Recipients::All,
            },
            0,
        );
        load_events.clear();
        all_scripts._handle = Some(server.load_folder("".to_string()));
        next_state.set(GameState::Playing);
    }
}

fn script_host_setup(
    server: Res<AssetServer>,
    mut host: ResMut<OurScriptHost<()>>,
    scripts: Res<Assets<LuaFennel>>,
    counter: Res<FrameCount>,
) {
    //Setup initial stuff
    let frame = counter.0;
    if host.loader_asset.is_none() {
        let loader_setup = server.load::<LuaFennel>("embedded://ons_lua/lua/our_tools.lua");
        //info!("{frame:9} l_s {loader_setup:#?}");
        host.loader_asset = Some(loader_setup.clone());
    }
    if host.fennel.is_none() {
        let fennel = server.load::<LuaFennel>("embedded://ons_lua/lua/fennel.lua");
        //info!("{frame:9} f {fennel:#?}");
        host.fennel = Some(fennel.clone());
    }

    if !host.ready() {
        host.try_start(&scripts);
        //if host.ready() {
        //    let _ = server.load_folder("scripts");
        //}
    }
}
#[derive(Resource, Default, Deref, DerefMut)]
struct FrameCount(u64);
fn tick_frames(mut count: ResMut<FrameCount>) {
    count.0 += 1;
}
fn script_asset_monitor(
    server: Res<AssetServer>,
    assets: Res<Assets<LuaFennel>>,
    mut events: EventReader<bevy::asset::AssetEvent<LuaFennel>>,
    counter: Res<FrameCount>,
    // mut waiting: Local<Vec<Handle<LuaFennel>>>,
    mut host: ResMut<OurScriptHost<()>>,
) {
    let frame = counter.0;
    //Host should be ready but lets be sure
    if host.ready() {
        let l = host.loader_asset.clone().unwrap();
        let l_path = l.path().unwrap();
        let f = host.fennel.clone().unwrap();
        let f_path = f.path().unwrap();
        for event in events.read() {
            //info!("\n{frame:9} | {event:?}");
            if let bevy::asset::AssetEvent::Modified { id } = event {
                if let Some(lf) = assets.get(*id) {
                    if let Some(h) = server.get_id_handle(*id) {
                        let p = h.path().expect("arghl");
                        let n = format!("{p}");
                        let r = host.run_asset_module(&n, &h, lf);
                        //info!("\n{frame:9} MOD--------------- \n\t{p} \n\t{n} \n\t{h:?} \n\t{r:?}");
                        host.activate_pending_modules().expect("msg");
                    };
                };
            };
            if let bevy::asset::AssetEvent::Added { id } = event {
                if let Some(lf) = assets.get(*id) {
                    if let Some(h) = server.get_id_handle(*id) {
                        let p = h.path().expect("arghl");
                        if !(p == l_path || p == f_path) {
                            let n = format!("{p}");
                            let r = host.run_asset_module(&n, &h, lf);
                            //info!("\n{frame:9} ADD--------- \n\t{p} \n\t{n} \n\t{h:?}\n\t{r:?}");
                            host.activate_pending_modules().expect("msg");
                        }
                    };
                };
            };
        }
    }
}

fn do_update(mut w: PriorityEventWriter<LuaEvent<()>>) {
    //info!("sending lua update tick");
    w.send(
        LuaEvent {
            hook_name: "on_level".to_string(),
            args: (),
            recipients: Recipients::All,
        },
        0,
    );
}
