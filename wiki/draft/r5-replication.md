r5 replication is an optimization for exactly 5 replicas, 
with only one RTT to make consensus.

Consider 2 interfering

La choose a quourm of 3, e.g. `Qa = Ri, Rj, Rk`
Lb choose a quourm of 3, e.g. `Qb = Ri, Rj, Rk`

La send FastAccept with its quorum
Lb send FastAccept with its quorum

`Qi = Qa ∩ Qb != ø`.

Choose the first replica `Ro` in `Qi` as order leader.

If `La` receives reply from `Ro`, commit with the order `Ro` replied.
Otherwise, `La` increment ballot and choose another quorum to retry.

Because the order is determined by `Ro`, `La, Lb` always commit the same order
for `a, b`.

# Recovery

A recovery requires 3 replica thus it can see at least one of `La, Lb, Ro`.

If recovery see 3 non-leader replicas:

```
b   b     a  a
a            b
La  R  R  R  Lb


b   b  a  a
a            b
La  R  R  R  Lb

b   ab a
a            b
La  R  R  R  Lb

    b  ab a
a            b
La  R  R  R  Lb
```



If recovery see 1 leader: `La`


