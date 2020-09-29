# Welcome to the celeritasdb wiki!

Celeritas db is a distributed data storage based on Qpaxos consensus algorithm.

## Qpaxos consensus algorithm

- [[Replication | replication]]

- [[Execution algo based on seq | exec]] | 中文: [[基于 seq 的执行算法 | exec-cn]]

- [[Execution algo demo | report ]] 

---

This leaderless consensus algo is inspired by
[Epaxos](https://github.com/efficient/epaxos).

But epaxos is not very well designed. Confirmed bugs of epaxos are:

- [[Lack of vballot | epaxos-bug-lackof-vballot ]]
- Unspecified behavior about how to merge deps.

Imperfect impl:

- Livelock problem about dealing with big strongln-connected-component.
    Our [[ execution algo | exec ]] sovled this problem:
    [[Execution algo demo | report ]] 


---

[[Discussion-of-known-problems]]
