
# Get rid of `seq`

对epaxos的主要修改:

第1个是 用请求的ballot number区分PreAccept和Accept请求:

- Accept-0: 其中(req.ballot=0). 相当于epaxos的PreAccept
- Accept-1: 其中(req.ballot=1). 相当于epaxos的Accept

这样有几个个好处:

- 用数字的方式判断PreAccept和Accept.通过大小比较替代类型判断.

- epaxos中用`seq`表示整个instace的先后顺序,
    实际上instance对每个它的dep都有一个独立的先后顺序.
    这里用ballot来替代seq, 粒度更细, 判断也更容易,
    不需要处理`seq`由一个dep变化而变化的复杂性.

- 把所有`seq`相关的逻辑都可以转化成dep相关的逻辑.

- 可以引入一个比deps更清楚的强先后顺序, 更优雅的解决SCC过大的livelock问题.

第2个是引入强先后关系▷, 以此来替代seq在SCC中的作用.



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

- Accept-0: 发送一个instance, 其中(instance.ballot=0). 相当于epaxos的PreAccept
- Accept-1: 发送一个instance, 其中(instance.ballot=1). 相当于epaxos的Accept
- Prepare: 请求中 `ballot>=2`. 相当于epaxos的Prepare.
- Accept-n: 发送一个instance,其中(instance.ballot>=2), 相当于epaxos中的Paxos-Accept.


### Observation 1

每个ballot只能用来确定一个值, 或没有确定任何值.

## Properties

算法保证如下特性:

### Thrifty: 一个replica在它接受处理的instance达到stable状态就立刻返回client OK.

Stable的定义, 简单说就是不会再变化了, 满足以下任一条件:

- 一个instance已经committed,
- 在Accept-0(epaxos的pre-accept阶段)收到fast-quorum(⌊(F+1)/2⌋)数量的相同返回值,
- 在Accept-1(epaxos的accept阶段)收到quorum(F+1)数量的OK,
- 在recovery过程中的Accept-n达到quorum(F+1)数量的OK

不考虑等待deps的所有instance都commit才返回client的方式去掉seq.
这样可以保证最小延迟.

### Consistency: 2个相关的instance γ ∼δ, 如果γ至少一次在δ处于stable状态后看到它, 那么γ一定在δ之后execute.

### Nondeterministic: γ如果看到的δ都是变成stable之前的状态(有收到了Accept-1等导致ballot变大), 则不能保证他们之间的execute的顺序.

## 关于stable的例子

除非δ是committed状态的, 
否则, δ是否stable并不是γ看到δ时能确定的, 它取决于δ在被看到之后是否又发生了变化.

During PreAccept phase, instance γ sees instance δ, and δ is in **PreAccept** status(δ.ballot=0).

- If δ is committed with PreAccept status(ballot=0), γ had seen a **stable** δ.
- If δ is committed with Accept status(ballot=1), or committed by recovery with higher ballot,
that means γ had **not** seen a stable δ, but only a obsolete δ.

If δ is stable, then δ must be stored on a quorum of replicas,
(fast-quorum or classic-quorum).
Then γ will see the δ of the greatest ballot, because 2 quorums have a intersection.

## Commit算法: 从接受到请求到提交

1. 一个replica R₁收到client发来的请求. 初始化一个instance γ 

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

1. R₁作为γ的leader, 发出Accept-0的请求给其他replica, Accept-0请求只包含一个instance γ.

1. 其他replica接受并处理Accept-0的请求, 例如在 R₂上:

    如果R₂上有γ, 并且本地记录的γ.ballot 大于 接受到的γ.ballot, 则返回NACK.

    如果收到Accept-0的replica上存在一个instance δ, δ ∼ γ, 并且满足以下其中一个条件:

    - δ不在γ的deps里.
    - δ在γ的deps里,但δ.ballot 大于 γ.deps中记录的对应δ的ballot.

    则更新γ的deps中δ对应的记录到更大的ballot.

    > 这表示γ 要记录看到最新的δ.

- 返回更新后的γ.


### Leader收集Accept-0的返回

这部分和原paper描述类似. 除了:

- 判断2个instance attribute相同的条件不需要考虑seq,
    但要求deps中的instanceID和ballot都相同才认为相同.

    原paper中只要求deps中的instanceID相同.

### Execute

1. Find an instance γ that is committed but not executed.

读出γ.deps, 包括一组instanceID以及看到对应instance时, instance的ballot的最大值:
```
γ.deps = {
    {instID: δ₁, ballot: 0}, 
    {instID: δ₂, ballot: 1}, 
    {instID: δ₃, ballot: n}, 
    ...
}
```

读取instID对应的instance, 例如读出δ₁, 比较γ.deps中记录的δ₁.ballot和δ commit时的ballot
如果deps中记录的和committed ballot一样, 则γ看到了一个stable的δ₁.
否则没有

到这里可以确认stable状态了, 然后引入强先后顺序的关系:

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



