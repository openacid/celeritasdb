<!--ts-->


<!-- Added by: drdrxp, at: Thu Feb 20 19:24:26 CST 2020 -->

<!--te-->

# Goals

- Remove infinite strongly-connected-components and livelock.
- Remove `seq`.
- Remove `defer` during recovery.

Major changes from epaxos:

- When updating `deps`,
    set `deps` to accumulated deps, which includes all reachable instances by
    walking through the dep-graph.

- Removed fast-quorum, use only one quorum `Q=F+1`

- A leader(default leader or recovery leader) forwards only F messages.

- `deps_committed` is removed, fast-commit does not relies on committed flag.
    e.g. FP-condition is removed too.

- `initial_deps` is useless and removed:
    recovery does not need `initial_deps`. only `deps`.

- `final_deps` is useless and removed.


# Terminology

- `R0`, `R1` ... or `R[0]`, `R[1]`... : replica.
- `a`, `b` ... `x`, `y`... : instance.
- `La`, `Lb`: is the leader replica of an instance `a` or `b`.

- `F`: number of max allowed failed replica that.
- `N`: number of replicas, `N = 2F+1`.
- `Q`: quorum: `Q = F+1`.

- `a₀`: initial value of instance `a`.
- `a₁ⁱ`: updated instance `a` by `R[i]` when it is forwarded to replica `R[i]`.
- `a₂`: value of instance `a` some relica believes to be safe.

- `→`: depends-on: `a → b` means `a` depends-on `b`.

### Def-instance

An instance is an internal representation of a client request.

Two essential fields are:

- `cmds` the commands a client wants to execute.
- `deps` what other instance it sees before being committed. This field is to
    determine the instance execution order.

```
type InstanceID(ReplicaID, i64)

type Instance {

    deps:          Vec<InstanceID>;
    committed:     bool;
    executed:      bool;

    cmds: Vec<Commands>;
    ballot: BallotNum;
}
```

- `a.deps`: is instance id set `a` depends-on,
  when `a` is created on leader or
  forwarded to other replica.



### Def-depends-on


**Given two interfering instances `a` and `b`:
`a` depends-on `b`(or `a → b`):
if `a` knows the existence of `b`**.


There could be a cycle along several committed instances:
`a → b → c → a`.
Because an instance may update its own `deps`:
e.g., initially, `a → b → c → ø`, then `c` updates its `deps` with `c → a`, but
`a` and `b` is not updated.


TODO move to where?

## Guarantees

Our algo must meet following guarantees to make consensus:

### G-exec-consistency

Execution consistency:
If two interfering commands `a` and `b` are successfully committed,
they will be executed in the same order by every replica.

### G-exec-finite

Every instance must be executed in finite time, e.g. no livelock with a SCC.

### G-exec-linear

Execution linearizability:

If two instances are serialized by client(`a` is proposed only after `b` is
committed by any replica), then every replica will execute `b` before `a`.


### Def-interfere

For two instances `a, b`,
`a ~ b` if there is an instance sequence `x, ... a, ... b, ...y`, that has
different execution results if exchange position of `a, b`.

### Def-interfering-graph

An indirect graph `Gi` of interfering relations of instances.


### Def-commit

Commit is to broadcast a value to every replica.
A replica always accept a value in a Commit request.


To satisfy G-exec-consistency, two conflicting values must not be both committed.

### Def-safe

A value is safe if no other conflicting value will be chosen for Commit.

∴ two processes choosing conflicting values must perceive the other's choice.

∴ before Commit a value, a process have to broadcast its chosen value.
  And retrieve others chosen value.

- A chosen value must be seen.
    - A chosen value must constitute a quorum `Q`.



- To prevent multiple values to be chosen:
  If two values have been seen, none of them could have been chosen.


Two interfering instance must depends-on other:
→ an instance must be replicate to a quorum to be seen.

### Def-after

TODO only execute committed instance.

To satisfy G-exec-linear, `a` is proposed only after `b` is
committed by any replica

When `b` commit with the interfering-graph `Gi` it saws.

From Def-safe, a committed value `b` constitutes a quorum.
∴ `a` is able to see the safe value of `Gi` of `b`: `b.Gi`.
When `a` is committed, it is committed with a bigger `a.Gi | a.Gi ⊃ b.Gi`.

∴ That if `a.Gi ⊃ b.Gi` then execute `a` after `b`, satisfies
G-exec-linear.

∴ Define `after` to `a` knows all `b` knows.
`a.Gi ⊃ b.Gi`: `a after b`.
And by comparing `x.Gi` of instances the order satisfies G-exec-linear.



### Def-deps

`a.deps`: is defined as: all instances `a` directly or indirectly depends-on, including `a` itself.

- `deps` what other instance it sees before being committed. This field is to
    determine the instance execution order.
- `a.deps`: is instance id set `a` depends-on,
  when `a` is created on leader or
  forwarded to other replica.


### Def-itf-order

To satisfy G-exec-consistency,
two interfering instances must have consistent order on every replica.

This implies the one to execute after should be aware of the early one.

∴ for `a, b`,  before commit, there must be at least one of them knows of the
other, i.e., `a` dont know `b`(`a < b`) and `b` dont know `a`(`b < a`) must not both committed.

**Round-1**:
∴ **Round-1**: the first round
`a` broadcast `a < b`,
`b` broadcast `b < a`,
to a quorum.

When a replica receives an instance, save it locally,
unless there is `b` exists.

If `b` exists, save `a < b+1` and respond negative.

If the leader receives quorum of postive response, Fast-Commit can be done.


### Fast-Quorum
In order to recover,
recovery process must be able to identify if `a < b` is committed.
`a < b` and `b < a` are conflicting values.
∴ there wont be two quorum: Q₁: `a < b`, Q₂: `b < a`: Q₁ ∩ Q and Q₂ ∩ Q
∴ If `La ∉ Q` and `Lb ⊄ Q`, |Q₁ - {a} ∩ Q| + |Q₂ - {b} ∩ Q| > |Q|

```
Q = 5, F = 4, fq = 6
a       | a<x a<x a<x a>x a>x |     x
--- --- | --- --- --- --- --- | --- ---
La                                  Lx
down                                down
```

∴ fq = F + ⌊(F+1) / 2⌋

If `La ⊄ Q` but `Lb ∈ Q`, `Lb` does not provide useful info about which is committed.
We must wait for `b` to commit.
If `b` is committed with `b < a`, then only `a → b` can be committed.
If `b` is committed with `b → a`, then we can not tell which is committed.

```
         | a→x  a←x a←x  |
         |               |
a        |               |
---  --- | ---  ---  --- |
La                   Lx
down down
```




**Ballot**:
Since there could be multiple leaders operating on this instance `a`, leaders
need **Ballot** to identify itself.
And at any time, there must be at most only 1 leader can proceed.

- The initial leader, i.e., that proposed instance `a`, uses `ballot=0`.

- A recovery leader, when the initial leader failed to commit `a`, takes
    leadership by increment Ballot on a quorum of replicas.
 A replica must reject request from an old leader.




As received replies from quorum, leader of `a` union them into the `Gi` to
commit.
Because any seen instance may not know of `a` yet.

**Round-2**: Next, `La` broadcast the union `Gi` to a quorum to make it safe.

**Round-3**: Then commit.


# Optimization

### Fast-Commit

If Round-1 received identical replies from quorum, it could choose to run
Round-3 without Round-2.



# Recovery Leader

### G-recover-consistency
Another leader for recovery must commit the same `Gi` if previous leader already
committed.

**Recovery-1** A recovery leader first broadcast new ballot to a quorum to take leadership so
that old leader can not proceed.

Within this step, a replica saves the new ballot locally if its ballot is
smaller.
Then it responds to the new leader with the instance.

If the recovery leader sees a Committed instance, it just broadcast the
instance.

If it sees an instance received **Round-2**, it choose the one with greatest
ballot and re-run **Round-2** then commit.

If it sees only **Round-1** instance, the system must guarantees that committed
instance can be see.

**Rcv1-msg**:
This requires that in the quorum, if two different instances are seen, no
Fast-Commit can be done.
∴ **Round-1** sends **Round-1** messages to at most `Q` replicas, including the
leader.


∴ the recovery leader can see only one value that could have done Fast-Commit.


Recovery is similar to initial leader replication:

Because it also need to check against others `Gi`, it should
run **Round-1** again on recovery quorum.

After Round-1, recovery leader may discover different value of `a.Gi₁`,
In this case recovery leader have to decide whether the previous value is
committed.

For an instance `z ∈ a.Gi₁` and `z ∈ a.Gi`, assumes leader of `z` is `Lz`.


If there are more than one `a.Gi`,
Run **Round-2** and then commit,

because no other value `z` could
commit without knowing `a`.


If `z ↛ a`, `a.Gi₁` must be committed, because from TODO, two interfering
instance must have on relation.

If `Lz` is not seen:
∴ `z ↛ a`, `a.Gi` must not be committed. Because Lz does not know `a`.
In this case, commit `a → z`.

∴ disallow `Lz` to accept `z → a`
∴ In Round-1, if `x → a`, forbid `a → x`. TODO elaborate it.

If `z → a`, `a` does not need to have `z` in its `Gi`,
in this case to commit `a.Gi` is safe.
∴ for a replica: if `z → a`, should not update with `a → z`.


If `Lz` is seen:





## new

Replicate:
`a` replicates that `a` does not know `x`: `a < x`.
`x` replicates that `x` does not know `a`: `x < a`.











Since the order is determined by `a.deps`.

∴ `a.deps` must be safe to be committed.


From Def-after, interfering `a` `b` must see each other


`a` proposed after `b` must reaches a quorum to perceive the safe value `b`.

∴ first round to contact `a` to a quorum, to see what instances there are and
what they knows.

Then choose value `a` knows to be union of the response.

Broadcast the value to a quorum to make it safe.

But two process may

record what `a` knows of, and
respond it to leader.






From Def-itf-order, two interfering

FastAccept requires a → x and a ↛ x to be exclusive
∴ FastAccept request can not be handled twice, or two process may believes their
value constituted a quorum.
∴ Since `a.deps` use only the max id to describe deps, FastAccept of older instance must be handled before newer ones.
Otherwise, for a replica it feels like handled an older instance twice.

If `x → a`,

TODO use baohai's  exec-algo requires newer instance depends-on older one?
even if they do not interfere.



### Def-replciation-order

Two interfering instances by a same leader must be handled in FIFO order on a
replica.

E.g. if `b` is replciated before `a`, `a` and `x` finally has the same `deps`,
thus there is no way to tell which is earlier:

```
b
a               x
---    ---    ---
R0     R1     R2

b----> b
a               x
---    ---    ---
R0     R1     R2

b      b
a        x <----x    // R1: x→{b, x}
---    ---    ---
R0     R1     R2

b      b
a----> a x      x    // R1: x→{b, x}, a→{b, x}
---    ---    ---
R0     R1     R2
```

Processing instances in FIFO order is simple in impl.

# Instance relation

Two interfering instances `a` and `b` has one of two relation:
`a→b` or `a↛b`.
The Same for `b`.

Thus there 3 relation between `a` and `b`:
`a→b`, `b→a`, `a↔b`.





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


## Examples of depends-on

- Initially, there are 3 instances `x, y, z`.

  When `a` is initiated on `R0`, `a` depends-on all others:
  `a₀ → {x, y, z}`.

  When `b` is initiated on `R1`, `b` depends-on all others:
  `b₀ → {x, y, z}`.

  When `c` is initiated on `R2`, `c` depends-on all others:
  `c₀ → {x, y, z}`.

  When `d` is initiated on `R0`, `d` depends-on all others:
  `d₀ → {a, x, y, z}`.

  ```
  d
  ↓
  a            b            c
  x y z      x y z      x y z
  -----      -----      -----
  R0         R1         R2
  ```

### Simple case:

When `d` is replicated to `R1`,
`R1` believes that `d₁¹ → {a, b, x, y, z}`.

`d₁¹` got a new relation `d₁¹ → b`:

```
d          d
↓           ↘
a            b            c
x y z      x y z      x y z
-----      -----      -----
R0         R1         R2
```

### Transitivity:

Then `c` is replicated to `R1`,
`R1` believes that `c₁¹ → {d, a, b, x, y, z}`.

`c₁¹` got three new relations `c₁¹ → {b, d, a}`(
because `R1` believes `d → a` thus `c₁¹ → a`):

```
              .c
            ↙  |
d          d   |
↓           ↘ ↙
a            b            c
x y z      x y z      x y z
-----      -----      -----
R0         R1         R2
```

### Not to override existent replation:

Then `a` is replicated to `R1`,
`R1` believes that `a₁¹ → {b, x, y, z}`.

`a₁¹` got only one new relation `a₁¹ → b`:
`R1` already believes `d₀ → a` because it had received `d₀` from `R0`.
`c₁¹ → d` thus `c₁¹ → a`.

```
              .c
            ↙  |
d          d   |
↓          ↓↘ ↙
a          a→b            c
x y z      x y z      x y z
-----      -----      -----
R0         R1         R2
```

### Transitive-2: update `deps` with unknown instances

Starts with a new initial setup:

```
d
↓↘
a c                       b
x y z      x y z      x y z
-----      -----      -----
R0         R1         R2
```

After forwarding `d` to `R1`:
`d₁¹ = d₀ → {a, c, z}`

```
d          d
↓↘
a c                       b
x y z      x y z      x y z
-----      -----      -----
R0         R1         R2
```

Then `b` is forwarded to `R1`:

`b` did not see `a` and `c`,
but `b` still updates with three new relations:
`ḇ₁¹ → {d, a, c}`.
Because `d → {a, c}` and `deps` is transitive.

```
               b
             ↙
d          d
↓↘
a c                       b
x y z      x y z      x y z
-----      -----      -----
R0         R1         R2
```

We see that different replicas have their own view of instance relations.

## Property: antisymmetric
TODO

- On a replica,
  If `a → b` has been seen, then `b → a` does not hold.

- On a replica,
  `a > a` never holds.

## Property: transitivity

On a replica,
`a → b` and `b → c` implies `a → c`.

# Definition: attribute deps

- `a.deps`: is instance id set when `a` is created on leader.
  when `a` is forwarded to other replica, it is updated instnce id set.

On a replica:
`a.deps` is all instances that `a` depends-on:
`a.deps = {x | a → x}`.

On implementation,
`a.deps` is split into `N` subset,
where `N` is number of replicas.
Every subset contains only instances from leader `Ri`:
`a.deps[Ri] = {x | x.replicaID == Ri and a → x}`.

### Properties of attribute deps
TODO replace this section with definition of `after`

On a replica:

- `a → b` implies `a.deps ⊃ b.deps`.

- Thus `a.deps ⊂ b.deps` then `a < b` does not hold.


### Implementation

`a.deps[i]` stores only the max instance id in it(that is why FastAccept must
be handled in instance id order, otherwise recording only the max instance id
includes more instances),

because an instance is **after** all preceding instances by the same leader.

# Definition: commit

The action commit is to broadcast to all replica about what value is **safe**.


### Example: safe relation

`a` is safe if every `a.deps[Ri]` is safe.

```
       a₁¹    a₁²    a₁³
       |      |      |↘
       |      |      | c₁³
       ↓      ↓      ↓
a₀     b₀     c₀     b₀
---    ---    ---    ---    ---
R0     R1     R2     R3     R4
```

```
a₁¹.deps = {b}
a₁².deps = {c}
a₁³.deps = {b, c}
```

Thus `a.deps = {b, c}` can be committed.

## Commit an instance

In this algorithm we need to ensure two things to be **safe**,
before committing it:

- What to execute: `a.cmds`.

  To commit `a.cmds`, forwards it to `Q=F+1`
  replicas,
  because `a.cmds` never changes.

- and when to execute: `a.deps`.

  `a.deps` have different values on different replicas.
  Thus it requires `Q` replicas to have the identical value to be safe.

### Commit "a.deps"

Since `a.deps` has `N` indepedent fields:

```
a.deps = {
    0: x,
    1: y,
    ...
}
```

- If every `a.deps[Ri]` is safe, `a` is safe.
  Then leader commit it on fast-path.

- Otherwise if any of `a.deps[Ri]` is not safe, run another round of Accept to
  make it safe(slow path).

## Proof: all replica have the same view of committed instance

the value of two interfering instance `a→b`, can only be commit when it is safe.
A safe value requires a quorum.
Two quorums must have at least one common replica.
There is only one value could be chosen to be safe.
∴ no two different value, e.g., `a→b` and `a↛b` could be both committed.

∴ finally an instance is committed the same value on all replias.

∴ All replicas have the same set of instances.

## Fast path

Leader:

1. Initiate instance `a`

   build `a.deps`:

   ```
   max_known_instance_id[leaderOf(a)] = a
   // N is the number of all leaders
   for l in (0..N):
       a.deps[l] = max_known_instance_id[l];

   ```

2. FastAccept: forward `a` to other replicas.

3. Handle-FastAcceptReply

   Update `a.deps`:

   ```
   for i in 0..N:
       values = {a.deps[i] for a in all_replies}

       // received different values.
       if (count(v[0], values) != Q-1):
           return quit_fast_path()

   commit(a)
   ```

Non-leader replicas:

1. Handle-FastAccept

   If FastAccept is already handled, ignore all future FastAccept request.

   TODO need proof of linearizability with this.
   TODO explain why this is efficient reducing conflict.

   TODO allow or not allow backward depends-on is an option.
   By disallowing backward depends-on, baohais's exec algo would work.

   update `a.deps'`.

   > committed flag are ignored in this pseudo code for clarity

   ```
   for x in all_instances_on_this_repilca:

       if (x ~ a):
           l = leaderOf(x)
           a.deps[l] = max(x, a.deps[l])

   reply(a)
   ```

## Slow path

Leader:

1. Choose `a.deps`

2. Send Accept to replicas

3. Handle AcceptReply

Non-leader replicas:

1. Handle Accept

## Commit

Just commit.

# Messages

- All request messages have 3 common fields:

  - `ballot` is the ballot number,

    - the leader of an instance use `ballot=0`.

    - the recovery process use `ballot > 0`.

      A recovery process is actually another leader that takes leadership by
      increment ballot.

    - `ballot` in Commit message is useless.

  - `instance_id` is the instance id this request for.

- All reply messages have 3 common fields:
  - `last_ballot` is the ballot number before processing the request.
  - `instance_id`.


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

- an instance.

    TODO

# Execution

## Guarantees


Order is defined as:

- `a.deps ⊃ b.deps` : exec `a` after `b`.
  From Def-after, if `a.deps ⊃ b.deps`, execute `a` after `b` guarantees
  linearizability.

- Otherwise: exec `a` and `b` in instance id order.


## Execution algorithm

In the following digram, `a.deps ⊃ d.deps` thus `a` should be executed after
`d`.
`b` and `e` interferes but `b.deps ⊅ e.deps` or `e.deps ⊅ b.deps`.
They could be executed in any order that is identical on every replica.

```
        a      b
          ↘ ↙   ~
     c  ~  d  ~  e
      ↘   ↙ ↘
        f    g
```

### Algo-1

One general exec algo is by walking the depends-on graph, remove some edges to reduce the graph to a DAG and instances have determined order to execute.

See other doc TODO

### Algo-2

Another exec algo is much simpler to proof correctness but requires additional
constrains: for instances on a leader, a newer instance must be executed after an older instance.

Our replication algo
One of the constrain must be applied to replication:

- A replica must handle older instance before handling a newer one.
- Or the `deps` of an older instance must not include `deps` of the newer instance, when handling FastAccept.

Either one of the above guarantees `newer.deps ⊃ older.deps`.

See other doc TODO


# Recover

Assumes:

- The instance to recover is `a`.
- The leader of `a` `La` is `R0`
- The recovery process is `P`(`P != R0`).

## Cases not need to recover:

After Preparing on a quorum(`Q=F+1`):

- If `P` saw `R0`, exit and wait for `R0` to commit `a`.

- If `P` saw a committed `a`: broadcast and quit.

- If `P` saw `a` with `ballot>0`: run classic paxos with this
  value and quit.

  TODO explain ballot

∴ `P` only need to recover if all of `a` it saw are in FastAccept phase.

## Recover FastAccept instance

Recovery is to choose a value of `a.deps` that could have been committed on
fast-path.

Choose only the values with highest ballot seen.

`P` tries to choose a value for `a.deps[0]`, `a.deps[1]` ... one by one.

Assumes we start to recover `a.deps[1]`.

## Recover one relation

After Prepare on a quorum,
`P` may see different values of `a.deps[1]`, e.g., `x`, `y`... on different replicas.

As the following diagram shows:

```
       x     ...    a.deps[1]=x    a.deps[1]=y
a      y
---    ---   ...    ---           ---
R0     R1    ...    R2
```

### Choose the value that could have been committed.

∵ Leader sends exactly `F` FastAccept message(including the leader, at most `Q=F+1` replica deps this instance).

∴ If there are two different value of `a.deps[1]`, `a` can not fast-commit.

∴ `P` choose the only value seen or the highest value.

### Determine the value for recovery


- If `Lx` is not reached:

    ```
           x | a→x            |
           ↓ |   ↓            |
    a      y |   y            |
    ---  --- | ---   ---  --- |
    R0   R1  | R2    R3   R4  |
    La   Lx
    down down
    ```

    Use the the instance id `P` chosen as an initial `deps` to run FastAccept with the recovery ballot, e.g., `ballot=1` to recovery quorum.

    If new `deps` `z` by leader `Lx` is found:

    If there is `z→a`, then Lx has `z→a` then there wont be `z↛a` exists.
    Commit `a→x`.

    Otherwise, on `Lx` there must be `z↛a`, which means `a→x` is not
    committed because there is not enough quorum.
    Then run Accept and Commit with `a→z`.

    This is the same as the replication procedure on the leader, except ballot
    is not 0.

    TODO proof recovery from a recovery.


- If `Lx` is reached:

    ```
         |   x  a→x       |
         |   ↓    ↓       |
    a    |   y    y       |
    ---  | ---  ---   --- |  ---
    R0   | R1   R2    R3  |  R4
    La     Lx
    down                     down
    ```

    If new `deps` `z` by leader `Lx` is found:

    wait z to commit,
    if z is committed with `z→a`, commit `a→x`.
    if z is committed with `z↛a`, there is an unreachable replica does not have
    `a`, which means `a→x` is not fast-committed. Accept and Commit `a→z`.


## Recover the instance

Collect all recovered values of `a.deps[i]` and run Accept and Commit.
