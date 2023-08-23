--[[ "import a few helpers from the common lib file" ]]
local common = require("scripts/common")
local add_component = common.add_component
local count_table = common.count_table
local function on_level()
  --[[ "First check how many entities are active with our tracking tag
      If there's less than our threshold we'll spawn one this frame." ]]
  if (1000 > count_table(our.all_with("TagA"))) then
    --[[ "spawn returns an entity index other API functions use for inputs" ]]
    local e = world:spawn()
    --[[ "we add a tag so this counts towards our count" ]]
    world:add_default_component(e, world:get_type_by_name("TagA"))
    --[[ Next we want to add some visuals to it ]]
    do
      local x, y, z = 0, 0, 0
      local sides = 3
      local radius = 2
      local brightness = 10.0
      local r = (rand.unit() * brightness)
      local g = (rand.unit() * brightness)
      local b = (rand.unit() * brightness)
      --[[ "Create a sprite though actually it's a colored mesh polygon" ]]
      our.new_poly(e, x, y, z, r, g, b, radius, sides)
    end
    --[[ "lets give it a position. The new_poly lets us do that
            but we can mess with it afterwards too" ]]
    --[[ "the entity already has a transform component but inserting a new one
            is a safe way to get a handle for it" ]]
    do
      local transform = add_component(e, "Transform")
      local pos = transform.translation
      local radius = 100
      pos.x = rand.range(( - radius), radius)
      pos.y = rand.range(( - radius), radius)
    end
    --[[ "give our component a duration!" ]]
    do
      local life = add_component(e, "Lifetime")
      do end (life)[1] = (rand.unit() * 4)
    end
    --[[ "Make them go zoom" ]]
    local speed = 1000
    --[[ "Velocity is a deref tuple in rust and our translation to lua
                  is currently a bit awkwar  so we need a bit of extra accessing stuff" ]]
    local vel = (add_component(e, "Velocity"))[1]
    vel.x = rand.range(( - speed), speed)
    vel.y = rand.range(( - speed), speed)
    return nil
  else
    return nil
  end
end
return {on_level = on_level} 