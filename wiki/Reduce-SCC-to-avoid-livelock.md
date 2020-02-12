This page is an optimization based on [Algorithm to search for SCC](Epaxos-execution)

# Why

As epaxos paper described, an SCC must be fully walked before we can determine the instance order.
An SCC could be very large(possibly infinite) that halts execution.
In the paper the author suggested deferring commit to break a large SCC.
Here we propose a simpler way to determine instance order without halting the system.

Former discussion:
[Another solution to livelock](https://github.com/efficient/epaxos/issues/14)

# How

Within an SCC, the leader replica set of all instances: `replicas`.

First, there are several properties must be maintained with our algorithm:

## Properties:

### Execute-earlier-instance-first

For two instances `a` and `b` initiated by a same leader L:
If `a.instance_num < b.instance_num`, `a` must be executed before `b`.

### Execute-dependent-instance-first

If there is not an SCC,
dependent instance must be executed first.

### Execute-smaller-seq-instance-first

If there is an SCC,
instance with smaller `seq` must be executed earlier. (2)

## Find the first instance to execute.

- In a SCC, only the instances with least `instance_num` could be the first one to execute.
  By [Execute-earlier-instance-first][].

  E.g. in a replica the instance space is as below:

  ```
  |   d   |
  b   |   e
  |   c   |
  a   |   |
  |   |   f
  ----------
  R1  R2  R3
  ```

  The first instance to execute could be one of `a, c, f` but can **NOT** be either
  of `b, d, e`.

  This way we reduce the entire SCC(of `a, b, c, d, e`) into a smaller SCC(`a, c, f`).

  > The first one to execute in entire SCC is also the first one to execute in
  > the reduced SCC.

  > Obviously, the reduce graph has to be an SCC.

* Find the instance from `a, c, f` with the smallest `seq` to execute.

  > Or choose the first one by other criteria.

[execute-earlier-instance-first]: #Execute-earlier-instance-first
