# Terminology

`f` : max failure replicas
`Q` : quorum, `|Q| = f + 1`

# Init
Instance `a` add all seen instance into `a.deps`.

# FastAccept

Broadcast `a` to a `Q`.
On other replica, if there is a `x: x ∉ a.deps` is seen, response `x` to leader of `a` `La`.

`La` commit `a` if no such `x` replied.

Otherwise, `La` creates an empty instance `a₁`, with all such `x` included in
`a₁.deps` and run FastAccept with `a₁`.

# Execution

All `a.deps` must be executed before `a`.
Then `a` is **ready** to execute, but cant execute it yet.

`a` can be executed when there is a `z` depends-on `a`: `a ∈ z.deps`.


# Recovery

If `x ∈ a.deps` and `a` wait too long for `x` to be present.
A recovery process is started to increment ballot and run paxos to commit an
empty `x`.


# Lemma-acyclic

An instance `a` only depends-on instances that are initiated before `a`.
∴ there is no cycle of depends-on.


# Consistency

∵ `a` wont change after initiated.
∴ all replicas have the same view of all instances.

# Execution-linearizability


periodically propose execution seed by every leader. seed interferes with each other.
