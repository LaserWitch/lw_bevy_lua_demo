
(comment "convenience to add a component by string type name
and return the component handle")
(fn add_component [entity type-name]
  (let [t (world:get_type_by_name type-name)]
    (world:add_default_component entity t)
    (world:get_component entity t)))


(comment "I don't trust length in lua so this should ensure counting accurately.")
(fn count_table [t]
  (var count 0)
  (each [k v (pairs t)] (set count (+ count 1)))
  count)

{: add_component : count_table}