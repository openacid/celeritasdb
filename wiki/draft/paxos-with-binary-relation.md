<!--ts-->

<!--te-->

2020 Aug 22

The most basic problem in a distributed system is to determine order of two object.
All other consensus is built upon the consensus about ordering.
E.g. a distributed state machine, the core is consensus about the operation log.
And the log is about to determine the order of a serias of command.
And the very basic problem is about the order of two command.

# Terminology

- `R0`, `R1` ... or `R[0]`, `R[1]`... : replica.
- `a`, `b` ... `x`, `y`... : instance.
- `La`, `Lb`: is the leader replica of an instance `a` or `b`.

- `F`: number of max allowed failed replica that.
- `n`: number of replicas, `n = 2F+1`.
- `Qc`: classic quorum: `Qc = F+1`.
- `Qf`: fast quorum: `Qf = F+⌊Qc/2⌋ = F + ⌊(F+1)/2⌋`.

- `a ~ b`: interfere: `a` and `b` can not exchange execution order in any
    instance(command) sequence.
- `a → b`: depends-on: `a` interferes with `b` and has seen `b`.
- `a < b`: do-not-know: `a` did not see `b`.


# Local order

Local order is determined by clock.
A server receives in `a, b` then the order this server believes is `b → a`.


# Definition: commit

The action commit is to broadcast to all replica about what value is **safe**.

## Definition: safe

Some value(e.g. an instance or a relation or something else)
is **safe** if:
it has been forwarded to enough replicas and constituted a quorum(`Qf` or `Qc`)
so that no process(command leader or recovery process) would never choose other
value for it to commit.




The algo of paxos-binary is about to determine the consensus about order of two object.
i.e., two replica initiated instance `a` and `b`.

We use a FastPaxos round to determine the order of `a, b`.
If FastPaxos does not commit, run into a ClassicPaxos to determine their order.

valid orders are:
`a → b`
`a ← b`
`a ↔ b`


## FP-guarantee

If a value is FastCommit-ed,
The system must guarantees recovery always choose this value.

∴ FastCommit-ed value must be all identical and constitues a fast round quorum.


### Fast-entanglement

Consider the value of the relation between two instance `a, b`,
If both are FastCommit-ed, from FP-guarantee:
`a` does not see `x` implies `x` sees `a`:
`a < x ⇒ x → a`.

∴ If one of two FastCommit-ed value is determined, the other can be determined too.

### Recovery-core

If a recovery process reaches either of leader of `a`(`La`) or leader of `b`(`Lb`),
from Fast-entanglement,
it is able to find the relation between `a, b` from a leader.

∴ We only need to consider a recover that does not reach either of `La` and `Lb`.

Recovery from the left `n-2` replica is the same as fast paxos specified.

In order to prevent recovery from choosing different value:

∴  Constitute fast quorum of `n-2`, i.e.,
two fast quorum `Qf(a), Qf(b)` must
satisfies:
for a recovery quorum, i.e., classic quorum
`Qc: La ∉ Qc and Lb ∉ Qc`:
`Qf(a) \ {La, Lb} ∩ Qc` and `Qf(b) \ {La, Lb} ∩ Qc` must have intersection.


For 5 replicas scenario, `Qc=3`, `n-2=3`,
`|Qf(a) \ {La, Lb}|` and `|Qf(a) \ {La, Lb}|` is at least 2.
For 7 replicas scenario, `Qc=4`, `n-2=5`,
`|Qf(a) \ {La, Lb}|` and `|Qf(a) \ {La, Lb}|` is at least 4.


Inference: the value of relation `a, x`,
If `x` is committed with `x < a`, then only `a → x` can be FastCommit-ed.
∴ A more strict conclusion is:


## Fast path

Leader:

1. Leader: Initiate instance `a`: build `a.deps`:
2. Leader: FastAccept: forward `a` to other replicas.
3. NonLeader: FastAccept: update `a` with local instances
4. Leader: Handle-FastAcceptReply

## Slow path

Leader:

1. Leader: Choose `a.deps`
2. Leader: Send Accept to replicas
3. NonLeader: handle Accept
4. Leader: Handle AcceptReply


## Commit

Just commit.

```
|                                      |
| Leader of a                          | Non Leader
| ---                                  | ---
| blt=0                                | 
| fast_accept(v=a<b, blt)              |
| ---                                  | --- handle-fast-accept-request
|                                      |
|                                      | if a > b exists:
|                                      |   if status_of(a > b) == fast_accepted:
|                                      |     reply(declined)
|                                      |   if status_of(a > b) == accepted:
|                                      |     reply(a > b, accepted)
|                                      |   if status_of(a > b) == committed:
|                                      |     reply(a > b, committed)
|                                      |
|                                      | else:
|                                      |   save(a < b, fast_accepted)
|                                      |   reply(a < b, fast_accepted)
| --- handle-fast-accept-replies       | ---
|                                      |
| for repl in replies:                 |
|   if repl.committed:                 |
|     return commit(repl)              |
|                                      |
| max_accepted = None                  |
| for repl in replies:                 |
|   if repl.accepted:                  |
|     return accept(repl)              |
|                                      |
| ab = 0                               |
| ba = 0                               |
| for repl in replies:                 |
|   if repl.accepted:                  |
|     return accept(repl)              |
|                                      |
|   if repl is from Lb:                |
|     continue                         |
|                                      |
|   if repl == (a<b):                  |
|     ab++                             |
|   else:                              |
|     ba++                             |
|                                      |
| if ab >= (n-2)-Qc+Qc/2:              |
|   return commit(b→a)                 |
|                                      |
| if ba >= (n-2)-Qc+Qc/2:              |
|   return commit(a→b)                 |
|                                      |
| if ab == 0 and count(replies) >= Qc: |
|   return accept(a→b)                 |
|                                      |
| if ba == 0 and count(replies) >= Qc: |
|   return accept(b→a)                 |
|                                      |
| accept(a↔b)                          |
| ---                                  | --- handle_accept_request(relation)
|                                      |
|                                      | save(relation)
|                                      |
| --- handle_accept_replies            | ---
|                                      |
| positive = 0                         |
| for repl in replies:                 |
|   if repl.ok:                        |
|     positive++                       |
|                                      |
| if positive >= Qc:                   |
|   commit(replies[0])                 |
|                                      |
```



# Messages

`ballot` is the ballot number,
- For FastAccept is always `2k`.
- For Accept ballot is `2k+1`.
- `ballot` in Commit message is useless.

TODO
Changes:

If `x` is fast-committed:

- If `a` reached `Lx`, then `a` know if `x` is committed, because `Lx` is the
    first to commit.
    Although there is chance `x` is committed after `a` reaches `Lx`,
    `Lx` broadcasts `x is committed` very likely earlier than another instance
    brings `x is committed` through its fast-accept request.

- If `a` did not reach `Lx`, then `a` must have reached `g - {La, Lx}`,
  this prevent other value of `a > y` to commit.
  ∴ `a > x` is safe to fast commit.


# Recovery

Assumes:

- The instance to recover is `a`.
- The leader of `a` `La` is `R0`
- The recovery process is `P`(`P != R0`).

## Cases not need to recover:

After Preparing on a quorum(`Qc`):

- If `P` saw `La` or `Lb`, exit and wait for `La` or `Lb` to commit.

- If `P` saw a Commit phase `a`: broadcast `a` and quit.

- If `P` saw a Accept phase `a`: run classic paxos with this
  value and quit.

∴ `P` only need to recover if all of `a` it saw are in FastAccept phase.

## Recover instance in FastAccept phase

After Prepare for relation `a, b` on a quorum,
`P` could see different values `a < b` or `b < a`(`a < b` is the same as `a ← b`, because only `a ← b` can be set on this replica).

From TODO,
Only

As the following diagram shows:

```
a      b            a<b           a>b
---    ---   ...    ---           ---
R0     R1    ...    Ri
```

E.g.:

```
          | a     a      a
        x | '→x    ↘      ↘
a       y |   y      y      y
---   --- | ---   ----   ----
R0    R1  | R2    R3     R4
La    Lb
down  down
```

### Lemma-fast-commit-candidate

```
                    | a     a       b    b    b
                    |  ↘     ↘     ↙    ↙    ↙ 
a       b           |   b     b   a    a    a  
---   ---  ---  --- | ---   ---   ---  ---  ---
La    Lb
down  down
```

FastCommit requires Qc/2+1 identical value in any `Qc`.

`a → b` is not FastCommit-ed.
`b → a` is possible to be FastCommit-ed.

∴ There is at most one value of `a, b` that may have been FastCommit-ed.

∴ choose the value that has at least `Qc/2+1`. Otherwise choose the any value.

Then run Accept to decide it.




<!-- vi: iskeyword+=-
-->
