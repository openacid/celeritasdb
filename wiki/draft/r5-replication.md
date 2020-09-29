# 666paxos

666paxos, aka `6p` is an optimization for at most 6 replicas, 
with only one RTT to achieve consensus, and allows at most 2 failure replicas.

In this paper the discussion is in a 5 replicas scenario.

# Guarantees

- Every two instances interfere with each other.

- Consistency.

- Local-Linearizability: If instance `b` is proposed after `a` is committed, `b`
    is always executed after `a`, if `Lb == La`.

    **This is different from epaxos**.
    A single round commit can not achieve global ordering.
    And this is similar to the reality: according to relativity, two events happens at two place has no absolute order.

    To achieve global ordering, another round is required to broadcast the
    committed value to a quorum, if the committed value is different from the
    initial value.

- Safety.

# Terminology

- `R0`, `R1` ... : replica. `Rab` is the replica received both `a` and `b`.
- `a`, `b` ... `x`, `y`... : instance.
- `La`, `Lb`: is the leader replica of an instance `a` or `b`.

- `f`: max failure replica: `2`
- `n`: number of replicas, `n = 5`.
- `Q`: quorum: `|Q| = f+1`.
- `Qa`: quorum chosen for instance `a`.

- `a → b`: relation depends-on: `a` depends-on `b`.

# The relation between 2 instance

`La` and `Lb` choose quorums `Qa` and `Qb`, 

`{Rab} = Qa ∩ Qb != ø`.

`La` broadcast to `Qa` a message which contains:
- the commands to execute, 
- what `Qa` is.

so does `Lb`.

Because `La` must see at least one `b`, thus both `a` and `b` knows the
quorum of the other.

## Determine the order

There are 3 different patterns how `a` and `b` are replicated:

case-1. `La ∈ Qb and Lb ∈ Qa`:

```
b   b     a   a
a₀            b₀
La  R  R  R   Lb
```

In this case, both `La` and `Lb` decides `a ↔ b`: any consistent execution order
is correct.


case-2. `La ∈ Qb and Lb ∉ Qa`:

```
b   b  a  a
a₀            b₀
La  R  R  R   Lb
```

In this case `b → a`.

case-3. `{La, Lb} ∩ ∉ Qb and Lb ∉ Qa`:

```
    b  b
       a   a
a₀            b₀
La  R  Rab R  Lb

or
    b   b
    a   a   
a₀            b₀
La  Rab R  R  Lb
```

In this case the order is determined by the order the left-most replica received
`a` and `b`: 
On `Rab`, if `a` comes first, it determines `b → a` and vice versus.

In all these 3 cases, when `La` receives 2 replies, it gets the relation between
`a, b`,  and `Lb` will always get the same result.

∴ Consistency is guaranteed. Then `La` commit `a` and then responds to client
and simultaneously, broadcast the committed value of `a` to all others.


# Recovery

Recovery process `P` contacts a `Q` and recovers the value.
`P` can always recover a committed value because `La, Lb, Rab` constitues a
quorum of 5 replicas:

If `a` and `b` are both committed, `P` always see at least one `a` and one `b`,
thus `P` knows about their quorum `Qa` and `Qb`.

In case-1, `P` always knows only `a ↔ b` can be committed from `Qa` and `Qb`.

In case-2, `P` always knows only `a → b` or `b → a` can be committed from `Qa` and `Qb`.

For case-3,
- If `P` reached `La` or `Lb`, it gets the order of `a, b` from the leader(by
    waiting for them to commit).
- Otherwise, `P` must have reached `Rab`, it gets the order from `Rab`.

Then `P` commit the order.

If `P` didnot see `a`, then `a` is not committed.
`P` should Prepare with a higher ballot to take leadership of `a` and choose a
different `Qa` to recommit.

# Execution

see qpaxos-exec.md and exec-demo.py


