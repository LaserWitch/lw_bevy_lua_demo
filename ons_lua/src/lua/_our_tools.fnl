(set _G.is (fn [v t] (= (type v) t)))

;(macro -- [...] (list (sym :comment) ...))
(macro -- [...]
  `(comment ,...))

(-- "This file was written in fennel and compiled to lua ahead of time. This does result in odd Lua code,"
    "no effort has been made to clean this up.")

(comment isnil)
(fn isnil [self v]
  (if (= (type v) :nil) true false))

(comment ensure_subtable_table)
(fn ensure_subtable_table [self t n]
  (if (not= (type t) :table)
      false
      (do
        (when (self:isnil (. t n)) (tset t n {}))
        (. t n))))

(fn ensure_subtable_array [self t n]
  (if (not= (type t) :table)
      false
      (do
        (when (self:isnil (. t n)) (tset t n []))
        (. t n))))

(comment event_run)
(fn event_run [self event-name ...]
  (local runners (. _G.__event_runners.by_function event-name))
  (each [k mod (pairs runners)]
    (when (= (type mod) :table)
      (local runner (. mod event-name))
      (when (= (type runner) :function)
        (print (.. "event " event-name " sent to module " (tostring k)
                   " result: " (tostring (runner mod ...)))))))
  (values))

(comment init)
(fn init [self]
  (let [evrun (self:sub-t _G :__event_runners)]
    (self:sub-t evrun :by_module)
    (self:sub-t evrun :by_function))
  (values))

(comment "Module loading needs phases
  1) Load/Compile(Insert): Compiles modules into chunks and stores them in package.preload. Stores names of pending packages.
  2) Execute/scan(Activate): Runs Require on all modules and scans them for event hooks.")

(comment insert_fennel_module)
(fn insert_fennel_module [self name source preload-functions]
  (local fennel (require :fennel))
  (local (compiled err) (fennel.compileString source))
  ;(print :loading :fennel :module name :result :is (type compiled))
  (if (not compiled)
      (do
        ;(print (.. "error fennel compiling " name "" err))
        (values compiled err))
      (self:insert_lua_module name compiled preload-functions)))

(fn insert_lua_module [self name source preload-functions]
  ;(when (not (= name :fennel))
    ;(print :loading :lua :module name :source)
    ;(print source);
   ; )
  (local (chunk err) (load source))
  ;(print :loading :lua :module name :result :is (type chunk))
  (if (not chunk)
      (print (.. "error evaluating " name ": " err))
      (tset preload-functions name chunk))
  (values chunk err))

(fn insert_module [self name is-lua source]
  (-- "TODO: The name handling has the full path from asset root. We need a more resiliant way to deal with that")
  (let [name (if (= name :fennel)
                 name
                 (string.sub (string.gsub name "/" ".") 9 -5))
        package (self:sub-t _G :package)
        preload (self:sub-t package :preload)
        pending (self:sub-a self :pending_modules)
        chunk (if is-lua
                  (self:insert_lua_module name source preload)
                  (self:insert_fennel_module name source preload))]
    (if chunk
        (do
          (tset package.preload name chunk)
          (table.insert pending name)
          ;(print (.. "success building module chunk for " name))
          )
        (print (.. "error building module chunk for ")))))

(comment "We need to maintain a list of what modules have what hook fns.
This needs to handle reloads, it will check if the hooks are already listed and remove them if the fn is not in the new moduel state")

(fn activate_all_pending [self]
  (let [package (self:sub-t _G :package)
        preload (self:sub-t package :preload)
        loaded (self:sub-t package :loaded)
        pending (self:sub-a self :pending_modules)
        f (require :fennel)] ; (print "starting activation of pending modules") ; (print (f.view self))
    ;(print :----preload--table------)
    ;(print (f.view preload))
    (each [i v (ipairs pending)]
      ;(print "activate all pending" :i- i :-v- v)
      (let [chunk (. preload v)
            existing (. loaded v)
            result (chunk)]
        ;(print "_" :activate v)
        ;(print "_" :result :is (type result))
        ;(f.view result))
        (when (and result (is result :table))
          (if (is existing :table)
              (do
                ;(print "_" "_" :merge)
                (self:merge existing result))
              (do
                ;(print "_" "_" :new)
                (tset loaded v result))))
        (when (. loaded v)
          (self:maintain_by_module v (. loaded v))
          (let [m (. loaded v)
                on_load m.on_load]
            (when on_load
              (m:on_load))))
        (tset pending i nil))) ; (print "---post_activation----")
    ;(print (f.view self))
    )
  ;(local f (require :fennel))
  ;(each [k1 v1 (pairs self.hook_lib)] ;  (print :_ k1) ;  (each [k2 v2 (pairs v1)] ;    (print :_ :_ k2) ;      (each [k3 v3 (pairs v2)] ;       (print :_ :_ :_ k3 (type v3)))
  ; ;    )) ;    (print (f.view self.hook_lib))
  )

(comment "The by-hook table is a means to look up modules by the hooks they contain")
(fn maintain_by_hook [self hook_name mod_name mod_table]
  (let [hooks (self:sub-t self :hook_lib)
        modules_by_hook (self:sub-t hooks :by_hook_name)
        this_hook_fn (. mod_table hook_name)]
    ;(print (.. "maintain_by_hook on mod_name " mod_name " hook_name " hook_name " is a " (type this_hook_fn)))
    (when (is this_hook_fn :function)
      (let [this_hook_tbl (self:sub-t modules_by_hook hook_name)]
        (tset this_hook_tbl mod_name mod_table)))))

(comment "The by-modules table is a means to look up all hooks a module contains
the key feature of this function is to clean up removed hook references on a reload that could remove modules")

(fn maintain_by_module [self mod_name fresh_mod_table]
  ;(print (.. "maintain_by_module on mod_name " mod_name))
  (let [hook_names self.hook_names
        hooks (self:sub-t self :hook_lib)
        modules_by_hook (self:sub-t hooks :by_hook_name)
        hooks_by_module (self:sub-t hooks :by_module_name)
        existing? (?. hooks_by_module mod_name)
        old_mod_table (self:sub-t hooks_by_module mod_name)]
    (when existing?
      (each [_ hook_name (ipairs hook_names)]
        ;(print hook_name "?")
        (-- "this indicates a hook that the new mod table lacks. If just merging by the keys of the new mod table these would persist and still be called, so we need to sweep for them")
        (when (is (. fresh_mod_table hook_name) :nil)
          ;(print :is :nil)
          (tset old_mod_table hook_name nil))))
    (-- "we want to avoid direct table assignment for fear of any possible references to an old table becoming broken/dangling.")
    (self:merge old_mod_table fresh_mod_table)
    (each [_ hook_name (ipairs hook_names)]
      ;(print hook_name "?")
      (-- "we use the old mod table here else we risk scope weirdness maybe")
      (self:maintain_by_hook hook_name mod_name old_mod_table))))

(fn invoke_global_hook [self hook_name ...]
  ;(local f (require :fennel)) ;   (print :preload)
  ;(each [k v (pairs _G.package.preload)] ;  (when (not= (string.sub k 1 6) :fennel) ;    (print (string.sub k 1 6) k) (f.view v)));) ;   (print :loaded) ;   (each [k v (pairs _G.package.loaded)] ;     (when (and  ;        (not= k :package)  ;        (not= k :loaded)  ;        (not= k :_G) ;        (not= (string.sub k 1 6) :fennel)) ;       (print k :: (f.view v))));)
  (let [hooks (self:sub-t self :hook_lib)
        modules_by_hook (self:sub-t hooks :by_hook_name)
        modules (. modules_by_hook hook_name)]
    (when modules
      (each [key val (pairs modules)]
        (when (is val :table)
        ;(print hook_name key)
        (let [args (self:deep_copy [...])
              hkfn (. val hook_name)]
          (when hkfn (hkfn val args)))))))
  (values))

(fn deep_copy [self from]
  (-- "to avoid weird side effects sometimes we need as strict a copy-by-value as possible")
  (if (is from :table)
      (let [to {}]
        (each [k v (pairs from)]
          (tset to (self:deep_copy k) (self:deep_copy v)))
        to) ; "in other cases we want to return the copy"
      from))

(fn merge [self into-table from-table]
  (each [k v (pairs from-table)]
    (global t (type v))
    (if (= t :table)
        (if (and (. into-table k) (= (type (. into-table k)) :table))
            (self:merge (. into-table k) (. from-table k))
            (tset into-table k (. from-table k)))
        (= t :function)
        (tset into-table k (. from-table k)))))

(set _G.our_tools {:hook_names [:on_game_start :on_level :on_load]
                   : init
                   : isnil
                   : ensure_subtable_table
                   :sub-t ensure_subtable_table
                   : ensure_subtable_array
                   :sub-a ensure_subtable_array
                   : activate_all_pending
                   : event_run
                   : insert_fennel_module
                   : insert_lua_module
                   : insert_module
                   : maintain_by_module
                   : maintain_by_hook
                   : invoke_global_hook
                   : deep_copy
                   : merge})
