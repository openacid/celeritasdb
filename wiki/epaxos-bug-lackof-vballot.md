Epaxos does not store the ballot at which an instance is accepted, which is different from classic paxos.
And this is a bug that inconsistent dep-graph would be committed by different
replica through recovery.

Epaxos mentioned that it will choose an accepted value, in Figure 3: The EPaxos simplified recovery procedure:
> else if R contains an (γ,seqγ,depsγ,accepted) then run Paxos-Accept phase for (γ,seqγ,depsγ) at L.i

And it said that only committing a safe value is guaranteed via classic paxos:

Proposition 2. Replicas commit only safe tuples:
> Proof sketch. A tuple (γ,seqγ,depsγ) can only be committed at a certain instance Q.i (1) after the Paxos- Accept phase, or (2) directly after Phase 1.
> Case 1: A tuple is committed after the Paxos-Accept phase if more than half of the replicas have logged the tuple as accepted (line 20 in Figure 2). The tuple is safe via the classic Paxos algorithm guarantees.

Classic paxos need to record the ballot at which a value is accepted(`vrnd` in the classic paxos paper), and it
choose the value with the greatest ballot thus only the committed value will be
chosen to commit at a higher ballot.

Epaxos impl has no such a field, only the last seen ballot. 
How does it guarantee the committed value will be chosen in this scenario?

With a setup with: n = 7, f = 3;

- And instance `a`, a recovery process with ballot=1 wrote an accepted `a` with
`a→b₂`(`a` depends on `b` and `seq` of `b` is 2) at ballot=1 on replica-1(`R1`). Then quit.

- Another recovery process with ballot=2 successfully committed `a` with
    `a→b₃`(`a` depends on `b` and `seq` of `b` is 3) at ballot=2, on replicas(`R2, R3, R4, R5`).

- A third recovery process with ballot=3 sends Prepare to R1 then quit. Now on
    R1, the ballot for `a` is 3.

- A forth recovery process prepared on `R1, R2, R3, R4`. It saw two accepted
    value of `a`:
    - `a→b₂`, at ballot=3
    - `a→b₃`, at ballot=2

```

time | 
     | La  R1  R2  R3  R4  R5  Lb  |
     | a                       b₀  |
     |             b₁  b₂  b₃      |
     |     a   a   a   a   a       |
     |             ↓   ↓   ↓       |
     |             b₁  b₂  b₃      |
     |                             |
     |     --------------          |
     |     Recovery blt=1          |
     |     choose a→b₂             |
     |     a                       |
     |     ↓                       |
     |     b₂                      | Prepared on R1..R4;
     |     A                       | Accepted at ballot=1 by R1
     |                             |
     |         --------------      |
     |         Recovery blt=2      |
     |         choose a→b₃         |
     |         a                   |
     |         ↓                   |
     |         b₃                  | Prepared on R2..R5;
     |         A   A   A   A       | Accepted at ballot=2 by R2..R5
     |                             |
     |     --------------          |
     |     Recovery blt=3          |
     |     a                       |
     |     ↓                       |
     |     b₂                      | Prepared on R1 with ballot=3;
     |     A                       | 
     |                             |
     |     --------------          |
     |     Recovery blt=4          |
     |                             | Prepared on R1..R4
     |                             |
     v
```

After these steps, recovery with ballot=4 would see two accepted value `a→b₂` at
ballot=3 on R1, and `a→b₃` at ballot=2, on R2, R3, R4.

Now the recovery process can not tell which is the committed value, by ballot.

---

The issue about this problem is at
https://github.com/efficient/epaxos/issues/20

And this bug is also mentioned in another paper: 
https://arxiv.org/abs/1906.10917
