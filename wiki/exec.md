# Execution

- Find out the set of smallest instances of every leader: `S`.
  Easy to see `S` includes all instances those should execute first.

- If there are any `a → b` relations(`a.final_deps ⊃ b.final_deps`)
  in `S`, replace `S` with:
  `S = {x | x ∈ S and (∃y: y → x)}`

  Repeat this step until there is no `a → b` in `S`.

- Execute all instances in `S` in instance-id-order.
