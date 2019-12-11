### Snapshot 模块文档

#### 整体详情

##### 职责

sanpshot 模块的主要职责是持久化 epaxos 算法过程中任何需要持久化的部分，并且提供读取功能。目前看，包括：

- 写入 celeritas 的 key 和 value 的值；
- epaxos 过程中产生的日志；

##### 保证

为了配合 epaxos 算法的执行，还需要以下的保证：

- 已经存入 snapshot 的日志，如果它对应的 instance 还不是 Purged 状态的，则它一定能被正确的读出；
- 支持事务的更新 keys-values；
- 支持事务的更新 keys-values 以及相应日志。

##### 存取 Key-Value 值

这部分数据的来源是 executor 模块。它计算出 client 的请求中需要写入 celeritas 的数据之后，将其更新到 snapshot 模块中。

这部分数据的使用者是 executor 模块。它可能计算出 client 的某个请求是读取某个 key 的值；或者是更新某个 key 的值。这时，需要从 snapshot 模块中获取值。

##### 存取日志

日志数据的唯一标识是：replica_id + instance_id。

日志数据的一个来源是 log-manager 模块。它在 epaxos 交互过程中需要持久化的 instance 作为日志传入 snapshot。

另一个来源是 executor 模块。它在将 Key-Value 存入 snapshot 的时候，同时应该将这一批 Key-Value 所关联的 instance 一同更新。与 log-manager 一样，需要将 instance 更新到 snapshot。

日志数据的使用者是 executor 模块。它需要利用 snapshot 中的日志数据对内存中的日志数据做备份和可能的恢复。这时需要读取日志。

#### 接口的规划

为了给 executor 和 log-manager 提供方便的支持，需要提供以下接口：

- 提供一个用来保存 executor 模块执行结果的接口。目前看，应该包含：
    - instance 中的 cmds 涉及的 keys，以及 executor 产生的结果 values；
    - instance 本身；（需要把状态更新为 executed ）
- 提供一个读取 celeritas 的数据的接口，根据 key 返回 value；
    - 提供选项：是否需要 GetForUpdate，来锁定读走的 key 不被修改
- 提供一个用来保存 log 的接口，以 instance 为输入；
- 提供一个用来读取 log 的接口，以 ReplicaID+InstanceID 为输入，获取一个 json 格式的 instance ；
- 提供一个用来遍历某个 Replica 中所有状态不是 executed 的 instance 的接口，以 ReplicaID 为输入，获取一个产生 instance 的 iterator；
- 提供一个用来读取 log 状态信息的接口：
    - 状态信息包括（不限于）：committed—instance-id、accepted-instance-id、executed-instance-id、max-instance-id；

#### 依赖的外部数据结构

sanpshot 模块依赖一些外部定义的数据结构，这里给个例子，细节还需要再完善。

```rust
struct Instance {
    cmds: Vec<Command>,
    seq:  String,
    deps: Vec<Instance>,
    replica_id: String,
    id:  String,
    status: InstanceStatus,
}

enum Command {
    Get(String),
    Put(String, String),
    Deleted(String),
}

enum InstanceStatus {
    Pre_accepted,
    Accepted,
    Committed,
    Executed,
    Purged,
}
```

#### 内部的规划

snapshot 内部使用 RocksDB 来存储数据。Key-Value 存取和 log 存取使用不同的 column family。

##### snapshot 接收数据的大小限制

RocksDB 的 key 和 value 的大小限制是 8MB 和 3GB；据此是否作出一些限制，或者考虑其他解决方案？

##### 日志的生命周期

所有的 instance，按照 epaxos 的正常运行过程，最终状态都会变成 executed，一段时间之后，不再会有关于它的请求。snapshot 将其标记为 Purged。所以在 snapshot 模块中，当一条 log（也就是 instance）状态到达了 Purged 之后，就是可删除的。

snapshot 需要一个定期删除日志的机制，把已经 Purged 的日志清除。
