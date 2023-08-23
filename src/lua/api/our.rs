use std::sync::Arc;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_mod_scripting::prelude::{LuaError, ScriptError, Value};
use bevy_script_api::{
    common::bevy::ScriptTypeRegistration,
    lua::bevy::LuaEntity,
    prelude::{GetWorld, ToLuaProxy},
};
use mlua::Lua;

use super::insert_function;

pub fn insert(ctx: &mut Lua) -> Result<(), ScriptError> {
    let mut t = ctx.create_table().unwrap();
    insert_function(ctx, &mut t, "all_with", list_entities)?;
    insert_function(ctx, &mut t, "new_poly", make_colored_triangle)?;
    ctx.globals()
        .set("our", t)
        .map_err(ScriptError::new_other)?;

    Ok(())
}

fn make_colored_triangle(
    ctx: &Lua,
    args: (LuaEntity, f32, f32, f32, f32, f32, f32, f32, usize),
) -> Result<Value<'_>, LuaError> {
    let (d, x, y, z, r, g, b, size, sides) = args;
    let position = Vec3::new(x, y, z);
    let color = Color::rgb(r, g, b);
    let entity = d.inner().expect("bad entity passed");

    let world = ctx.get_world()?;
    let mut world = world.write();
    world.resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
        world.resource_scope(|world, mut materials: Mut<Assets<ColorMaterial>>| {
            world
                .get_entity_mut(entity)
                .expect("bad entity")
                .insert(MaterialMesh2dBundle::<ColorMaterial> {
                    mesh: meshes
                        .add(shape::RegularPolygon::new(size, sides).into())
                        .into(),
                    material: materials.add(ColorMaterial::from(color)),
                    transform: Transform {
                        translation: position,
                        ..default()
                    },
                    ..default()
                });
        });
    });
    true.to_lua_proxy(ctx)
}
fn list_entities(ctx: &Lua, type_name: String) -> Result<Value<'_>, LuaError> {
    // retrieve the world pointer
    let world = ctx.get_world()?;
    let world = world.write();

    let registry: &AppTypeRegistry = world.get_resource().unwrap();
    let registry = registry.read();
    let c_id = registry
        .get_with_short_name(type_name.as_str())
        .or_else(|| registry.get_with_name(type_name.as_str()))
        .map(|registration| ScriptTypeRegistration::new(Arc::new(registration.clone())))
        .unwrap()
        .type_id();
    let entity_list: Vec<_> = world
        .iter_entities()
        .map(|entity| (entity.id(), entity.contains_type_id(c_id)))
        .filter(|pair| pair.1)
        .map(|(id, _)| id)
        .collect();
    entity_list.to_lua_proxy(ctx)
}
