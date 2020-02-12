# Get rid of `seq`

对 epaxos 的主要修改:

第 1 个是 用请求的 ballot number 区分 PreAccept 和 Accept 请求:

- Accept-0: 其中(req.ballot=0). 相当于 epaxos 的 PreAccept
- Accept-1: 其中(req.ballot=1). 相当于 epaxos 的 Accept

这样有几个个好处:

- 用数字的方式判断 PreAccept 和 Accept.通过大小比较替代类型判断.

- epaxos 中用`seq`表示整个 instace 的先后顺序,
  实际上 instance 对每个它的 dep 都有一个独立的先后顺序.
  这里用 ballot 来替代 seq, 粒度更细, 判断也更容易,
  不需要处理`seq`由一个 dep 变化而变化的复杂性.

- 把所有`seq`相关的逻辑都可以转化成 dep 相关的逻辑.

- 可以引入一个比 deps 更清楚的强先后顺序, 更优雅的解决 SCC 过大的 livelock 问题.

第 2 个是引入强先后关系 ▷, 以此来替代 seq 在 SCC 中的作用.

TODO 定义 try-pre-accept

# 定义请求, 类型等

```
type Ballot = i64;
type ReplicaID = i64;

type InstanceOffset = i64;

struct InstanceID {
    replicaID: ReplicaID;
    offset: InstanceOffset;
}

struct Dep {
    instID InstanceID;
    ballot Ballot;
}

struct Instance {
    ballot: Ballot;
    instID: InstanceID;
    cmds: Vector<Command>;
    deps: Vector<Dep>;
}

struct Replica {
    instSpace: HashMap<ReplicaId, Vector<Instance>>,
}

type Accept {
    inst: Instance;
}
type Prepare {
    ballot: Ballot;
}

```

### Assign ballot:

- Accept-0: 发送一个 instance, 其中(instance.ballot=0). 相当于 epaxos 的 PreAccept
- Accept-1: 发送一个 instance, 其中(instance.ballot=1). 相当于 epaxos 的 Accept
- Prepare: 请求中 `ballot>=2`. 相当于 epaxos 的 Prepare.
- Accept-n: 发送一个 instance,其中(instance.ballot>=2), 相当于 epaxos 中的 Paxos-Accept.

### Observation 1

每个 ballot 只能用来确定一个值, 或没有确定任何值.

## Properties

算法保证如下特性:

### Thrifty: 一个 replica 在它接受处理的 instance 达到 stable 状态就立刻返回 client OK.

Stable 的定义, 简单说就是不会再变化了, 满足以下任一条件:

- 一个 instance 已经 committed,
- 在 Accept-0(epaxos 的 pre-accept 阶段)收到 fast-quorum(⌊(F+1)/2⌋)数量的相同返回值,
- 在 Accept-1(epaxos 的 accept 阶段)收到 quorum(F+1)数量的 OK,
- 在 recovery 过程中的 Accept-n 达到 quorum(F+1)数量的 OK

不考虑等待 deps 的所有 instance 都 commit 才返回 client 的方式去掉 seq.
这样可以保证最小延迟.

### Consistency: 2 个相关的 instance γ ∼δ, 如果 γ 至少一次在 δ 处于 stable 状态后看到它, 那么 γ 一定在 δ 之后 execute.

### Nondeterministic: γ 如果看到的 δ 都是变成 stable 之前的状态(有收到了 Accept-1 等导致 ballot 变大), 则不能保证他们之间的 execute 的顺序.

## 关于 stable 的例子

除非 δ 是 committed 状态的,
否则, δ 是否 stable 并不是 γ 看到 δ 时能确定的, 它取决于 δ 在被看到之后是否又发生了变化.

During PreAccept phase, instance γ sees instance δ, and δ is in **PreAccept** status(δ.ballot=0).

- If δ is committed with PreAccept status(ballot=0), γ had seen a **stable** δ.
- If δ is committed with Accept status(ballot=1), or committed by recovery with higher ballot,
  that means γ had **not** seen a stable δ, but only a obsolete δ.

If δ is stable, then δ must be stored on a quorum of replicas,
(fast-quorum or classic-quorum).
Then γ will see the δ of the greatest ballot, because 2 quorums have a intersection.

## Commit 算法: 从接受到请求到提交

1. 一个 replica R₁ 收到 client 发来的请求. 初始化一个 instance γ

   ```
   Instance {
       ballot: 0,
       instID: InstanceID{
           replicaID: my_id(),
           offset: get_next_unused_inst_slot(),
       },
       cmds: Vec(...),
       deps: [
           // inst1: the last interfering instance with γ
           replicaID_1: Dep{
               instID: inst1.instID,
               ballot: inst1.ballot,
           },
           // inst2: the last interfering instance with γ
           replicaID_2: Dep{
               instID: inst2.instID,
               ballot: inst2.ballot,
           },
           ...
       ]
   }
   ```

1. R₁ 作为 γ 的 leader, 发出 Accept-0 的请求给其他 replica, Accept-0 请求只包含一个 instance γ.

1. 其他 replica 接受并处理 Accept-0 的请求, 例如在 R₂ 上:

   如果 R₂ 上有 γ, 并且本地记录的 γ.ballot 大于 接受到的 γ.ballot, 则返回 NACK.

   如果收到 Accept-0 的 replica 上存在一个 instance δ, δ ∼ γ, 并且满足以下其中一个条件:

   - δ 不在 γ 的 deps 里.
   - δ 在 γ 的 deps 里,但 δ.ballot 大于 γ.deps 中记录的对应 δ 的 ballot.

   则更新 γ 的 deps 中 δ 对应的记录到更大的 ballot.

   > 这表示 γ 要记录看到最新的 δ.

- 返回更新后的 γ.

### Leader 收集 Accept-0 的返回

这部分和原 paper 描述类似. 除了:

- 判断 2 个 instance attribute 相同的条件不需要考虑 seq,
  但要求 deps 中的 instanceID 和 ballot 都相同才认为相同.

  原 paper 中只要求 deps 中的 instanceID 相同.

### Execute

1. Find an instance γ that is committed but not executed.

读出 γ.deps, 包括一组 instanceID 以及看到对应 instance 时, instance 的 ballot 的最大值:

```
γ.deps = {
    {instID: δ₁, ballot: 0},
    {instID: δ₂, ballot: 1},
    {instID: δ₃, ballot: n},
    ...
}
```

读取 instID 对应的 instance, 例如读出 δ₁, 比较 γ.deps 中记录的 δ₁.ballot 和 δ commit 时的 ballot
如果 deps 中记录的和 committed ballot 一样, 则 γ 看到了一个 stable 的 δ₁.
否则没有

到这里可以确认 stable 状态了, 然后引入强先后顺序的关系:

### Definition: interfering

interfering: γ ∼ δ

Just the same as epaxos specified.

### Definition: depends on

γ depends on δ : γ → δ

Just the same as epaxos specified.

### Definition: after

γ after δ : γ ▷ δ

For two committed instance γ ∼ δ.

- If γ and δ has different leader:

  γ ▷ δ if δ does not change after γ sees δ(when committing δ, δ.ballat is the
  same as γ sees):

  ```
  ∃ dep ∈ γ.deps
      ᴧ dep.instID == δ.instID
      ᴧ dep.ballot == δ.ballot
  ```

- If γ and δ has a same leader:

  by their natural order
  γ ▷ δ if γ comes after δ:

  ```
  γ.instID.idx > δ.instID.idx
  ```

### Transitivity

- γ ▷ δ implies not δ ▷ γ. **Not like →, there is no circle of ▷ relation**.

- If γ → δ, δ → ε: does not imply that γ → ε

- If γ ▷ δ, δ ▷ ε: then γ ▷ ε.

- If γ → δ, δ ▷ ε: then γ → ε.

- If γ ▷ δ, δ → ε: then γ → ε.

Thus every two instances γ and δ have one of following relation:

- γ ▷ δ : implies γ → δ
- γ → δ
- γ ↔ δ : γ → δ and δ → γ

## Determine execution order without seq

We determine the order by finding the first instance to execute:

1. reduce an SCC to a smaller one of at most n nodes(n = count(replicas)).

In an SCC(S₁), if γ ▷ δ, γ should not be execute before δ.
Thus γ must not be the first to execute.
Thus we remove γ from the SCC:

```
for γ in S₁:
    for δ in S₁:
        if γ ▷ δ:
            remove γ from S₁
```

Finally, we have a reduced SCC(S₂):

- Every replica(leader) has at most one instance(the oldest) in S₂.

  > Because two instances by a same leader must have a ▷ relation.

- There is not a ▷ relation in it, or it will be removed.

  In other word, no instance in S₂ has been initiated after any another instance
  responds client OK.

  Which means, these instances are initiated at **almost** the same time,
  choosing any order to execute them are reasonable.

2. Sort these instances by `replicaID` of their leader and execute the first one.

**NOTE: We can not execute all S₂ because the second instance to execute may not
in it**.
