<!--ts-->

- [Goals](#goals)
- [Terminology](#terminology)
- [Definition: instance](#definition-instance)
- [Definition instance space](#definition-instance-space)
  - [instance space layout](#instance-space-layout)
- [Definition: depends on](#definition-depends-on)
  - [Examples of relation depends-on](#examples-of-relation-depends-on)
    - [Simple case:](#simple-case)
    - [Transitive:](#transitive)
    - [Not to override existent replation:](#not-to-override-existent-replation)
    - [Transitive-2: update deps with unknown instances](#transitive-2-update-deps-with-unknown-instances)
  - [Property: antisymmetric](#property-antisymmetric)
  - [Property: transitivity](#property-transitivity)
- [Definition: attribute deps](#definition-attribute-deps)
  _ [Properties of attribute deps](#properties-of-attribute-deps)
  _ [Implementation](#implementation)
- [Definition: commit](#definition-commit)
  - [Definition: safe](#definition-safe)
    - [Example: safe](#example-safe)
  - [Commit an instance](#commit-an-instance)
    - [Commit "a.deps"](#commit-adeps)
    - [FP-condition](#fp-condition)
  - [Proof: all replica have the same view of committed instance](#proof-all-replica-have-the-same-view-of-committed-instance)
  - [Fast path](#fast-path)
  - [Slow path](#slow-path)
  - [Commit](#commit)
- [Execution](#execution)
  - [Execution order](#execution-order)
  - [Guarantees:](#guarantees)
  - [For interfering instances:](#for-interfering-instances)
  - [For non-interfering instances:](#for-non-interfering-instances)
  - [Proof](#proof)
- [Execution algorithm](#execution-algorithm)
- [Recover](#recover)
  - [Cases not need to recover:](#cases-not-need-to-recover)
  - [Recover PreAccept instance](#recover-preaccept-instance)
  - [Recover one relation](#recover-one-relation)
    - [Case-1: R1 is unreachable, there could be two possibly committed value of a.deps[1].](#case-1-r1-is-unreachable-there-could-be-two-possibly-committed-value-of-adeps1)
      - [Lemma-1: R1 does not have a &lt; x on it](#lemma-1-r1-does-not-have-a--x-on-it)
      - [Lemma-2: R0 does not have a &gt; x on it.](#lemma-2-r0-does-not-have-a--x-on-it)
      - [Lemma-3: x could only have been committed on fast path with x &gt; a:](#lemma-3-x-could-only-have-been-committed-on-fast-path-with-x--a)
      - [slow-committed value of x](#slow-committed-value-of-x)
    - [Case-2: R1 is unreachable, only one possibly committed value of a.deps[1].](#case-2-r1-is-unreachable-only-one-possibly-committed-value-of-adeps1)
    - [Case-3: R1 is reached.](#case-3-r1-is-reached)

<!-- Added by: drdrxp, at: Fri Feb  7 15:22:25 CST 2020 -->

<!--te-->

# Goals

- Remove infinite strongly-connected-components
- Remove `seq`.
- Remove `defer` during recovery.

Major changes from epaxos:

- When updating `deps`, only check against PreAccept phase values:
  `deps` updated by Accept or Commit is ignored.

- Instances by a same leader has a strong depends-on relation.
  A later instance always depends on a former one.

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

- `>`: depend-on: `a > b` means `a` depends on `b`.

# Definition: instance

Instance: an internal representation of a client request.

```
type InstanceID(ReplicaID, i64)

// another bool indicates if the instance is committed.
type Dep(InstanceID, bool)

type Instance {

    initialDeps:   Vec<Dep>;
    deps:          Vec<Dep>;
    final_deps:    Vec<Dep>;
    status:        "committed|acceptted|preaccepted";

    cmds: Vec<Commands>;
    ballot: BallotNum; // ballot number.
}
```

An instance has 4 attributes for `deps`:

- `a.initialDeps`: is instance id set when `a` is created on leader.
- `a.deps`: when `a` is created it is same as `a.initialDeps`.
  when `a` is forwarded to other replica through PreAccept,
  it is updated instnce id set.

- `a.final_deps` is `deps` updated by Accept or Commit.

On a replica:
`a.deps` is all instances that `a` is after:
`a.deps = {x | a > x}`.

On a replica:
for instance `a`,
`a.deps` is a set of **instance-ids** that **should** be executed before `a`.

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

# Definition: depends on

**depends on** is a local relation between instances on a replica.

**two instances `a` and `b`:
`a` depends on `b`(or `a > b`):
if `a` is after `b` in time**.

`a ≯ b` means `a` is not after `b` in time.

From definition, we infer that:

- On a replica,
  any two instances `a` and `b` must have one of `a > b` or `b > a`.

- When a leader initiates an instance `a`, `a` depends on all existent instances.
  Because none of existent instances have an `depends-on` relation with `a`.

  > committed flag are ignored in this pseudo code for clarity

  ```
  a.deps  = a.initialDeps = all_instances_on_this_repilca
  ```

- When a replica receives PreAccept of `a`,
  it updates `a` to depend on
  all those do not have a relation `>` with `a`

  > committed flag are ignored in this pseudo code for clarity

  ```
  for x in all_instances_on_this_repilca:
      if not x > a:
          update a.deps with `a > x`
  ```

## Examples of relation depends-on

- Initially, there are 3 instances `x, y, z`.

  When `a` is initiated on `R0`, `R0` believes it is after all others:
  `a₀ > {x, y, z}`.

  When `b` is initiated on `R1`, `R1` believes it is after all others:
  `b₀ > {x, y, z}`.

  When `c` is initiated on `R2`, `R2` believes it is after all others:
  `c₀ > {x, y, z}`.

  When `d` is initiated on `R0`, `R0` believes it is after all others:
  `d₀ > {a, x, y, z}`.

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
`R1` believes that `d₁¹ > {a, b, x, y, z}`.

`d₁¹` got a new relation `d₁¹ > b`:

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
`R1` believes that `c₁¹ > {d, a, b, x, y, z}`.

`c₁¹` got three new relations `c₁¹ > {b, d, a}`(
because `R1` believes `d > a` thus `c₁¹ > a`):

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
`R1` believes that `a₁¹ > {b, x, y, z}`.

`a₁¹` got only one new relation `a₁¹ > b`:
`R1` already believes `d₀ > a` because it had received `d₀` from `R0`.
`c₁¹ > d` thus `c₁¹ > a`.

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
`d₁¹ = d₀ > {a, c, z}`

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
`ḇ₁¹ > {d, a, c}`.
Because `d > {a, c}` and `deps` is transitive.

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

- On a replica,
  If `a > b` has been seen, then `b > a` does not hold.

- On a replica,
  `a > a` never holds.

## Property: transitivity

On a replica,
`a > b` and `b > c` implies `a > c`.

# Definition: attribute deps

An instance has 4 attributes for `deps`:

- `a.initialDeps`: is instance id set when `a` is created on leader.
- `a.deps`: when `a` is created it is same as `a.initialDeps`.
  when `a` is forwarded to other replica, it is updated instnce id set.

- `a.final_deps` is `deps` updated by Accept or Commit.

On a replica:
`a.deps` is all instances that `a` is after:
`a.deps = {x | a > x}`.

And `a.deps` is split into `n` subset,
where `n` is number of replicas.
Every subset contains only instances from leader `Ri`:
`a.deps[Ri] = {x | x.replicaID == Ri and a > x}`.

### Properties of attribute deps

On a replica:

- `a > b` implies `a.deps ⊃ b.deps`.

- Thus `a.deps ⊂ b.deps` then `a < b` does not hold.

### Implementation

`a.deps[i]` stores only the max instance id in it,
because an instance is **after** all preceding instances by the same leader.

# Definition: commit

The action commit is to broadcast to all replica about what value is **safe**.

## Definition: safe

Some value(e.g. an instance or a relation or something else)
is **safe** if:
it has been forwarded to enough replicas and constituted a quorum(`Qf` or `Qc`)
so that no process(command leader or recovery process) would never choose other
value for it to commit.

### Example: safe

`a` is safe if every `a.deps[Ri]` is safe.

```
       a₁¹    a₁²    a₁³
       |      |      ↓
       |      |      c₁³
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

  To commit `a.cmds`, forwards it to `Qc=F+1`
  replicas,
  because `a.cmds` never changes.

- and when to execute: `a.deps`.

  `a.deps` have different values on different replicas.
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
  make it safe(slow path).

### FP-condition

Conditions must be sastisified to commit on fast-path:

- For every updated `a.deps[i] == x`, the leader received at least one reply with
  committed `x` and `x < a`.

- `a.deps[i] == x` constitutes a fast-quorum.

These two condition guarantees that `x` will never depends on `a`.
This is necessary to recover a fast-committed instance.

## Proof: all replica have the same view of committed instance

TODO obviously

There is only one value could be chosen to be safe.

∴ finally an instance is committed the same value on all replias.

∴ All replicas have the same set of instances.

## Fast path

Leader:

1. Initiate instance `a`

   build `a.deps`:

   > committed flag are ignored in this pseudo code for clarity

   ```
   for x in all_instances_on_this_repilca:
       Lx = leaderOf(x)
       a.deps[Lx] = max(x, a.deps[Lx])

   a.initialDeps = a.deps
   ```

2. PreAccept: forward `a` to other replicas.

3. Handle-PreAcceptReply

   There are two step of PreAcceptReply:

   If `a` has some committed instances in `a.deps`,
   update the commit status to local instance-space.

   Thus there would be some empty slot in instance space has a committed
   status.

   The second step is to update `a.deps`:

   ```
   a.commitDeps = [];
   for i in 0..n:
       values = {a.deps[i] for a in all_replies}
       for v in values:
           if (count(v, all_replies) >= fast_quorum
                   and v is committed):

               a.commitDeps[i] = v
               break
       if a.commitDeps[i] is NULL:
           quit fast-path.

   commit(a)
   ```

Non-leader replicas:

1. Handle-PreAccept

   TODO need proof of linearizability with this.
   TODO explain why this is efficient reducing conflict.

   update `a.deps'`

   > committed flag are ignored in this pseudo code for clarity

   ```
   for x in all_instances_on_this_repilca:
       Lx = leaderOf(x)
       if (not x.deps[Lx] > a
               and (
                   x ~ a
                   or x is committed
               )):

           a.deps[Lx] = max(x, a.deps[Lx])

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
    - `req_type` identify type: PreAccept, Accept, Commit or Prepare.

    - `ballot` is the ballot number,
        - For PreAccept it is always `0`.
        - Fast path Accept ballot is `1`.
        - Slow path Accept ballot is `2` or greater.
        - `ballot` in Commit message is useless.
        - `ballot` in a Prepare is chosen by recovery process and should be
            `>2`.
    - `instance_id` is the instance id this request for.

- All reply messages have 3 common fields:
    - `req_type`.
    - `last_ballot` is the ballot number before processing the request.
    - `instance_id`.


## PreAccept request

- `cmds`: the commands to run.
- `initial_deps`: the deps when leader initiate the instance.
- `deps_status`: a vector of committed flag of every instance in `initial_deps`

## PreAccept reply

- `deps`: udpated deps by a replica.
- `deps_status`: a vector of committed flag of every instance in `deps`

## Accept request

- `final_deps`: the deps chosen by leader or recovery process.

## Accept reply

Nothing except the common fileds.

## Commit request

- `cmds`: the commands to run.
- `final_deps`: the deps chosen by leader or recovery process.

## Commit reply

Nothing except the common fileds.

## Prepare request

Nothing except the common fileds.

## Prepare reply

- `status` is the status of the instance on a replica.

# Execution

## Execution order

On a replica, `a.deps` represents local relation between `a` and other instances
on this replia. There is not circle, e.g.: `a > b > c > a` on a replica.

But `a.final_deps` could have.

E.g. initially:

```
a₀        b₀
-----  -----
R0     R1
```

`R0` and `R1` forward `a` and `b` to the other:

```
   b₁⁰  a₁¹
 ↙       ↘
a₀        b₀
-----  -----
R0     R1
```

Finaly `a.final_deps = {b}`,
`b.final_deps = {a}`.

For this reason execution order is determined not by relation `>`,
but by comparing the vecotr `a.final_deps`.

TODO

## Guarantees:

- Execution consistency:
  If two interfering commands `a` and `b` are successfully committed,
  they will be executed in the same order by every replica.

- Execution linearizability:
  If two instance
  TODO proof

## For interfering instances:

- `a.final_deps ⊃ b.final_deps` : exec `a` after `b`
- `a.final_deps ⊅ b.final_deps` and `a.final_deps ⊄ b.final_deps` : exec `a` and `b` in instance id
  order.
- there is no `a` and `b` have `a.final_deps == b.final_deps`.

## For non-interfering instances:

- `a.final_deps ⊃ b.final_deps` : exec `a` after `b`
- `a.final_deps == b.final_deps`: exec `a` and `b` in instance id
  order.
- `a.final_deps ⊅ b.final_deps` and `a.final_deps ⊄ b.final_deps` : exec `a` and `b` in instance id
  order.

## Proof

If `a` is initiated after `b` became safe,
`b` will never see `a`, and `a` will definitely see `b`.
Then `b` will be added into `a.deps`.

Thus `a.deps ⊃ b.deps`

∴ execute `a` after `b` will never break guarantees.

# Execution algorithm

Guarantees:

- Consistency: execution order must be the same on all replicas.

- Every instance must be executed in finite time.

- If there is `x: a ⊃ x` then `x` must be executed before `a`

```
        a      b
          ↘ ↙   ~
     c  ~  d  ~  e
      ↘   ↙ ↘
        f    g
```

TODO

# Recover

Assumes:

- The instance to recover is `a`.
- The leader of `a` `La` is `R0`
- The recovery process is `P`(`P != R0`).

## Cases not need to recover:

After Preparing on a quorum(`F+1`):

- If `P` saw `R0`, exit and wait for `R0` to commit `a`.

- If `P` saw a committed `a`: broadcast and quit.

- If `P` saw `a` with `ballot>0`: run classic paxos with this
  value and quit.

  TODO explain ballot

∴ `P` only need to recover if all of `a` it saw are in PreAccept phase.

## Recover PreAccept instance

Recovery is to choose a value of `a.deps` that could have been committed on
fast-path.

`P` tries to choose a value for `a.deps[0]`, `a.deps[1]` ... one by one.

First we start to recover `a.deps[1]`.

> `a.deps[La]` is will never change thus do not need to recover it.

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

### Case-1: `R1` is unreachable, there could be two possibly committed value of `a.deps[1]`.

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

In this situation, both `a.deps[1] == x` and `a.deps[1] == y` could have been committed
on fast-path.

#### Lemma-1: `R1` does not have `a < x` on it

∵ `a.deps[1] = x` has been seen.

∴ Initially, `a < x` is not on `R1` and it will never be.

> But `R1` could have `a > x` on it.

#### Lemma-2: `R0` does not have `a > x` on it.

∵ `a.deps[1] = y` has been seen.

∴ Initially, `a > x` is not on `R0` and it will never be.

> But `R0` could have `a > y` on it.

There are `F + Nx` replicas
accepted or could have accepted `a.deps[1] == x`.
`F` unreached replicas plus `Nx`.

There are `F -1 + Ny` replicas
accepted or could have accepted `a.deps[1] == y`.
`F` unreached replicas except `R1` plus `Ny`.

If both of these two constituted fast-quorum(`F + ⌊(F+1)/2⌋)`), it requires:

- `Nx >= ⌊(F+1)/2⌋`
- `Ny >= ⌊(F+1)/2⌋ + 1`
- `Nx + Ny <= F + 1`

Thus if `F=2k` there could be two value possibly committed on fast-path:

```
Nx = ⌊(F+1)/2⌋ = k
Ny = ⌊(F+1/2)⌋ + 1 = k + 1
```

E.g.:

- `R0, R1, R2` could have constituted a fast-quorum for `a.deps[1] = x`.
- `R0, R3, R4` could have constituted a fast-quorum for `a.deps[1] = y`.

`P` needs to eliminate one of them

- `a` did not see a fast-committed `x`

If `a.deps[1] == x` is committed,
by [fast-commit requirements](#fast-commit-requirements),
the committed `x.deps` must not contain `a`(`x < a`)

#### Lemma-3: `x` could only have been committed on fast path with `x > a`:

∵ `Nx = ⌊(F+1)/2⌋`.

∴ There are at most `F - 1 + Nx = F + ⌊(F+1)/2⌋ - 1 < F + ⌊(F+1)/2⌋` replicas accepts `x < a` in PreAccept phase:

> `F` unreached replicas except `R0`, plus `Nx`.

∴ `x < a` can not constitute a fast-quorum.

∴ If `x` is fast-committed, it must be committed with `x > a`.

#### slow-committed value of `x`

Prepare on `x`.

If `P` saw a Accepted `x`,
run classic paxos and commit `x`.

- If `P` saw a committed `x`:

  - If `x > a`, From FP-condition,
    `a.deps[1] == x` could not have been committed on fast-path.

    Discard `a.deps[1] == x`, try other value of `a.deps[1]`.

  - If `x < a`, there is a at least classic-quorum on which
    `x` came before `a`.

    ∴ `x` must be in `a.deps` when `a` is committed.

    ∴ `a.deps[1] == x` is the only possible value to commit.

- Otherwise, `x` is not committed on slow-path.

  From Lemma-3:
  `a.deps[1] == x` can not be committed on fast-path.

  Discard `a.deps[1] == x`, try other value of `a.deps[1]`.

Continue checking if `a.deps[1] == y` can be committed on fast-path, and so on.
If no value of `a.deps[1]` could have been committed, use the initial value:
`a.initialDeps[1]`.

### Case-2: `R1` is unreachable, only one possibly committed value of `a.deps[1]`.

Choose the only value to commit `a.deps[1]`.

### Case-3: `R1` is reached.

E.g.:

```
     |   x  a→x   a   |
     |   ↓    ↓   |   |
a    |   y    y   '→y |
---  | ---  ---   --- |  ---
R0   | R1   R2    R3  |  R4
La     Lb
down                     down
```

If `x` is committed,

- If `x < a`, then only `a.deps[1] == x` can be committed.
  Because if `a.deps[1] == y` is committed, `a` and `x` has no relation, which is
  impossible.

- If `x > a`, then `a.deps[1] == x` can not be committed.
  Because on `R2` `x` does not have `x > a`, this is not a committed value of
  `x`.

If `x` is not committed, wait until it is committed.

Continue repeat these step on `a.deps[1] == y` to choose a value.
