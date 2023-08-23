use bevy_mod_scripting::prelude::*;
use std::sync::Mutex;

mod our;
mod random;

/// In general this is quite rough code, and probably handles results in a very sub-optimal ways.
#[derive(Default)]
pub struct OurAPI;

impl APIProvider for OurAPI {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        our::insert(ctx)?;
        random::insert(ctx)?;
        Ok(())
    }

    fn setup_script(
        &mut self,
        _: &ScriptData,
        _: &mut Self::ScriptContext,
    ) -> Result<(), ScriptError> {
        Ok(())
    }
}

//Generalizing some from the bevy_mod_scripting API provider examples
fn insert_function<'lua, A, R, F>(
    ctx: &'lua Lua,
    t: &mut mlua::Table,
    name: &str,
    f: F,
) -> Result<(), bevy_mod_scripting::prelude::ScriptError>
where
    A: FromLuaMulti<'lua>,
    R: ToLuaMulti<'lua>,
    F: 'static + Send + Fn(&'lua Lua, A) -> mlua::Result<R>,
{
    t.set(
        name,
        ctx.create_function(f).map_err(ScriptError::new_other)?,
    )
    .map_err(ScriptError::new_other)
}
