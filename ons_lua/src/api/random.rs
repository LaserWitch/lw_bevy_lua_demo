use bevy_mod_scripting::prelude::{LuaError, ScriptError, Value};
use bevy_script_api::prelude::IntoLuaProxy;
use mlua::Lua;
use rand::{rngs::ThreadRng, Rng};
use rand_distr::Standard;

use super::insert_function;

pub fn insert(ctx: &mut Lua) -> Result<(), ScriptError> {
    let mut t = ctx.create_table().unwrap();
    insert_function(ctx, &mut t, "unit", random_unit)?;
    insert_function(ctx, &mut t, "range", random_range)?;
    ctx.globals()
        .set("rand", t)
        .map_err(ScriptError::new_other)?;

    Ok(())
}

fn random_unit(ctx: &Lua, _: ()) -> Result<Value<'_>, LuaError> {
    let mut rng: ThreadRng = Default::default();
    let f = rng.sample::<f32, _>(Standard);
    f.to_lua_proxy(ctx)
}
fn random_range(ctx: &Lua, args: (f32, f32)) -> Result<Value<'_>, LuaError> {
    let mut rng: ThreadRng = Default::default();
    let f = rng.gen_range(args.0..args.1);
    f.to_lua_proxy(ctx)
}
