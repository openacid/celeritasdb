<!--ts-->


<!-- Added by: drdrxp, at: Thu Feb 20 19:24:26 CST 2020 -->

<!--te-->

2020 Aug 11

TODO change: relation between a and b: the order between a and b

# Goals

Major simplifications with comparison to epaxos:

- Removed livelock issue with infinite SCC from execution algo.

- Removed `seq` thus recovery is significantly simplified. Add `knows` attribute to determine execution order. `knows` is a more generalized concept than `seq` and can be specialized to `seq`.

- Accept-ed, Commit-ed status now are per-dep field.

- `deps` now contains instance id and all instances it knows.

- Get rid of `defered recovery` from recovery. Instead, one Prepare+Accept is
    quite enough.

- Simplify the proof of correctness.

- Recovery is the as simple as a Prepare and standard replication protocol.

- When updating `deps` for FastAccept of `a`,
    add `x` into `a.deps` only when `x` does not depends-on `a`, i.e., `x < a`.
    This reduces some of the unnecessary cycle of depends-on.

- Instances by a same leader has a depends-on relation.
  A later instance always depends on a former one.
  This is guaranteed by handling FastAccept request sequentially.

- `Ballot` is a replica attribute, instead of an instance attribute.

# Terminology

- `R0`, `R1` ... or `R[0]`, `R[1]`... : replica.
- `a`, `b` ... `x`, `y`... : instance.
- `a₀`: the value of `a` when `a` is initiated on its leader.
- `L(a)`, `La`, `L(b)`, `Lb`: is the leader replica of an instance `a` or `b`.

- `f`: number of max allowed failed replica that.
- `n`: number of replicas, `n = 2f+1`.
- `Qc`: classic quorum: `|Qc| = f+1`.
- `Qf`: quorum of fast round:
  ```
  |Qf| = f+⌊|Qc|/2⌋
       = f+⌊(f+1)/2⌋
       = ⌊(3f+1)/2⌋
       = ⌊(3n-1)/4⌋
  ```

- `a ~ b`: interfere: `a` and `b` can not exchange execution order in any
    instance(command) sequence.
- `a → b`: depends-on: `a` interferes with `b` and has seen `b`.
- `a < b`: do-not-know: `a` did not see `b`.
- `a ↦ b`: exec-depends-on: `a` execute after `b`.



# Protocol guarantees

- G-consistency: two interfering instance `a, b` must have at least one relation
    established.

- G-exec-linearizability

- Safety

- 

# Execution guarantees



# Definition instance space

The entire instance space is a 3d array:

```
R[i][j][idx]

```

Fields:

- i: replicaID.
- j: replicaID.
- idx: index of a instance.

Explain:

- `R[i]`: all data on replica `i`
- `R[i][j]`: instances initiated by replica `j` those are stored on replica `i`.
- `R[i][j][idx]`: `idx`-th instance initiated by replica `j`.

## instance space layout

```
|                                                                        |
|                                                                        |
|                                                                        |
|                    c     f             c     f              c    f     |
|              a     b     e       a     b     e        a     b    e     |
|              ----------------    ----------------    ----------------  |
| leader:      [0]   [1]   [2]     [0]   [1]   [2]     [0]   [1]   [2]   |
|              ================    ================    ================  |
| replica:     R[0]                R[1]                R[2]              |
```


## Def-Ballot

Ballot is same as the ballot in paxos,
except it is a vector and is used to protect a relation between two instance.

A replica has a attribute `term`, and a ballot is vector of the two leaders of a
relation to decide:
`blt = {Ra: Ra.term, Rb:Rb.term}`.
In this way, operations on relation of `a, b` are serialized.

A replica remembers `term`s of all replicas, playing the same role of `last_blt`
in paxos: `Ri.last_blt = {R0: R0.term, R1: R1.term, ...}`.

If the request `blt` is not greater-or-equal the replica `last_blt`, the reqeust
is declined, i.e., a request will be served if `blt[a] >= last_blt[a] and blt[b] >= last_blt[b]`.

- `blt` does not decline a FastAccept request, because FastAccept does not
    change anything, it just reads an unknown value.
    TODO.

Ballot is used to identify a leader, just in the way paxos does.
A recovery process Prepare with a higher `blt` to become the new leader of a
replica.
A replica seizes the leadership the same way.

E.g.:
Initially `La` `Lb` have ballot `1, 2`, FastAccept and Accept leader use this
ballot to run.
If a recovery process starts to recover `b`, it increment ballot of Lb to 3.
Then run recovery with ballot `1, 3`.
`(1, 3)`

```
type Ballot HashMap<ReplicaID, i64>;
```

## Def-instance

An instance is the internal representation of a client request.

```
type InstanceID(ReplicaID, i64)

type Knows HashMap<InstanceID>

type Dep {
    id: InstanceID;
    status: FastAccepted | Accepted | Committed,
    knows: Knows;
}

type Instance {

    id:        InstanceID;
    cmds:      Vec<Commands>;

    knows:     Knows;
    deps:      HashMap<InstanceID, Dep>;
}
```

- `id`: is the instance id.

- `cmds` is the commands a client wants to execute.

- `knows` is all instances an instance knows and is used to determines order between instances when there is cycle of depends-on.
  `a.knows = {a} ∪ a.deps[0].knows ∪ a.deps[1].knows ∪ ...`

- `deps` is the set of intefering instances that `a` depends-on:
  `a.deps = {x : a → x}` with additional paxos related info.

  Two dependencies are different if:
    `x.id != y.id or x.knows != y.nows`

### Def-depends-on-previous

Instances proposed by a same leader have a natural depends-on relationship:
A newer instance depends-on all preceding ones.

This serializes execution of instances of a same leader
but doesnot affect the out-of-order commit feature.


# Proof: all replica have the same view of committed instance
TODO move down

There is only one value could be chosen to be safe.

∴ finally an instance is committed the same value on all replias.

∴ All replicas have the same set of instances and relations.

# Replication protocol

## Def-safe

A value of something is **safe** if:
it has been forwarded to enough replicas and constituted a quorum(`Qf` or `Qc`)
so that no other process(command leader or recovery process) would choose other
value for it to commit.

## Def-commit

The action commit is to broadcast to all replica about what value is **safe**.

### Commit an instance

With this protocol we need to ensure two info to be **safe**,
before committing an instance `a`:

- What to execute: `a.cmds`.

  To commit `a.cmds`, forwards it to a `Qc`,
  because `a.cmds` never changes.

- and the orders between `a` and all other interfering instances.

### Def-relations

For the order of two interfering instances `a ~ b`,
there are 3 legal relation pairs:

```
a → b and b < a: a depends-on b and b doesnot knows of a;
a → b and b → a: a b depends-on each other;
a < b and b → a: a doesnot knows of b and b depends-on a;
```

`a < b and b < a` is not allowed:
From G-exec-linearizability, 
two interfering instance must have at
least one relation established.
Otherwise `a` and `b` can be executed in arbitrary order.

### Def-leader

A leader is the process that initiates an instance.
`La` initiates `a` then tries to commit one of `a → b` or `a < b`.
`Lb` initiates `b` then tries to commit one of `b → a` or `b < a`.

### Def-basic-idea

Round-1, aka the FastPath, `La` reads the relation stored on every replica.
If a value that is safe is observed, FastCommit it.
Otherwise, run into Round-2, aka, the SlowPath, follow the paxos protocol,
Accept then Commit.

Initially a replica stores relationship for every pairs of instance,
but in a unknown status.

The `a` read it first, a replica decides the order is `a < b`.

### FastAccept-behavior

FastAccept only read the value on a replica, 
If a replica doesnot have `a` or `b` on it, the first arived decide the order:
e.g., if `a` arives first, the replica decides `a < b`.

Otherwise the replica returns the Commit-ed order.


## FastCommit-guarantee

If a value is FastCommit-ed, i.e., committed with one round,
The protocol must guarantees recovery always choose this value.

∴ we have:

### Lemma-FastQuorum-at-least-Qc:

The chosen value for FastCommit must be seen by any other quorum, i.e., 
A FastQuorum has intersection with every `Qc`.

## Lemma-FastCommit-all-equal

The value in a FastQuorum must be all the same to FastCommit:

Proof:

If a `Qc` can choose a value `v` by seeing some `v` and some other different value, then the other values are not needed.

∴ all values in FastQuorum are all the same.


### Lemma-FastCommit-entanglement

If two interfering instances `a ~ b` are both FastCommit-ed,
the value of `b` can be determined from the value of `a`: `a < b ⇔ b → a`.

Proof:

`a ~ b` are both FastCommit-ed,
from Def-basic-idea, TODO 
the FastAccpet request respect the existent relation.
Thus there is at least one replica on which the order is `a → b` and `b < a`.

∴ From Lemma-FastCommit-all-equal,
that `a` is FastCommit-ed with `a → b` implies that `b` is FastCommit-ed with `b < a` and vice versa.


### Def-recovery
TODO define recovery goals.

### recovery-safety

TODO define recovery-safety.
TODO define recovery need only a `Qc`.

### Recovery-without-leaders

If a recovery process reached either of leader of `a`(`La`) or leader of `b`(`Lb`),
it is able to find the relation between `a, b` from the leader.

From Lemma-FastCommit-entanglement, 
We only need to recover an order when it does not reach either of `La` and `Lb`.
This places demands on the definition of FastQuorum:




### Def-FastQuorum

A FastQuorum is a set of replicas to which `La` forwarded that guarantees
recovery-safety.

A FastQuorum requires: `|Qf'(a, b)| > |Qc|/2 + |Qc| - 3`,
where `Qf'(a, b) = Qf(a) \ {La, Lb}`, i.e., `Qf(a)` without leader of `a` or 
`b`.

Proof:

Assumes
`La` and `Lb` have chosen their FastQuorum as `Qf(a)` and `Qf(b)`.
Recovery process has chosen a classic quorum `Qc` for recovery.

From Recovery-without-leaders, `{La, Lb} ∩ Qc = ø`.

To satisfy FastCommit-guarantee, in a `Qc`, only 
one of `a < b` and `b < a` is allowed to constitute a FastQuorum.

`Qf(a), Qf(b)` must satisfies:
`Qf'(a, b) ∩ Qc` and `Qf'(b, a) ∩ Qc` must have intersection.

I.e., `|Qf'(a, b) ∩ Qc| > |Qc|/2` for any `Qc`.
```
∴ |Qf'(a, b) ∩ Qc|         > |Qc|/2
∴ |Qf'(a, b)| - (n-2-|Qc|) > |Qc|/2 
∴ |Qf'(a, b)|              > |Qc|/2 + |Qc| - 3
```

Examples:

- For 5 replicas, `Qc=3`, `|Qf'(a, b)|=2`.
- For 7 replicas, `Qc=4`, `|Qf'(a, b)|=4`.
- For 9 replicas, `Qc=5`, `|Qf'(a, b)|=5`.


### Lemma-entanglement-with-committed

If `b` is committed with `b < a`, then a can only be FastCommit-ed with `a → b` ,
because `a` must see a `a → b`.

If `b` is committed with `b → a`, then only `a < b` will be FastCommit-ed.
because `a → b` can be committed only when `b` is committed.
From TODO.

```
a    a→b   a→b  a<b   b
--------------
           ------------
La                    Lb

If a is FastCommit-ed with a→b, Lb still can SlowCommit b→a
```


### Def-Commit-protocol

`La != Lb`, otherwise no distributed algo is required.

1. `La` sends FastAccept request to a `Qf(a)`. So does `Lb`.

2. If `La` received identical replies from `Qf(a)`, e.g., `a → b` or `a < b`,
   FastCommit it.

3. If incompatible order is seen, e.g., `a → b` and `a < b`, run into Accept.


From Lemma-entanglement-with-committed, 
If Accept-ed `b → a` is seen, `b < a` is not FastCommit-ed.

A SlowCommit-ed `b < a` must have constituted a `Qc` thus only `a → b` can be committed.

> TODO: `b → a` can be SlowCommit-ed.

From Lemma-entanglement-with-committed, 
If Commit-ed `b < a` is seen, `La` can just FastCommit `a → b`.
Because two FastCommit-ed value is entangled from FastCommit-entanglement.
and SlowCommit-ed `b < a` always has intersection with another `Qc` thus it can
be recovered.

And this directly infers epaxos fast commit condition:


-----------------

Inference: the value of relationship between `a, x`,
If `x` is committed with `x < a`, then only `a → x` can be FastCommit-ed.
∴ A more strict conclusion is:



### FP-condition

From Lemma-entanglement-with-committed, another FastCommit condition is:
seeing a committed `b`, and forwarded `a` to a `Qc`.


Conditions must be sastisified to commit on FastPath:

- For every updated `a → x`, the leader received at least one reply with
  committed `x`.

- `a → x` constitutes a fast-quorum.


### FastCommit: One RTT commit conditions

All following statements requires FastAccept are sent to a `Qc`(including the leader).

- condition-1 `a < b` constituted a `Qf'(a, b)`: commit `a < b`. The same for `a > b`.
- condition-2 Accept-ed `a < b` constituted a `Qc`(including leader): commit `a < b`. The same for `a > b`.
- condition-3 `a > b` are Commit-ed: commit `a > b`.

Otherwise, SlowPath:
- send Accept `a < b` if `a > b` is not seen.
- send Accept `a → b` if `a > b` is seen.


### FastQuorum without the other leader

Excluding `a, b`, FastPath requires at least `|Qc|/2 + |Qc| - 2` replicas, 
i.e., `⌊f+1⌋/2 + F-1`.
Including the leader itself, it is `⌊f+1⌋/2 + f`.
as epaxos specified.

### FastCommit with other leader

If FastAccept of `a` reached `Lb`,
`Lb` does not provide any info to recovery.
Thus it requires some other constrain, to commit with no more replicas.
E.g. using the condition-3, a committed relation.

∴ The FastPath requirements are:
- If `Lb ∈ Qf(a)`: `|Qf(a)| >= f+1` and `b` is committed.
- If `Lb ∉ Qf(a)`: `|Qf(a)| >= ⌊f+1⌋/2 + f`.

The coresponding recovery is: after prepare:

### Def-Recovery-protocol

- `a:A/C`: commit a.
- `b:A/C`: commit b.

If only one is committed, from Lemma-entanglement-with-committed to choose the
other value and commit.

- `a:no-A and b:no-A`: `a` or `b` can only be FastCommit-ed with condition-1.
    If some value has more than half of `|Qc|`, choose it and commit.

- Otherwise, no value is FastCommit-ed. run SlowPath.


And epaxos is a special case of this protocol,
epaxos strengthened the condition to be: 
Always requires `b` to be committed
Always use `|Qf| = ⌊f+1⌋/2 + f`.


### Lemma-same-leader-order

∴ From Def-depends-on-previous, 
If `L(a) == L(b)` and `a < b`,
only `b → a` can be committed, `a → b` or `a ↔ b` can never be committed.


# Replication workflow

The previous section discussed how to determine the order of two instances.
But execution requires more to determine the order.
E.g., a cycle `a ↔ b` or `a → b → c → a`.
From TODO,
we use `knows` to determine execution order.
With `knows`, if `b₁.knows != b₂.knows`, `b₁, b₂` are different instances.
thus the two instance protocol applies to `a ~ b₁` and `a ~ b₂`.

```
fn interferings(a):
  rst = {}
  for x in local_instances:
    if x ~ a:
      rst.insert(x)
  return rst

fn new_dep(x):
  return Dep{
    id: x.id,
    accepted: false,
    committed: false,
    knows: x.knows,
    ballot: 0,
  }

fn update_dep(a, b):
  a.deps[b.id].knows ∪= b.knows
  if b.committed:
    a.deps[b.id].committed = true
  if b.accepted:
    a.deps[b.id].accepted = true

fn union(a, b):
  c = {}
  c.deps.keys = a.deps.keys ∪ b.deps.keys
  for id in c.deps:
    update_dep(c.deps[id], a.deps[id])
    update_dep(c.deps[id], b.deps[id])
  return c
```

## Fast path

1. Leader: Initiate instance `a₀`: fill `a₀.deps` with all local instances that
   interferes with `a₀`.  Then forward `a₀` to all other replicas.

2. NonLeader: handle FastAccept request: initiated `a'` the same way the leader initiates `a₀`.
   Then union `a'` and `a₀`: `a₁ = union(a', a₀)`.

    Reply `a₁`.

3. Leader: Handle FastAccept replies: For every instance `x` in replied `a.deps`, i.e., `x: x ∈ a.deps`, if it satisfies FP-condition: commit `x`. Otherwise enter SlowPath.

## Slow path

1. Leader: `union()` at least `|Qc|` replies to build `a₂`. Send Accept request with `a₂` to replicas.

2. NonLeader: handle Accept request: If the ballot in request is not smaller
   than the ballot in instance `a` locally,  `req.blt >= last_blt`, accept
   it. Then reply the `last_blt`

3. Leader: Handle Accept reply: if at least `|Qc|` OK replies are received,
   commit it.


## Replication workflow

```

|                                         |
| Leader                                  | Non Leader
| ---                                     | ---
| fn init(a₀):                            | ---
|   a₀.deps = {}                          |
|                                         |
|   for x in interferings(a₀):            |
|     a₀.deps.insert(x.id, new_dep(x))    |
|                                         |
| init(a₀)                                |
| forward(a₀)                             |
| ---                                     | ---
|                                         | fn handle_fast_accept_request(req):
|                                         |   a₀ = req.a₀
|                                         |   a' = a₀; a'.deps = {}
|                                         |   init(a')
|                                         |   a₁ = union(a', a₀)
|                                         |   reply(a₁)
| ---                                     | ---
| fn handle_fast_accept_replies(replies): |
|                                         |
|   dep_ids = {x.id: x ∈ replies[i]}      |
|   for dep_id in dep_ids:                |
|     same = true                         |
|     committed = false                   |
|     for repl in replies:                |
|        r0 = replies[0].deps[dep_id]     |
|        r  = repl.deps[dep_id]           |
|        same = same and r0 == r          |
|        if r.committed:                  |
|          committed = true               |
|                                         |
|     if same:                            |
|       if (r0 == a₀.deps[dep_id]):       |
|         commit(a, replies, dep_id)      |
|       else if committed:                |
|         commit(a, replies, dep_id)      |
|       else:                             |
|         slowpath(a, replies, dep_id)    |
|     else:                               |
|       slowpath(a, replies, dep_id)      |
|                                         |
| ---                                     | ---
| fn slowpath(a, replies, dep_id):        | ---
|                                         |
|   for repl in replies:                  |
|     update_dep(a, repl.deps[dep_id])    |
|   a.deps[dep_id].accepted = true        |
|                                         |
|   send_accept(a, dep_id)                |
| ---                                     | ---
|                                         | fn handle_accept_request(req):
|                                         |
|                                         |   a = req.q; dep_id = req.dep_id 
|                                         |   if req.blt >= last_blt:
|                                         |     save(a.deps[dep_id])
|                                         |     last_blt = req.blt
|                                         |     reply(OK)
|                                         |   else:
|                                         |     reply(last_blt)
|                                         |
| ---                                     | ---
| fn handle_accept_replies(replies):      |
|   ok = 0                                |
|   for repl in replies:                  |
|     if repl.ok:                         |
|       ok++                              |
|   if ok >= Qc:                          |
|     commit(a, dep_id)                   |
|                                         |
| ---                                     | ---
| commit(a, dep_id)                       |
|   a.deps[dep_id].committed = true       |
|   send_commit(a, dep_id)                |
```


# Execution

Order is defined as:

- `a.knows ⊃ b.knows` : exec `a` after `b`.
  From Def-after, if `a.knows ⊃ b.knows`, execute `a` after `b` guarantees
  linearizability.

- Otherwise: exec `a` and `b` in instance id order.

See exec-update-accumulated.md


# Recovery

Assumes:

- The instance to recover is `a`.
- The leader of `a` `La` is `R0`
- The recovery process is `P`(`P != R0`).

## workflow

Recovery-1: increment ballot: `blt₁ = blt₀; blt₁[La]++`

Recovery-2: Send FastAccept of `a` with `blt₁`.

Recovery-3: If conflicting ballot is returned in `last_blt`, update local ballot and retry from Recovery-1.

## Recovery-4: Cases not need to recover:

After Preparing on a quorum(`Qc`):

- If `P` saw `La`, exit and wait for `La` to commit `a`.

- If `P` saw a Commit-ed `a`: broadcast `a` and quit.

- If `P` saw a Accept-ed `a`: run Accept with this value and quit.

∴ `P` need to continue recovery only if all of `a` it saw are in FastAccept-ed.

## Recovery-5: Recover FastAccept-ed instance

`P` tries to choose an order for every instance `b: b ∈ a.deps` one by one.

## Recovery-5.1: choose the value to recover

`P` could see different values of `b: b ∈ a.deps`.

For two value `x, y`(e.g.
`a → b` and `a < b` are different,
`a → b₁` and `a → b₂` are different, where `b₁, b₂` have different `knows`):


If `P` reached `Lb`, wait for `Lb` to commit `b`:
- If `b` is committed with `bᵢ < a`, choose `a → bᵢ`. From TODO, only `a → bᵢ`
    can be FastCommit-ed.
- If `b` is committed with `bᵢ → a`, From TODO, only `a < b` can be
    FastCommit-ed. to determine which is committed, just prepare and accept with
    `a < b` to see if there is a conflict value.


If `P` didnot reach `Lb`:
From Def-FastQuorum:
FastCommit requires `⌊|Qc|/2⌋+1` identical FastAccept-ed value in a `Qc: {La, Ly} ∩ Qc = ø`.

∴ choose the value that has at least `⌊|Qc|/2⌋+1`, run Accept, and finally commit.

## Recovery-6 commit

Use the value of `a` in Recovery-5, run Accept and Commit.


From Ballot-helps-relation,
No other `z < a` can be Accept-ed, because `La` increments its ballot.
∴ If Accept-ed, no other `a → z` can be FastCommit-ed.


### Ballot-helps-relation

Vector ballot ensures interfering instances can not Accept.


# Optimization

## deps

On implementation, `a.deps` is split into `N` subset, where `N` is number of replicas.
Every subset contains only instances from leader `Ri`:
`a.deps[Ri] = {x | x.replicaID == Ri and a → x}`.

And `a.deps[Ri]` records only the max instance id.

This way,
`Dep.id` `Dep.accepted` `Dep.committed` are all recorded with a int.
Because only Accept-ed a implies all instances on La before a are also
Accept-ed.
So is Commit-ed.



## Lemma-prepare-fast

Prepare with a new ballot and FastAccept can be combined.


## only one of a, x need Accept

optimization maybe:

```
a a a→x a←x a←x x x
```
`a` always knows x knows a, so a does not need to know x.

## exclude non existent

When handling FastAccept, need to exclude instance known but not on local
replica.
This guarantees only need to check the highest interfering instance when
recovery.

No way. `a after b` requires `a` knows of all `b` knows of.

## prepare and FastAccept together

Prepare and FastAccept can be done together
Because Accept only check ballot.
FastAccept does not change existent FastAccept.


inexistent FastAccept-ed value can be treat as existent value but dont know what
it is.

∵ FastAccept never change a value.
∴ reading an old value does not need to be protected with Ballot.


## FastAccept does not need to check ballot

seeing a Accept-ed instance:
A recovery always choose a possibly FastCommit-ed value.
∴ accept this FastAccept request wont let any other one to choose a different value.

As an optimization, respect the accepted value is more efficient.
re-populating an Accept-ed value does not break consistency.


## Instance.deps

`Instance.knows` is a virtual attribute and is not need to be persisted on disk.
Because it can be calculated dynamically from `deps`:
`a.knows = {a} ∪ a.deps[0].knows ∪ a.deps[1].knows ∪ ...`


## reduce knows to seq


## no deps updated on same leader

From Def-depends-on-previous,
Because newer instance on La always depends-on older instance,
∴ `a.deps[La]` will never change thus do not need to recover it.


# Removed features(from previous version)

## FastAccept request do not need to forward commit status of other instances.

If `x` is FastCommit-ed:

- If `a` reached `Lx`, then `a` know if `x` is committed, because `Lx` is the
    first to commit.
    Although there is chance `x` is committed after `a` reaches `Lx`,
    `Lx` broadcasts `x is committed` very likely earlier than another instance
    brings `x is committed` through its FastAccept request.

- If `a` did not reach `Lx`, then `a` must have reached `g - {La, Lx}`,
  this prevent other value of `a > y` to commit.
  ∴ `a > x` is safe to FastCommit.

<!-- vi: iskeyword+=-
-->
