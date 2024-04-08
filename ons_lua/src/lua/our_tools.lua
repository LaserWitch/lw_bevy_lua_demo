local function _1_(v, t)
    return (type(v) == t)
  end
  _G.is = _1_
  --[[ "This file was written in fennel and compiled to lua ahead of time. This does result in odd Lua code," "no effort has been made to clean this up." ]]
  --[[ isnil ]]
  local function isnil(self, v)
    if (type(v) == "nil") then
      return true
    else
      return false
    end
  end
  --[[ ensure_subtable_table ]]
  local function ensure_subtable_table(self, t, n)
    if (type(t) ~= "table") then
      return false
    else
      if self:isnil(t[n]) then
        t[n] = {}
      else
      end
      return t[n]
    end
  end
  local function ensure_subtable_array(self, t, n)
    if (type(t) ~= "table") then
      return false
    else
      if self:isnil(t[n]) then
        t[n] = {}
      else
      end
      return t[n]
    end
  end
  --[[ event_run ]]
  local function event_run(self, event_name, ...)
    local runners = _G.__event_runners.by_function[event_name]
    for k, mod in pairs(runners) do
      if (type(mod) == "table") then
        local runner = mod[event_name]
        if (type(runner) == "function") then
          print(("event " .. event_name .. " sent to module " .. tostring(k) .. " result: " .. tostring(runner(mod, ...))))
        else
        end
      else
      end
    end
    return 
  end
  --[[ init ]]
  local function init(self)
    do
      local evrun = self["sub-t"](self, _G, "__event_runners")
      self["sub-t"](self, evrun, "by_module")
      self["sub-t"](self, evrun, "by_function")
    end
    return 
  end
  --[[ "Module loading needs phases
    1) Load/Compile(Insert): Compiles modules into chunks and stores them in package.preload. Stores names of pending packages.
    2) Execute/scan(Activate): Runs Require on all modules and scans them for event hooks." ]]
  --[[ insert_fennel_module ]]
  local function insert_fennel_module(self, name, source, preload_functions)
    local fennel = require("fennel")
    local compiled, err = fennel.compileString(source)
    if not compiled then
      return compiled, err
    else
      return self:insert_lua_module(name, compiled, preload_functions)
    end
  end
  local function insert_lua_module(self, name, source, preload_functions)
    local chunk, err = load(source)
    if not chunk then
      print(("error evaluating " .. name .. ": " .. err))
    else
      preload_functions[name] = chunk
    end
    return chunk, err
  end
  local function insert_module(self, name, is_lua, source)
    --[[ "TODO: The name handling has the full path from asset root. We need a more resiliant way to deal with that" ]]
    local name0
    if (name == "fennel") then
      name0 = name
    else
      name0 = string.sub(string.gsub(name, "/", "."), 9, -5)
    end
    local package = self["sub-t"](self, _G, "package")
    local preload = self["sub-t"](self, package, "preload")
    local pending = self["sub-a"](self, self, "pending_modules")
    local chunk
    if is_lua then
      chunk = self:insert_lua_module(name0, source, preload)
    else
      chunk = self:insert_fennel_module(name0, source, preload)
    end
    if chunk then
      package.preload[name0] = chunk
      return table.insert(pending, name0)
    else
      return print("error building module chunk for ")
    end
  end
  --[[ "We need to maintain a list of what modules have what hook fns.
  This needs to handle reloads, it will check if the hooks are already listed and remove them if the fn is not in the new moduel state" ]]
  local function activate_all_pending(self)
    local package = self["sub-t"](self, _G, "package")
    local preload = self["sub-t"](self, package, "preload")
    local loaded = self["sub-t"](self, package, "loaded")
    local pending = self["sub-a"](self, self, "pending_modules")
    local f = require("fennel")
    for i, v in ipairs(pending) do
      local chunk = preload[v]
      local existing = loaded[v]
      local result = chunk()
      if (result and is(result, "table")) then
        if is(existing, "table") then
          self:merge(existing, result)
        else
          loaded[v] = result
        end
      else
      end
      if loaded[v] then
        self:maintain_by_module(v, loaded[v])
        local m = loaded[v]
        local on_load = m.on_load
        if on_load then
          m:on_load()
        else
        end
      else
      end
      pending[i] = nil
    end
    return nil
  end
  --[[ "The by-hook table is a means to look up modules by the hooks they contain" ]]
  local function maintain_by_hook(self, hook_name, mod_name, mod_table)
    local hooks = self["sub-t"](self, self, "hook_lib")
    local modules_by_hook = self["sub-t"](self, hooks, "by_hook_name")
    local this_hook_fn = mod_table[hook_name]
    if is(this_hook_fn, "function") then
      local this_hook_tbl = self["sub-t"](self, modules_by_hook, hook_name)
      do end (this_hook_tbl)[mod_name] = mod_table
      return nil
    else
      return nil
    end
  end
  --[[ "The by-modules table is a means to look up all hooks a module contains
  the key feature of this function is to clean up removed hook references on a reload that could remove modules" ]]
  local function maintain_by_module(self, mod_name, fresh_mod_table)
    local hook_names = self.hook_names
    local hooks = self["sub-t"](self, self, "hook_lib")
    local modules_by_hook = self["sub-t"](self, hooks, "by_hook_name")
    local hooks_by_module = self["sub-t"](self, hooks, "by_module_name")
    local existing_3f
    do
      local t_19_ = hooks_by_module
      if (nil ~= t_19_) then
        t_19_ = (t_19_)[mod_name]
      else
      end
      existing_3f = t_19_
    end
    local old_mod_table = self["sub-t"](self, hooks_by_module, mod_name)
    if existing_3f then
      for _, hook_name in ipairs(hook_names) do
        --[[ "this indicates a hook that the new mod table lacks. If just merging by the keys of the new mod table these would persist and still be called, so we need to sweep for them" ]]
        if is(fresh_mod_table[hook_name], "nil") then
          old_mod_table[hook_name] = nil
        else
        end
      end
    else
    end
    --[[ "we want to avoid direct table assignment for fear of any possible references to an old table becoming broken/dangling." ]]
    self:merge(old_mod_table, fresh_mod_table)
    for _, hook_name in ipairs(hook_names) do
      --[[ "we use the old mod table here else we risk scope weirdness maybe" ]]
      self:maintain_by_hook(hook_name, mod_name, old_mod_table)
    end
    return nil
  end
  local function invoke_global_hook(self, hook_name, ...)
    do
      local hooks = self["sub-t"](self, self, "hook_lib")
      local modules_by_hook = self["sub-t"](self, hooks, "by_hook_name")
      local modules = modules_by_hook[hook_name]
      if modules then
        for key, val in pairs(modules) do
          if is(val, "table") then
            local args = self:deep_copy({...})
            local hkfn = val[hook_name]
            if hkfn then
              hkfn(val, args)
            else
            end
          else
          end
        end
      else
      end
    end
    return 
  end
  local function deep_copy(self, from)
    --[[ "to avoid weird side effects sometimes we need as strict a copy-by-value as possible" ]]
    if is(from, "table") then
      local to = {}
      for k, v in pairs(from) do
        to[self:deep_copy(k)] = self:deep_copy(v)
      end
      return to
    else
      return from
    end
  end
  local function merge(self, into_table, from_table)
    for k, v in pairs(from_table) do
      t = type(v)
      if (t == "table") then
        if (into_table[k] and (type(into_table[k]) == "table")) then
          self:merge(into_table[k], from_table[k])
        else
          into_table[k] = from_table[k]
        end
      elseif (t == "function") then
        into_table[k] = from_table[k]
      else
      end
    end
    return nil
  end
  _G.our_tools = {hook_names = {"on_game_start", "on_level", "on_load"}, init = init, isnil = isnil, ensure_subtable_table = ensure_subtable_table, ["sub-t"] = ensure_subtable_table, ensure_subtable_array = ensure_subtable_array, ["sub-a"] = ensure_subtable_array, activate_all_pending = activate_all_pending, event_run = event_run, insert_fennel_module = insert_fennel_module, insert_lua_module = insert_lua_module, insert_module = insert_module, maintain_by_module = maintain_by_module, maintain_by_hook = maintain_by_hook, invoke_global_hook = invoke_global_hook, deep_copy = deep_copy, merge = merge}
  return nil 