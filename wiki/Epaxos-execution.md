# Instance

这里仅列出`execution`需要用到的变量

```go
type Instance struct {
    Deps    [ReplicaCount]int32
    Index   int
    Lowlink int
    Seq     int
}
```

- `Deps`：表示当前`instance`所依赖的其它`Replica`上的`instance id`

  - `Deps[0]`表示依赖`Replica 0`上的的`instance id`

  - `Deps[1]`表示依赖`Replica 1`上的的`instance id`

  - `Deps[2]`表示依赖`Replica 2`上的的`instance id`

- `Index`：用于使用`Tarjan`算法寻找强连通分量的节点搜索次序编号，`=0`表示此节点没有被访问过

- `Lowlink`：`Tarjan`算法中节点或节点的子树能够追溯到的最早的栈中节点的次序号

- `Seq`：用于强连通分量定序

# 看下执行过程

例如下面是一个`ins`的依赖图

```
ins1---------->ins3---------->ins5
 ^              |              |
 |              |              |
 |              |              |
 |              |              |
 |              V              V
ins2<----------ins4---------->ins6
```

采用`Tarjan`算法搜索图中的强连通分量`SCC:Strongly Connected Components`，该算法有两个重要的数组

- DFN[]：全称`Depth First Number`，表示节点被搜索到的次序编号，对应`struct Instance`中`Index`

- LOW[]：表示节点或者节点的子树能够追溯到的最早的栈中节点的次序编号，对应`struct Instance`中`Lowlink`

下面来展示一下执行过程，这里使用`DFN`(`struct Instance`中`Index`)，`LOW`(`struct Instance`中`Lowlink`)

- 从`ins1`开始`DFS`(`Depth First Search`)遍历, 比如顺序是`ins1->ins3->ins5->ins6`，依次入栈，如下

```
-----------
|  ins6   |->DFN:4 LOW:4
-----------
|  ins5   |->DFN:3 LOW:3
-----------
|  ins3   |->DFN:2 LOW:2
-----------
|  ins1   |->DFN:1 LOW:1
-----------
```

- 搜索到`ins6`发现没有边可搜索，这个时候退栈发现`DFN:4==LOW:4`，说明`ins6`是一个强连通分量，这个时候去`exec ins6`

- 同理`ins5`也是一个强连通分量，`exec ins5`之后，这个时候栈如下

```
-----------
|  ins3   |->DFN:2 LOW:2
-----------
|  ins1   |->DFN:1 LOW:1
-----------
```

- 出栈到`ins3`，继续搜索`ins4`并加入栈

- 继续搜索`ins2`，由于`ins2`有边指向`ins1`，`ins1`还在栈里面，这个时候`ins2`的`LOW`取`ins1`的`LOW`，也就是`1`，栈如下

```
-----------
|  ins2   |->DFN:6 LOW:1
-----------
|  ins4   |->DFN:5 LOW:5
-----------
|  ins3   |->DFN:2 LOW:2
-----------
|  ins1   |->DFN:1 LOW:1
-----------
```

- 这个时候，所有节点已经搜索完成，开始回溯并修改`LOW`的值，取看到的最小值，如下

```
-----------
|  ins2   |->DFN:6 LOW:1
-----------
|  ins4   |->DFN:5 LOW:1
-----------
|  ins3   |->DFN:2 LOW:1
-----------
|  ins1   |->DFN:1 LOW:1
-----------
```

- 我们需要找到一个节点`DFN=LOW`，也就是这个强连通分量的根，上述例子中就是`ins1`，栈里面的元素集合就是一个强连通分量，这里是
  `ins1 ins2 ins3 ins4`

# 强连通分量定序

- 强连通分量中节点是相互可达的，也就是相互依赖，但是我们需要保证强连通分量中的`ins`执行顺序在每个`Replica`中保持一致，采用的
  方式是通过`ins`中`Seq`进行排序，排序的结果也就是`ins`的执行顺序

# 其它实现细节

- 每个`Replica`保存了其`execution`后最大的`ins id`，定义为`ExecedUpTo`

- 比如`ins1(Replica1)->ins10(Replica2)`，实际上`Replica2`需要`exec`的是`[ExecedUpTo+1, ins10]`，也可以这样理解
  `ins1(Replica1)`依赖`Replica2`上`ins id`在`[ExecedUpTo+1, ins10]`中的所有`ins`

- `execution`线程会给当前`Replica`上最小的没有`committed`的`active ins`设置一个超时时间，到达超时时间还没有达到
  `committed`状态，会触发`ins`的恢复流程
