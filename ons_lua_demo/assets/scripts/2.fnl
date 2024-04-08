(local {: add_component} (require :common))

(comment "
  on_load runs every time the script is loaded,
  so by resaving the file and triggering the change detection, this gets invoked
  All our snippet does is put zero-duration lifetime components on everything
  effectively cleaning the scene")

(fn on_load []
; onload all_with currently crashes idk why
;  (each [k v (pairs (our.all_with :TagA))]
;    (local l (add_component v :Lifetime))
;    (tset l 1 0))
)

{: on_load}
