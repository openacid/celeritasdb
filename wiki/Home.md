# Welcome to the celeritasdb wiki!

Celeritas db is a distributed data storage based on Qpaxos consensus algorithm.

## Qpaxos consensus algorithm

- [[Replication and Exec | replication-algo]]

## Another execution algo:

This execution algorithm requires an `seq` to determine command execution order.
But it also applies to the original replication protocol with minor
modifications.

- [[Execution algo based on seq | qpaxos-exec]] | 中文: [[基于 seq 的执行算法 | qpaxos-exec-cn]]

---

[[Discussion-of-known-problems]]
