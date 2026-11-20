---- MODULE dining_orders ----
EXTENDS Protocol

CONSTANT RecordThoughts
VARIABLE actions

Initial == Init /\ actions = <<>>
Step == /\ Next
        /\ actions' = CASE p0_eat(Phil0) -> Append(actions, "A")
                        [] p1_eat(Phil1) -> Append(actions, "B")
                        [] p2_eat(Phil2) -> Append(actions, "C")
                        [] RecordThoughts /\ p0_think(Phil0) -> Append(actions, "a")
                        [] RecordThoughts /\ p1_think(Phil1) -> Append(actions, "b")
                        [] RecordThoughts /\ p2_think(Phil2) -> Append(actions, "c")
                        [] OTHER -> actions
====
