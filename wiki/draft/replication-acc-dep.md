<!--ts-->


<!-- Added by: drdrxp, at: Thu Feb 20 19:24:26 CST 2020 -->

<!--te-->

2020 Aug 11

# Goals

Major simplifications with comparison to epaxos:

- Removed infinite strongly-connected-components and livelock from execution algo.
- Removed `seq` thus recovery is significantly simplified.
- Removed unecessary backward depends-on, thus no livelock exists.
- Removed `defered recovery` from recovery algo(If using per-dep commit algo).
- Simplify proof of correctness.

- `deps` now contains instance id and the instance `ver`.

- When updating `deps` for FastAccept of `a`,
    add `x` into `a.deps` only when `x` does not know `a`: `x < a`

- Instances by a same leader has a strong depends-on relation.
  A later instance always depends on a former one.
  This is guaranteed by handling FastAccept request sequentially.

- Use the **all-committed** constrain.
  **all-initial-value** and **only-to-quorum** is not used.

# Terminology

- `R0`, `R1` ... or `R[0]`, `R[1]`... : replica.
- `a`, `b` ... `x`, `y`... : instance.
- `La`, `Lb`: is the leader replica of an instance `a` or `b`.

- `F`: number of max allowed failed replica that.
- `n`: number of replicas, `n = 2F+1`.
- `Qc`: classic quorum: `Qc = F+1`.
- `Qf`: fast quorum: `Qf = F+⌊Qc/2⌋ = F + ⌊(F+1)/2⌋`.

- `a₀`: initial value of instance `a`.
- `a₁ⁱ`: updated instance `a` by `R[i]` when it is forwarded to replica `R[i]`.
- `a₂`: value of instance `a` some relica believes to be safe.

- `a ~ b`: interfere: `a` and `b` can not exchange execution order in any
    instance(command) sequence.
- `a → b`: depends-on: `a` interferes with `b` and has seen `b`.
- `a ..→ b`: indirect depends-on: `a` has seen `b` along a depends-on path: `a
    → x → y ... → b`.
- `a < b`: do-not-know: `a` has not yet seen `b`.
- `a ↦ b`: exec-depends-on: `a` should execute after `b`.

### Def-instance

An instance is an internal representation of a client request.

Two essential fields are:

- `cmds` the commands a client wants to execute.

```
type InstanceID(ReplicaID, i64)

type Seq

type Instance {

    seq:           Seq;
    deps:          Vec<(InstanceID, Seq)>;
    committed:     bool;

    cmds: Vec<Commands>;
    ballot: BallotNum;
}
```

`seq` is something to determine order between instances.
`seq` must be a partial-order relation so that `a` after `b` in time ⇒ `a.seq > b.seq`.

In practice, `seq` can be simply a number and depending instance `seq` is
defined to be 1 plus max of dependent instance `seq`.

Or `seq` can be the set of all instance id an instance has seen, plus itself,
i.e., an instance knows of all instances its dependent instance knows.

`seq` is a virtual attribute and is not necessary to persist on disk.
Because it can be calculated dynamically from `deps`.


### Def-deps

`a.adeps`: is instance id set of all instances that directly or indirectly interfering with `a`.

`adeps` is a virtual attribute and does not need to persist or recvoer it.
Because `a.adeps = {a} ∪ a.deps[0].adeps ∪ a.deps[1].adeps ...`.


`a.deps` is set max intefering instances `adeps` of these instances.

E.g., `a ~ b ~ c` but `a ≁ c`,
`a.adeps = {a, b, c}`
`a.deps = {b}`


On implementation, `a.deps` is split into `N` subset, where `N` is number of replicas.
Every subset contains only instances from leader `Ri`:
`a.deps[Ri] = {x | x.replicaID == Ri and a → x}`.

And `a.deps[Ri]` records only the max instance id.


### Do-not-need-bidirection-knows

> only works for interfering `a, x`, 
> 
> accumulated deps may produce a fake dep: `x→a`:
> 
> ```
> a←b   ==>   a←bₐ
>               b←xₐ   <==   b←x
> R0          R1             R2
> ```


When updating `deps` for FastAccept of `a`,
add `x` into `a.deps` only when `x` does not know `a`: `x < a`.
i.e., if `x → a`, then `a < x`.

Proof:

```
| a a<xₐ  a<xₐ a←x | x x x |
|     x     x      |       |
| Qc               | F     |
```

If `a` is committed with `a < x`, there are at least Qc replicas
`a < x`. `x` commit:

- Any higher ballot must commit with `x→a`.
- All seen ballot can only commit with `x→a`.
- `x < a` is not fast committed, because some process enters Accept phase with
    `x→a`, indicates that FP-condition does not hold with `x < a`.
    FP-condition always choose fast value if it could be FastCommit-ed.

∴ Without bidirection knows, `a, x` has at least one relation committed.

∴ `a < b` always hold if `a, b` is initiated by a same leader and `a ~ b`.


### exclude non existent

When handling FastAccept, need to exclude instance known but not on local
replica.
This guarantees only need to check the highest interfering instance when
recovery.

No way. `a after b` requires `a` knows of all `b` knows of.


### prepare and FastAccept together

Prepare and FastAccept can be done together
Because Accept only check ballot.
FastAccept does not change existent FastAccept.

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

We may write `R[0]` as `R0` for short.


# Definition: commit

The action commit is to broadcast to all replica about what value is **safe**.

## Definition: safe

Some value(e.g. an instance or a relation or something else)
is **safe** if:
it has been forwarded to enough replicas and constituted a quorum(`Qf` or `Qc`)
so that no process(command leader or recovery process) would never choose other
value for it to commit.


## Commit an instance

In this algorithm we need to ensure two things to be **safe**,
before committing it:

- What to execute: `a.cmds`.

  To commit `a.cmds`, forwards it to `Qc=F+1` replicas,
  because `a.cmds` never changes.

- and when to execute: `a.deps`.

  `a.deps` have different values on different replicas.
  This is identical to the problem fast-paxos solved: multi-value-one-round.
  Thus it requires `Qf` replicas to have the identical value to be safe.

### Commit "a.deps"

Since `a.deps` has `n` indepedent fields:

```
a.deps = {
    0: x,
    1: y,
    ...
}
```

- If all `a.deps[Ri]` is safe, `a` is safe.
  Then leader commit it on fast-path.

- Otherwise if any of `a.deps[Ri]` is not safe, run another round of Accept to
  make it safe(slow-path).

### FP-condition

Conditions must be sastisified to commit on fast-path:

- For every updated `a.deps[i] == x`, the leader received at least one reply with
  committed `u` with `a → u` and `u ..→ x`.

- `a.deps[i] == x` constitutes a fast-quorum.

TODO proof:
These two condition guarantees that `x` will never depends on `a`.
This is necessary to recover a fast-committed instance.

## Proof: all replica have the same view of committed instance

TODO obviously

There is only one value could be chosen to be safe.

∴ finally an instance is committed the same value on all replias.

∴ All replicas have the same set of instances.

## Fast path

Leader:

1. Leader: Initiate instance `a`: build `a.deps`:
2. Leader: FastAccept: forward `a` to other replicas.
3. NonLeader: FastAccept: update a with local instances
4. Leader: Handle-FastAcceptReply

## Slow path

Leader:

1. Choose `a.deps`

2. Send Accept to replicas

3. Handle AcceptReply

Non-leader replicas:

1. Handle Accept

## Commit

Just commit.

```
| Leader                                   | Non Leader
| --- init                                 | ---
| a.deps = {}                              |
|                                          |
| for x in local_instances:                |
|     // 1. interferes                     |
|     // 2. TODO                           |
|     if a ~ x and x < a:                  |
|          Lx = leaderOf(x)                |
|          a.deps[Lx] = max(x, a.deps[Lx]) |
|                                          |
| forward(a)                               |
| ---                                      | --- handle-fast-accept-request
|                                          |
|                                          | for x in local_instances:
|                                          |   if not a ~ x:
|                                          |       continue
|                                          |
|                                          |   if x < a:
|                                          |      Lx = leaderOf(x)
|                                          |      a.deps[Lx] = max(x, a.deps[Lx])
|                                          |
|                                          |   if x == a.deps[Lx]:
|                                          |      a.deps[Lx].adeps ∪= x.adeps
|                                          |      a.deps[Lx].committed = x.committed
|                                          |
|                                          | reply(a)
| --- handle-fast-accept-replies           | ---
|                                          |
| committed = [];                          |
| same = true                              |
| for repl in replies:                     |
|    same = same and repl == replies[0]    |
| for i in 0..n:                           |
|    for repl in replies:                  |
|        if repl.deps[i].committed         |
|            committed[i] = true           |
|    if a.deps[i] != replies[0]            |
|       and not committed[i]:              |
|       return slow_path(a)                |
|                                          |
| commit(a)                                |
|                                          |
| --- accept                               | ---
|                                          |
| for repl in replies:                     |
|     for i in 0..n:                       |
|         d = repl.deps[i]                 |
|         if a.deps[i].instance_id < d:    |
|             a.deps[i].instance_id = d    |
|         a.deps[i].adeps ∪= d.adeps       |
|                                          |
| accept(a)                                |
|                                          |
| ---                                      | --- handle-accept
|                                          |
|                                          | save(a)
|                                          |
| --- handle-accept-replies                | ---
|                                          |
| commit(a)                                |
|                                          |
```



# Messages

- All request messages have 3 common fields:

  - `req_type` identify type: FastAccept, Accept, Commit or Prepare.

  - `ballot` is the ballot number,
    - For FastAccept it is always `0`.
    - Fast path Accept ballot is `1`.
    - Slow path Accept ballot is `2` or greater.
        TODO :
    - `ballot` in Commit message is useless.
    - `ballot` in a Prepare is chosen by recovery process and should be
      `>2`.
  - `instance_id` is the instance id this request for.

- All reply messages have 3 common fields:
  - `req_type`.
  - `last_ballot` is the ballot number before processing the request.
  - `instance_id`.

TODO
Changes:

To fast-commit `a > x`:
If `x` is slow-committed, an Accept status `x` will be seen.
Thus `a` can be fast-committed.

If `x` is fast-committed:

- If `a` reached `Lx`, then `a` know if `x` is committed, because `Lx` is the
    first to commit.
    Although there is chance `x` is committed after `a` reaches `Lx`,
    `Lx` broadcasts `x is committed` very likely earlier than another instance
    brings `x is committed` through its fast-accept request.

- If `a` did not reach `Lx`, then `a` must have reached `g - {La, Lx}`,
  this prevent other value of `a > y` to commit.
  ∴ `a > x` is safe to fast commit.

## FastAccept request

- `cmds`: the commands to run.
- `deps`: the deps when leader initiate the instance.

## FastAccept reply

- `deps`: udpated deps by a replica.

## Accept request

- `cmds`: the commands to run.
- `deps`: the deps chosen by leader or recovery process.

## Accept reply

Nothing except the common fileds.

## Commit request

- `cmds`: the commands to run.
- `deps`: the deps chosen by leader or recovery process.

## Commit reply

Nothing except the common fileds.

## Prepare request

Nothing except the common fileds.

## Prepare reply

- `committed` is the committed flag of the instance on a replica.
    TODO 

# Execution

## Guarantees


Order is defined as:

- `a.deps ⊃ b.deps` : exec `a` after `b`.
  From Def-after, if `a.deps ⊃ b.deps`, execute `a` after `b` guarantees
  linearizability.

- Otherwise: exec `a` and `b` in instance id order.


## Execution algorithm

See exec-update-accumulated.md


# Recover

Assumes:

- The instance to recover is `a`.
- The leader of `a` `La` is `R0`
- The recovery process is `P`(`P != R0`).

## Cases not need to recover:

After Preparing on a quorum(`Qc`):

- If `P` saw `R0`, exit and wait for `R0` to commit `a`.

- If `P` saw a committed `a`: broadcast and quit.

- If `P` saw `a` with `ballot>0`: run classic paxos with this
  value and quit.

  TODO explain ballot

∴ `P` only need to recover if all of `a` it saw are in FastAccept phase.

## Recover FastAccept instance

Recovery is to choose a value of `a.deps` that could have been committed on
fast-path.

`P` tries to choose a value for `a.deps[0]`, `a.deps[1]` ... one by one.

First we start to recover `a.deps[1]`.

> `a.deps[La]` is will never change thus do not need to recover it.
TODO 

## Recover one relation

After Prepare on a quorum,
`P` could see different values of `a.deps[1]`(`x`, `y`...) from different replicas.

Assumes that `x > y` and leader of `x`, `y` is `R1`.

- Define `Nx` to be the the number of PrepareReply with `a.deps[1] == x`.
- Define `Ny` to be the the number of PrepareReply with `a.deps[1] == y`.
- ...

As the following diagram shows:

```
       x     ...    a.deps[1]=x    a.deps[1]=y
       y
a      z
---    ---   ...    ---           ---
R0     R1    ...    R2
```

`R1` is unreachable, there could be two possibly committed value of `a.deps[1]`.

E.g.:

```
        x | a→x   a     a
        ↓ |   ↓   |     |
a       y |   y   `→y   `→y
---   --- | ---   ---   ---
R0    R1  | R2    R3    R4
La    Lb
down  down
```

### Lemma-fast-commit-candidate

```
        x | a→u..→x   a          a
          |            ↘          ↘
a       y |       y      v..→y      w..→y
---   --- | -------   --------   --------
R0    R1  | R2        R3         R4
La    Lb
down  down
```

FastCommit requires F+Qc/2 identical FastAccept.
∴ if `u` did not reach `La`, only F+Qc/2-1 `a → u ..→ x` can be committed. Thus
`a ..→ x` can not be FastCommit-ed.
∴ There is at most one value of `a` that may have been FastCommit-ed.
∴ choose the value that has at least `Qc/2+1`.



```
        x | a→u..→x   a          a
          |            ↘          ↘
a       y |       y      v..→y      w..→y
---   --- | -------   --------   --------
R0    R1  | R2        R3         R4
La    Lb
down  down
```

When rerun FastAccept, on R2 it found a new relation `a → u ..→ z`

- If there is a `u` so that `u → a`, and there is no Accept-ed or Commit-ed `u`
    has been seen, `a → u ..→ z` can not be committed, because this recovery
    has to commit with `u → a`.

- If all FastAccept-ed `u < a`, And on replicas(R3, R4) without `a ..→ z` is seen,
  `u ..→ z` can not be committed.



---

-   If no value of `a.deps[1]` could have FastCommit-ed:
    Use any value as initial value to re-run replication algo.

-   Otherwise, recover all instances interfering with `a` and reachable from
    `a.deps`.

    Then re-run replication algo from FastAccept phase.


### Lemma-prepare-fast

Prepare with a new ballot and FastAccept can be combined.


### Acyclic-Defer-recovery

There is no defer-cycle, because defering requires that:
`a` has `> Qc/2` identical values, which requires `>Qc/2` `z` still on these
replicas, otherwise `z` does not have identical value thus can not defer.

And when defering `a, z, .... u` thus `u < a` does not hold.

when found a `u` so that `L(u) == L(a)` , `u` always knows-of `a`.

And on other `< Qc/2` replicas,  defer chain `a, z, u` requires `u<z<a`, this
does not hold.
thus when defer chain goes to a replica twice, defer is ends.


defer chain(old interfering version):
if `L(w) == L(a) w > a`, then no matter what `u` is committed it always knows `a`.
∴ z must knows `a`
```
       w
     u
   z
 ↙ 
a
 ↘ y
```




<!-- vi: iskeyword+=-
-->
