local common = require("scripts/common")
local add_component = common.add_component

--[[ "
  on_load runs every time the script is loaded,
  so by resaving the file and triggering the change detection, this gets invoked
  All our snippet does is put zero-duration lifetime components on everything
  effectively cleaning the scene" ]]
local function on_load()
  for k, v in pairs(our.all_with("TagA")) do
    local l = add_component(+v, "Lifetime")
    do end (l)[1] = 0
  end
  return nil
end
return {on_load = on_load}