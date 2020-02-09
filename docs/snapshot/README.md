### Snapshot 模块文档

#### 整体详情

##### 职责

sanpshot 模块的主要职责是持久化 epaxos 算法过程中任何需要持久化的部分，并且提供
读取功能。目前看，包括：

- 写入 celeritas 的 key 和 value 的值；
- epaxos 交互过程中的 instance；

##### 保证

为了配合 epaxos 算法的执行，还需要以下的保证：

- 已经存入 snapshot 的日志，如果该 instance 的 leader 上，instance 还不是
  executed 状态的，则它一定能被正确的读出。
- 支持事务的更新 keys-values；
- 支持事务的更新 keys-values 以及相应的 instance 。

##### 存取 Key-Value 值

这部分数据的来源是 executor 模块。它计算出 client 的请求中需要写入 celeritas db
的数据之后，将其更新到 snapshot 模块中。

这部分数据的使用者是 executor 模块。它可能计算出 client 的某个请求是读取某个 key
的值；或者是更新某个 key 的值。这时，需要从 snapshot 模块中获取值。

##### 存取日志

日志数据的来源是 smr 模块。它在 epaxos 交互过程中需要持久化的 instance 传入
snapshot 进行持久化保存。

日志数据的一个使用者是 executor 模块。它需要利用 snapshot 中的 instance 数据计算
出 需要保存在 celeritas db 中的 key-value。日志数据还会被 smr 模块使用，读取
epaxos 交互过程中需要更新 deps 时的 instance。

#### 接口的规划

需要提供以下接口：

- 提供一个用来存取 key-value 的接口。
  - client 提交的 cmds 涉及的 keys，以及 executor 产生的结果 values:
  ```
  Engine.set_kvs(vec<[u8]>, vec<[u8]>, vec<Instance>);
  Engine.get_kv([u8)]) -> [u8];
  Engine.get_kv_for_update([u8]) -> [u8];
  ```
- 提供一个用来存取 instance 的接口。
  - epaxos 交互过程中需要的 instance:
  ```
  Engine.set_instance(Instance);
  Engine.update_instance(Instance);
  Engine.get_instance(InstanceID) -> Instance;
  Engine.scan_instances(ReplicaID) -> InstanceIter;
  ```

#### 内部的规划

snapshot 内部使用 RocksDB 来存储数据。Key-Value 存取和 instance 存取使用不同的
column family。

##### snapshot 接收数据的大小限制

RocksDB 的 key 和 value 的大小限制是 8MB 和 3GB；目前看，还没有影响。

##### 日志的生命周期

所有的 instance，按照 epaxos 的正常运行过程，最终状态都会变成 executed，之后其不
会被改变，也不会被读取。所以在 snapshot 模块中，当一条 instance 在它的 replica
上的状态到达了 executed 之后，就是可删除的。

snapshot 启动一个定期删除日志的进程。

##### snapshot 的内部状态

snapshot 需要保存一些状态支持其他模块对 instance 的读取需求。这些配置存在一个单
独的 column family 中。

- 每个 replica 已经产生的最大的 instance id;
  ```
  key: $replica_id + current
  ```
- 每个 replica 已经 executed 的 instance id;
  ```
  key: $replica_id + executed
  ```
