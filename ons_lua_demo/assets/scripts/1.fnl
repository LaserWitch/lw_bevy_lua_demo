
(comment "import a few helpers from the common libscript")
(print "foo")
; Live development and ECS composition!
;; Just a fun demo of how this can work
;; adjust values, or comment out sections, and see the app respond
(fn on_level []
  (local {: add_component : count_table} (require :common))

  (comment "First check how many entities are active with our tracking tag
    If there's less than our threshold we'll spawn one this frame.")
  (when (> 1000 (count_table (our.all_with :TagA)))
    (comment "spawn returns an entity index other API functions use for inputs")
    (let [e (world:spawn)]

      (comment "we add a tag so this counts towards our count")
      (world:add_default_component e (world:get_type_by_name :TagA))

      (comment Next we want to add some visuals to it)
      (let [(x y z) (values 0 0 0)
            sides 3
            radius 2
            brightness 10.0
            r (* (rand.unit) brightness)
            g (* (rand.unit) brightness)
            b (* (rand.unit) brightness)]
        (comment "Create a sprite though actually it's a colored mesh polygon")
        (our.new_poly e x y z r g b radius sides))

      (comment "lets give it a position. The new_poly lets us do that
        but we can mess with it afterwards too")

      (comment "the entity already has a transform component but inserting a new one
        is a safe way to get a handle for it")
      (let [transform (add_component e :Transform)
            pos transform.translation
            radius 100]
        (set pos.x (rand.range (- radius) radius))
        (set pos.y (rand.range (- radius) radius)))

      (comment "give our component a duration!" )
      (let [life (add_component e :Lifetime)]
        (tset life 1 (* (rand.unit) 4)))

      (comment "Make them go zoom")
      (let [speed 100
            _ (comment "Velocity is a deref tuple in rust and our translation to lua
              is currently a bit awkwar  so we need a bit of extra accessing stuff")
            vel (. (add_component e :Velocity) 1)]
        (set vel.x (rand.range (- speed) speed))
        (set vel.y (rand.range (- speed) speed)))
    )
    ))
{: on_level }