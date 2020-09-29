2020 Sep 05

# Goals

Simplify epaxos.

Major simplifications and optimizations with comparison to epaxos:

- Get rid of the livelock issue with infinite SCC.

- This protocol is very simple, only two message: Prepare and Accept.
    epaxos has 2 for replication, 3 for recovery.

- Recovery is the as simple as a Prepare and standard replication protocol.
    Removed `defered recovery` from recovery. Instead, one Prepare+Accept is
    quite enough.

- Move `seq` into `deps` thus recovery doesnot need to consider `seq`.

- Accept-ed, Commit-ed status now are per-dep field.

- In this paper `Ballot` is a replica attribute, instead of an instance attribute.

- Simplify the proof of correctness.

# Terminology

- `R0`, `R1` ... or `R[0]`, `R[1]`... : replica.
- `a`, `b` ... `x`, `y`... : instance.
- `a₀`: the value of `a` when `a` is initiated on its leader.
- `La`, `Lb`: is the leader replica of an instance `a` or `b`.

- `f`: number of max allowed failed replica that.
- `n`: number of replicas, `n = 2f+1`.
- `Qc`: classic quorum: `|Qc| = f+1`.
- `Qf`: quorum of fast round: `|Qf| = f+⌊|Qc|/2⌋ = ⌊(3n-1)/4⌋`

- `a ~ b`: interfere: `a` and `b` can not exchange execution order in any
    instance(command) sequence.
- `a → b`: relation depends-on: `a` depends-on `b`.
- `a < b`: relation do-not-know: `a` doesnot see `b`.
- `a: → b`: instance status depends-on: instance `a` that depends-on `b`.
- `a: < b`: instance status do-not-know: instance `a` that doesnot see `b`.
- `a ↦ b`: relation exec-depends-on: `a` execute after `b`.


# Protocol guarantees

- G-consistency:
  there has to be at least one relation between two interfering instances `a, b` committed.
  Otherwise `a, b` may be executed in different order two replicas.

- G-exec-linearizability:
  If `a` is proposed after `b` is committed, `a` is always executed after
  `b`.

- Safety

# Replication

## Def-inst-space: Definition of instance space

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

### instance space layout

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
except it is a vector and is a barrier of both two instance and the relation between them.

Every replica has a attribute `term`, and a ballot is vector of the two leaders of a relation:
`blt = {Ra: Ra.term, Rb:Rb.term}`.

A replica stores `term`s of all replicas, playing the same role as `last_blt`
in paxos: `Ri.last_blt = {R0: R0.term, R1: R1.term, ...}`.

If the request `blt` is not greater-or-equal the replica `last_blt`, the reqeust
is declined, i.e., `blt[a] >= last_blt[a] and blt[b] >= last_blt[b]`.

Ballot is used to identify a leader, just in the way paxos does.
A recovery process Prepare with a higher `blt` on a quorum to become the new leader.

E.g.:
Initially `La` `Lb` have ballot `(1, 2)`, for Prepare and Accept request.
If a recovery process `P1` starts to recover `b`, it increment ballot of `Rb` to 3.
Then it tries to recover `b` with ballot `(1, 3)`.
Another recovery process `P2` with ballot `(2, 2)` would block `P1`, and `P1`
also blocks `P2`.

```
type Ballot HashMap<ReplicaID, i64>;
```

## Def-instance

An instance is the internal representation of a client request.

```
type InstanceID(ReplicaID, i64)

type View HashMap<InstanceID>

type Dep {
    id: InstanceID;
    accepted: Ballot;
    committed: bool;
    view: View;
}

type Instance {

    id:              InstanceID;
    ballot:          Ballot;
    accepted_ballot: Ballot;
    cmds:            Vec<Commands>;

    view:     View;
    deps:      HashMap<InstanceID, Dep>;

}
```

- `id`: is the instance id.

- `ballot`: is the highest seen ballot, 

- `accepted_ballot`: is the highest ballot of Accept-ed request. 

- `cmds` is the commands a client wants to execute.

- `view` is all instances an instance see and is used to determines order between instances when there is cycle of depends-on.
  `a.view = {a} ∪ a.deps[0].view ∪ a.deps[1].view ∪ ...`
  In practice, a simple int type `seq` is able to play the same role as `view`.

- `deps` is the set of intefering instances that `a` depends-on:
  `a.deps = {x : a → x}` with additional paxos related info.

  Two dependencies are different if:
    `x.id != y.id or x.view != y.view`

TODO when handling Prepare, the seq must be the max of all interfering
instances,  not only the max one.
Because former instance may be committed with a higher seq than latter instance.
This would break the G-exec-linearizability.


## Def-leader-order

Instances proposed by a same leader have a natural depends-on relation:
A newer instance depends-on all preceding ones.

This doesnot affect the out-of-order commit.

## Def-exec-order

If `b` is committed before `a` is proposed,
`a` see at least one `b.view` that is committed.

∴ If `a.view ⊃ b.view` then executing `a` after `b` satisfies G-exec-linearizability.

Because subset relation `⊃` is monotonic, thus there is no cycle.

And we define that a newer instance always executes after an older one.
Obviously this still have G-exec-linearizability held.



## Def-safe

A value of something is **safe** if:
no other process(leader or recovery process) would choose a different value to commit.

∴ A safe value has to be forwarded to a quorum(`Qf` or `Qc`) so that other
process can see the previous safe value.

## Def-commit

The action commit is to broadcast about what value is **safe**.

## Def-relation

For the order of two interfering instances `a ~ b`,
there are 3 legal relation pairs:

```
a: → b and b: < a   a depends-on b and b doesnot know of a;
a: → b and b: → a   a b depends-on each other;
a: < b and b: → a   a doesnot know of b and b depends-on a;
```

From G-consistency,
`a: < b and b: < a` is not allowed.

## Def-leader

A leader of an instance `a` is the only process that can commit `a`.
This is guaranteed by Ballot: the process Prepare-ed on a quorum with the
highest Ballot is the only one to proceed.

The process that initiates `a` is the default leader of it.
E.g., `La` initiates `a` is leader of `a` then it tries to commit one of `a: → b` or `a: < b`.


## Commit an instance

With this protocol we need to ensure two things about an instance `a` to be **safe**, before committing it:

- What to execute: `a.cmds`.

  To commit `a.cmds`, just simply forward it to a `Qc`,
  because `a.cmds` never changes.

- and the relations of `a` and all other interfering instances.


## Outline of the protocol

- Round-1: a leader, e.g., `La` sends RPC, i.e., Prepare to read the depends-on relation between `a` and all other instances(`a: → x, a: < y, a: → z, ...`) from some replicas.

    If `La` observed that the relations of `a` are safe, commit it.
    Thus this commit with only one round of RPC is called a FastCommit.
    And this procedure is so called FastPath

- If the relations is not safe, continue with Round-2, aka, the SlowPath. On SlowPath, `La` follows the paxos protocol, choose a relation for `a` and then send Accept then commit.


## Def-Prepare

When received Prepare request of `a`,
from Def-relation:
A replica decides the relation of `a, b` by the order `a` and `b`
arrives, if it didnot yet receive a Commit message.

A replica replies `a` do-not-know `b`, i.e., `a: < b` if
`b` does not exist on this replica. Otherwise it has to
reply `b` depends-on `a`, i.e., `b: → a`, to satisfy
G-consistency.

If a replica has received Commit message with `a: → b`,
it replies `b: < a` to Prepare request of `b`, without
breaking G-consistency.

∴ On a replica, the initial relations are exclusive: `a: < b and b: → a` or `a: → b and b: < a`.


## Def-Accept

If the relations `La` received in Round-1 are not safe,
`La` needs another round of RPC to make the value safe.

In Round-2, the leader choose a relation and broadcast it.
The relation it chose must satisfy
Def-relation:
i.e., `a: < b` and `b: < a` can not both chosen.

∴ If `a: → b` is seen, the leader has to choose `a: → b`.

The chosen relation also needs to satisfy Def-safe in order to commit.

∴ The chosen relation have to constitute a quorum: `Qc`, it must
be chosen by any other future leader(recovery).


## Def-recovery

A recovery process re-commits an instance if the previous leader fails(e.g., timeout) to complete the commit.

Our goal is to recover an instance with only `f+1` active replicas.


### Recovery-safety

Recovery have to guarantee that:
an already committed value must be chosen.

Paxos is quite enough to recover an Accept-ed value,
thus this protocol only need to consider recovering a FastCommit-ed value.


## Lemma-FastQuorum-at-least-Qc:

If a value is FastCommit-ed in the Round-1,
the protocol have to guarantee recovery always choose this value.

∴ The FastCommit-ed value must be seen by any other quorum, i.e., A FastQuorum is at least a `Qc`.


## Lemma-FastQuorum-same-value

The values replied from a FastQuorum `Qf` must be all the same.

Proof:

If a recovery process can choose the FastCommit-ed value `v` from a `Qc`,
when it saw some `v` and some other different value `v₁`,
then the other values are not needed.

∴ all values in FastQuorum are all the identical.


## Lemma-entangled-FastCommit

If two interfering instances `a ~ b` are both FastCommit-ed,
the value of `b` can be determined from the value of `a`, i.e., `a: < b ⇔ b: → a`.

Proof:

`a ~ b` are both FastCommit-ed,
from Lemma-FastQuorum-at-least-Qc,
there is at least one replica on which the relation is `a: → b` and `b: < a`.

And from Lemma-FastQuorum-same-value,
`a: → b` is FastCommit-ed implies that `b: < a` is the only value FastCommit-ed and vice versa.


## Lemma-entangled-LT

If `b: < a` is committed, then `a` can only be FastCommit-ed with `a: → b`.

Proof:

If `b: < a` is committed,
from Lemma-FastQuorum-at-least-Qc,
and Def-Accept,
there are at least `|Qc|` replicas that have `b: < a`.

∴ There are at least `|Qc|` replicas that have `b: < a`.
`La` always sees `b: < a`.
Thus from Def-Accept, `La` can only commit `a: → b`.


## Recovery-FastCommit-without-leaders

If a recovery process reached either of leader of `a`(`La`) or leader of `b`(`Lb`),
It is able to find the relation between `a, b` from the leader.

From Lemma-entangled-FastCommit,
We only need to recover the relation when recovery process doesnot reach either of `La` or `Lb`.

This places demands on the definition of FastQuorum:


## Def-FastQuorum

A FastQuorum is a set of replicas to which `La` forwarded `a` and guarantees Recovery-safety.

A FastQuorum requires: `|Qf'(a, b)| > |Qc|/2 + |Qc| - 3`,
where `Qf'(a, b) = Qf(a) \ {La, Lb}`, i.e., `Qf(a)` excluding leader of `a` or `b`.

Proof:

Assumes
`La` and `Lb` have chosen their FastQuorum as `Qf(a)` and `Qf(b)`.
Recovery process has chosen a classic quorum `Qc` for recovery.

From Recovery-FastCommit-without-leaders, assumes `{La, Lb} ∩ Qc = ø`.

To satisfy Recovery-safety, a `Qf` have to satisfies that:
in `Qc`, only one of `a: < b` or `b: < a` can be chosen.

∴ `Qf'(a, b) ∩ Qc` and `Qf'(b, a) ∩ Qc` must have intersection.

I.e., `|Qf'(a, b) ∩ Qc| > |Qc|/2` for any `Qc: {La, Lb} ∩ Qc = ø`. Then we have: `|Qf'(a, b)| - (n-2-|Qc|) > |Qc|/2 = |Qc|/2 + |Qc| - 3`.

E.g.:

- For 5 replicas, `Qc=3`, `|Qf'(a, b)|=2`.
- For 7 replicas, `Qc=4`, `|Qf'(a, b)|=4`.
- For 9 replicas, `Qc=5`, `|Qf'(a, b)|=5`.

∴ With `|Qc| = f+1`, and including `La`,
The minimal FastQuorum size is: `|Qf| >= f + |Qc|/2`.


## Lemma-entangled-GT

In order to recovery a FastCommit-ed value, another guarantee must be met:
If `b: → a` is committed, `a: < b` is the only relation that can be FastCommit-ed.

Proof:

From Def-FastQuorum, `|Qf(a)| = f + |Qc|/2`.
If `Lb ∉ Qf(a)`, a FastCommit-ed value may not be chosen.
To achieve this optimal size of `|Qf(a)| = f + |Qc|/2`, we need to recover `a` from a committed `b`.

From Lemma-entangled-LT, we already have: `b: < a` implies only `a: → b` can be FastCommit-ed.

Then we need that if `b: → a` is committed, `a: → b` can not be FastCommit-ed:
If `La` tries to FastCommit `a: → b`, `La` has to be sure that `b: < a` is committed.
i.e., `La` has seen at least one committed `b: < a`.

With the above constrain, we establish another entangled status:
If `b: → a` is committed, `a: < b` is the only relation that can be FastCommit-ed.


## Lemma-entangled

From Lemma-entangled-FastCommit, Lemma-entangled-LT and Lemma-entangled-GT,
we have that if `a ~ b` and `b` is committed,
`a` can only be FastCommit-ed with the opposite value, i.e.,

- Committed `b: < a` ⇒ `a: → b`
- Committed `b: → a` ⇒ `a: < b`


## FP-condition

Conditions must be satisified to FastCommit `a`:

`a: < x` constitutes a FastQuorum: `|Qf| = f + |Qc|/2`, or:

`a: → x` constitutes a FastQuorum: `|Qf| = f + |Qc|/2` and
the leader received at least one reply with committed `x`.


## Def-Commit-protocol

1. `La` sends Prepare request to `Qf(a)`. So does `Lb`.

2. If `La` received identical replies from `Qf(a)`, e.g.:
   - `a: → b` and `b: < a` is committed,
   - or `a: < b`,
   commit it.

3. Otherwise, send Accept with `a: → b`. Then commit it if received OK replies from some `Qc`.


# Replication workflow

From Def-exec-order,
we use `view` to determine execution order.
With `view`, if `b₁.view != b₂.view`, `b₁, b₂` are different instances.

This pseudo code describes an unoptimized impl:
it commits relations one by one. In practice, relations are batch processed.


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
    accepted: None,
    committed: false,
    view: x.view,
  }

fn update_dep(a, b):
  a.deps[b.id].view ∪= b.view
  if b.committed:
    a.deps[b.id].committed = true
  if b.accepted > a.deps[b.id]:
    a.deps[b.id].accepted = b.accepted

fn union(a, b):
  c = {}
  c.deps.keys = a.deps.keys ∪ b.deps.keys
  for id in c.deps:
    update_dep(c.deps[id], a.deps[id])
    update_dep(c.deps[id], b.deps[id])
  return c
```

## FastPath

1. Leader: Initiate instance `a₀`: fill `a₀.deps` with all local instances that
   interferes with `a₀`.  Then forward `a₀` to all other replicas.

2. NonLeader: handle Prepare request:
   If `req.ballot >= last_blt`, the replica rejects the request and replies
   `last_blt`.
   Otherwise, initiate `a'` the same way the `La` does with `a₀`.
   Then union `a'` and `a₀`: `a₁ = union(a', a₀)`.

   Reply `a₁`.

3. Leader: Handle Prepare replies: For every instance `x` in replied `a.deps`: `x: x ∈ a.deps`, if it satisfies FP-condition: commit `x`. Otherwise enter SlowPath.

## SlowPath

1. Leader: `union()` at least `|Qc|` replies to build `a₂`. Send Accept request with `a₂` to replicas.

2. NonLeader: handle Accept request:
   If `req.ballot >= last_blt`, the replica rejects the request and replies
   `last_blt`.
   Otherwise, store `a₂`.
   Then reply the `last_blt`.

3. Leader: Handle Accept reply: if at least `|Qc|` OK replies are received,
   commit it.


## Replication workflow

```

|                                        |
| Leader                                 | Non Leader
| -------------------------------------- | ---
| fn init(a₀):                           | ---
|   a₀.deps = {}                         |
|                                        |
|   for x in interferings(a₀):           |
|     if a.id in x.deps and x.committed: |
|       continue                         |
|     a₀.deps.insert(x.id, new_dep(x))   |
|                                        |
| init(a₀)                               |
| send_prepare(a₀)                       |
| ---                                    | ---
|                                        | fn handle_prepare_request(req):
|                                        |   if not req.blt >= last_blt:
|                                        |     reply(last_blt)
|                                        |   a₀ = req.a₀
|                                        |   a' = a₀; a'.deps = {}
|                                        |   init(a')
|                                        |   a₁ = union(a', a₀)
|                                        |   reply(a₁)
| ---                                    | ---
| fn handle_prepare_replies(replies):    |
|                                        |
|   dep_ids = {x.id: x ∈ replies[i]}     |
|   for dep_id in dep_ids:               |
|     same = true                        |
|     committed = false                  |
|     r0 = replies[0].deps[dep_id]       |
|     for repl in replies:               |
|        r  = repl.deps[dep_id]          |
|        same = same and r0 == r         |
|        if r.committed:                 |
|          committed = true              |
|                                        |
|     if same:                           |
|       if (r0 == a₀.deps[dep_id]        |
|           or committed):               |
|         commit(a, replies, dep_id)     |
|         continue                       |
|     slowpath(a, replies, dep_id)       |
|                                        |
| ---                                    | ---
| fn slowpath(a, replies, dep_id):       | ---
|                                        |
|   for repl in replies:                 |
|     update_dep(a, repl.deps[dep_id])   |
|   a.deps[dep_id].accepted = my_blt     |
|                                        |
|   send_accept(a, dep_id)               |
| ---                                    | ---
|                                        | fn handle_accept_request(req):
|                                        |
|                                        |   a = req.q; dep_id = req.dep_id
|                                        |   if req.blt >= last_blt:
|                                        |     save(a.deps[dep_id])
|                                        |     last_blt = req.blt
|                                        |     reply(OK)
|                                        |   else:
|                                        |     reply(last_blt)
|                                        |
| ---                                    | ---
| fn handle_accept_replies(replies):     |
|   ok = 0                               |
|   for repl in replies:                 |
|     if repl.ok:                        |
|       ok++                             |
|   if ok >= Qc:                         |
|     commit(a, dep_id)                  |
|                                        |
| ---                                    | ---
| commit(a, dep_id)                      |
|   a.deps[dep_id].committed = true      |
|   send_commit(a, dep_id)               |
```


# Recovery

Assumes:

- The instance to recover is `a`. The recovery process tries to determine the relation of `a` with `b`.
- The recovery process is `P`(`P != La`).

In the following steps,
if conflict ballot is returned in `reply.last_blt`, update local ballot and retry.


## Recovery-1: take leadership

Increment ballot: `blt₁ = blt₀; blt₁[La]++` and send Prepare of `a` with `blt₁` to take leadership from `La`.

If `a` is not on local replica `P` running on, send empty Prepare to a `Qc` to
find out `a` then retry.
If no `a` is found, commit a nil instance on SlowPath and quit.


## Recovery-2: recover SlowCommit-ed instance

After Prepare-ed on a quorum `Qc`,
commit `a` or `b` if it can be committed:

- If `Lb` is seen, wait for it to commit `b`.
- Accept-ed `a` or `b` is seen, choose the one with greatest `blt` and commit it on SlowPath and quit.

If `a` is commited, quit.

If no `a` is seen, commit a nil instance on SlowPath and quit.

If only `b` is committed, from Lemma-entangled to choose the
value of `a` and commit it on SlowPath and quit.


## Recovery-3: recover FastCommit-ed

Otherwise, `a` and `b` can only be FastCommit-ed:
From FP-condition, `a: → b` can be FastCommit-ed only when `b: < a` is committed.

∴ The next step is to determine the first FastCommit-ed: `a: < b` or `b: < a`.

From Def-Prepare, `Lb` wont reply `a: < b` if `b` is not committed.
∴ If `a: < b` is FastCommit-ed, then `Lb ∉ Qf(a)`.

From Def-FastQuorum, a FastCommit-ed value always occupies a majority in any `Qc` if `Qc: Qc ∩ {La, Lb} = ø`.

∴ Choose the one that counts more than `|Qc|/2`, commit it on SlowPath and quit.


## Recovery-4: recover uncommitted instance.

If no value of `a` has more than half of `|Qc|`,
choose the union of all seen value, commit it on SlowPath and quit.


# Execution

See exec-update-accumulated.md

---

Too long; Do not read the following sections.

# Proofs

## Proof: all replica have the same view of committed instance

There is only one value could be chosen to be safe.

∴ finally an instance is committed the same value on all replias.

∴ All replicas have the same set of instances and relations.

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

```
type DepBatch {
    id: InstanceID;
    accepted: Ballot;
    committed: bool;
    view: View;
}
```



## only one of a, x need Accept

optimization maybe:

```
a a a→x a←x a←x x x
```
`a` always view x view a, so a does not need to know x.

## exclude non existent

When handling Prepare, need to exclude instance known but not on local
replica.
This guarantees only need to check the highest interfering instance when
recovery.

No way. `a after b` requires `a` view of all `b` view of.

## Prepare and Prepare together

Prepare and Prepare can be done together
Because Accept only check ballot.
Prepare does not change existent Prepare.


inexistent Prepare-ed value can be treat as existent value but dont know what
it is.

∵ Prepare never change a value.
∴ reading an old value does not need to be protected with Ballot.


## Prepare does not need to check ballot

seeing a Accept-ed instance:
A recovery always choose a possibly FastCommit-ed value.
∴ accept this Prepare request wont let any other one to choose a different value.

As an optimization, respect the accepted value is more efficient.
re-populating an Accept-ed value does not break consistency.


## Instance.deps

`Instance.view` is a virtual attribute and is not need to be persisted on disk.
Because it can be calculated dynamically from `deps`:
`a.view = {a} ∪ a.deps[0].view ∪ a.deps[1].view ∪ ...`


## reduce view to seq


## no deps updated on same leader

From Def-leader-order,
Because newer instance on La always depends-on older instance,
∴ `a.deps[La]` will never change thus do not need to recover it.


# Removed features(from previous version)

## Prepare request do not need to forward commit status of other instances.

If `x` is FastCommit-ed:

- If `a` reached `Lx`, then `a` know if `x` is committed, because `Lx` is the
    first to commit.
    Although there is chance `x` is committed after `a` reaches `Lx`,
    `Lx` broadcasts `x is committed` very likely earlier than another instance
    brings `x is committed` through its Prepare request.

- If `a` did not reach `Lx`, then `a` must have reached `g - {La, Lx}`,
  this prevent other value of `a > y` to commit.
  ∴ `a > x` is safe to FastCommit.

<!-- vi: iskeyword+=-
-->
