use std::{ marker::PhantomData, sync::Mutex };

use super::lf_file::*;
use bevy::{ ecs::schedule::*, prelude::* };
use bevy_mod_scripting::{ core::{ systems::*, world::* }, prelude::* };
use bevy_mod_scripting_lua::*;
use mlua::*;

#[derive(Resource)]
/// OurScriptHost is derived from bevy_mod_scripting's LuaScriptHost.
/// Where LuaHost stores a new lua state in the context objects for each script,
/// OurHost instead stores it's own internal lua state that it runs every script in.
pub struct OurScriptHost<A: LuaArg> {
    lua: Mutex<Lua>,
    _ph: PhantomData<A>,
}

impl<A: LuaArg> Default for OurScriptHost<A> {
    fn default() -> Self {
        Self {
            _ph: Default::default(),
            lua: Mutex::from(Lua::new()),
        }
    }
}
// TODO: ensure run_one_shot works properly. It probably doesn't with the event handler changes.
impl<A: LuaArg> ScriptHost for OurScriptHost<A> {
    type ScriptContext = Mutex<Lua>;
    type APITarget = Mutex<Lua>;
    type ScriptEvent = LuaEvent<A>;
    type ScriptAsset = LuaFennel;
    type DocTarget = LuaDocFragment;

    //Besides the script asset and loader type this is still just a copy of the LuaScriptHost
    fn register_with_app_in_set(app: &mut App, schedule: impl ScheduleLabel, set: impl SystemSet) {
        app.add_priority_event::<Self::ScriptEvent>()
            .init_asset::<LuaFennel>()
            .init_asset_loader::<LuaFennelLoader>()
            .init_resource::<CachedScriptState<Self>>()
            .init_resource::<ScriptContexts<Self::ScriptContext>>()
            .init_resource::<APIProviders<Self>>()
            .register_type::<ScriptCollection<Self::ScriptAsset>>()
            .register_type::<Script<Self::ScriptAsset>>()
            .register_type::<Handle<LuaFennel>>()
            // handle script insertions removal first
            // then update their contexts later on script asset changes
            .add_systems(
                schedule,
                (
                    script_add_synchronizer::<Self>,
                    script_remove_synchronizer::<Self>,
                    script_hot_reload_handler::<Self>,
                )
                    .chain()
                    .in_set(set)
            );
    }

    /// load_script deviates heavily from LuaScriptHost's impl
    /// To make scripts live side by side, they are wrapped in a loader function and plugged into
    /// Lua's globals.preload table. They are later loaded through `require` by the event handler.
    ///
    /// This delayed loading, together with the 'loading screen behavior in the preload mod, allows
    /// scripts to depend each other without hitting load order issues.
    ///
    /// The loader behavior heavily uses lua code.
    fn load_script(
        &mut self,
        script: &[u8],
        script_data: &ScriptData,
        providers: &mut APIProviders<Self>
    ) -> std::result::Result<Self::ScriptContext, ScriptError> {
        {
            // We build a lua function out of the chunk loaded chunk...
            let lua = self.lua.lock().expect("bad lua state");
            let chunk = lua.load(script).set_name(script_data.name);
            /*
                .map_err(|e| {
                    ScriptError::FailedToLoad {
                        script: script_data.name.to_owned(),
                        msg: e.to_string(),
                    }
                })?;
            */

            let wrapped = chunk.into_function();
            // ... and insert it into the globals table under a name unlikely to collide with any user function names.
            // A reference to this function will end up in a table by the time we create another one
            // with this name, so we don't have to worry about tripping ourselves up.

            // GFZ: compiler warned about possible error in unused Result
            // TODO: circle back and see if any error handling should be done
            let _result = lua.globals().set("__temp_func", wrapped.unwrap());
            //info!("setting global table resulted in {:?}", _result);
            /*
                .map_err(|e| ScriptError::FailedToLoad {
                    script: script_data.name.to_owned(),
                    msg: e.to_string(),
                })?;
            */
            //Normal require syntax omits the file extension, so we need to strip that off
            //  it could be a fennel file too. Besides the extension everything special about them
            //  is handled before we get here.
            let name = match
                (script_data.name.strip_suffix(".lua"), script_data.name.strip_suffix(".fnl"))
            {
                (Some(l), None) => l,
                (None, Some(f)) => f,
                _ => "badname",
            };
            info!(
                "building internal loader for module \"{}\" from file {}",
                name,
                script_data.name
            );
            //TODO: the exact logic of when to call on_load may be wonky right now
            let runstr = format!(
                "
local name = \"{name}\"
--Out of significant paranoia, ensure the required table structure exists already
--  and then make local references to it for brevity
if not _G.package then
    _G.package = {{}}
end
local packs =  _G.package
if not packs.preload then
    packs.preload = {{}}
end
if not packs.loaded then
    packs.loaded = {{}}
end

local pr = packs.preload
local loaded = packs.loaded

function merge(into_table, from_table)
    for k,v in pairs(from_table) do
        t = type(v)
        if t == \"table\" then
            if into_table[k] and type(into_table[k]) == \"table\" then
                merge(into_table[k],from_table[k])
            else
                into_table[k] = from_table[k]
            end
        elseif t == \"function\" then
            into_table[k] = from_table[k]
        end
    end
end


--if loaded[name] is populated then someone has already called it and we need to recursive-merge the tables to avoid clobbering any module state.
local tf =  __temp_func
local function loader()
    local name = \"{name}\"
    --print(\"lua running require loader for bevy asset \" .. name )
    local result_table = tf(name)
    local final_table
    if package and package.loaded and package.loaded[name] and type(package.loaded[name]) == \"table\" then
        final_table = package.loaded[name]
        merge(final_table,result_table)
    else
        final_table = result_table
    end
-- GFZ: doesn't seem like this runs
--    if type(final_table) == \"table\" and type(final_table.on_load) == \"function\" then
--        final_table:on_load(name)
--    end
    return final_table
end


pr[name] = loader
local function rp(depth, tab)
    if tab.__LOOP then
        return
    end
    for k,v in pairs(tab) do
        typ = type(v)
        str = k
        for i=0,depth do
            str = \"  \"..str
        end
        if typ == \"table\" then
            tab.__LOOP = true
            print(str.. \" \"..typ)
            rp(depth+1,v)
            tab.__LOOP = nil
        elseif typ == \"userdata\" then
            print(str..\" \"..typ)
        else
            print(str..\" \"..typ..\" : \"..tostring(v))
        end
    end
end

--this crashes if a system mutates the exclusive world ref and the on_load uses it too
if loaded[name] then 
    local r = loader()
--    if r and type(r) == \"table\" and r.on_load then
--        r:on_load(name)
--    end
end
                "
            );
            // all that's left is to run our new chunk inside lua.
            let new_chunk = lua.load(&runstr);
            new_chunk.exec().map_err(|e| ScriptError::FailedToLoad {
                script: script_data.name.to_owned(),
                msg: e.to_string(),
            })?;
        }

        //Provider attachment just gets passed the self lua rather than a contextual lua
        //that's a bit wasteful, and potentially a problem if providers do any unconditual state setup,
        //but I haven't hit on a better way to ensure it gets done.
        providers.attach_all(&mut self.lua)?;

        //Due to the trait defintion we need to return a lua
        //That doesn't seem great conceptually if we end up with tons of scripts,
        //but it'll be effectively a one time cost so I'm letting it be.
        Ok(Mutex::new(Lua::new()))
    }

    fn setup_script(
        &mut self,
        script_data: &ScriptData,
        _ctx: &mut Self::ScriptContext,
        providers: &mut APIProviders<Self>
    ) -> std::result::Result<(), ScriptError> {
        //Only change from LuaScriptHost is passing the self context rather than the passed-in context
        providers.setup_all(script_data, &mut self.lua)
    }

    // Significant changes from original, detailed inside.
    // TODO: ensure we can have entity-attached scripts in some form still.
    fn handle_events<'a>(
        &mut self,
        world: &mut World,
        events: &[Self::ScriptEvent],
        ctxs: impl Iterator<Item = (ScriptData<'a>, &'a mut Self::ScriptContext)>,
        providers: &mut APIProviders<Self>
    ) {
        // safety:
        // - we have &mut World access
        // - we do not use the original reference again anywhere in this function

        // GFZ: used to be WorldPointer, now WorldPointerGuard
        let world = unsafe { WorldPointerGuard::new(world) };

        //Original LuaHost would get the script-specific context here
        //  now it has that be an ignored value because we use the shared context
        //  but we still want to iterate for the possibility of using the entity attachemnt features
        //  as originally intended
        ctxs.for_each(|(script_data, _)| {
            providers
                .setup_runtime_all(world.clone(), &script_data, &mut self.lua)
                .expect("Could not setup script runtime");

            //We're going to load with require
            //  so we need the suffixless path
            let name = match
                (script_data.name.strip_suffix(".lua"), script_data.name.strip_suffix(".fnl"))
            {
                (Some(l), None) => l,
                (None, Some(f)) => f,
                _ => "badname",
            };

            // We need to ensure that our script has been evaluated, and require does that for us
            //   lua stores the return of a module in a table and provides it to subsequent calls,
            //   or stores `true` if it doesn't return a table.
            // This means that the script loader function only ever gets called once, rather than needing to run it every frame for every instance of the script.
            let ctx = self.lua.get_mut().expect("Poison error in context");
            let code = format!("__event_reciever = require (\"{name}\")");
            let _ = ctx.load(code.as_bytes()).exec();

            // LuaHost searched the global context for hook functions
            //   it was possible to set up global hooks with the new structure,
            //   but only one could exist at a time.
            // Instead we're going to check the module table we got from require, if it did return a table.
            // Lua could return nil if the require failed, or true if it executed but had no return value.
            // in either case, no hooks to call.
            let globals = ctx.globals();
            for event in events {
                // check if this script should handle this event
                if
                    !event.recipients().is_recipient(&script_data) ||
                    //the required table needs to be a table to receive any events
                    globals
                        .get("__event_reciever")
                        .is_ok_and(|x: LuaValue| matches!(x, LuaValue::Boolean(_) | LuaValue::Nil))
                {
                    continue;
                }
                let t: Table = globals.get("__event_reciever").expect("bad table");
                let f: Function = match t.raw_get(event.hook_name.clone()) {
                    Ok(f) => { f }
                    Err(_) => {
                        continue;
                    } // not subscribed to this event
                }; //;

                if let Err(error) = f.call::<_, ()>(event.args.clone()) {
                    let mut world = world.write();

                    let mut state = world.remove_resource::<CachedScriptState<Self>>().unwrap();
                    let (_, mut error_wrt, _) = state.event_state.get_mut(&mut world);

                    let error = ScriptError::RuntimeError {
                        script: script_data.name.to_owned(),
                        msg: error.to_string(),
                    };

                    error!("{}", error);
                    error_wrt.send(ScriptErrorEvent { error });
                    world.insert_resource(state);
                }
            }
        });
    }
}
