--[[ "convenience to add a component by string type name
and return the component handle" ]]
local function add_component(entity, type_name)
    local t = world:get_type_by_name(type_name)
    world:add_default_component(entity, t)
    return world:get_component(entity, t)
end
  --[[ "I don't trust length in lua so this should ensure counting accurately." ]]
local function count_table(t)
    local count = 0
    for k, v in pairs(t) do
      count = (count + 1)
    end
    return count
end

return {add_component = add_component, count_table = count_table} 