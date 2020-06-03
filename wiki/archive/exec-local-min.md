This a unfinished yet and can be proved to be an analogue to qpaxos-exec.md

# Execution

TODO: deps only records highest interfering, need to find the minimal. proof
that only directly deps  need to inherit the deps.

这个算法实现了有环depends-on graph中有限步的执行算法,
解决了livelock问题.


## Terminology

- `a → b`: `a` depends-on `b`.
- `a ↦ b`: `a` execute after `b`.

### Def-ord

`ord` 是用于为SCC中instance排序的字段: deps的每个`instance_id`之和, 以及`instance_id`:
`ord = (seq, instance_id)`.

∵ `a.deps ⊃ b.deps ⇒ a.seq > b.seq`

∴ 可以保证After关系一定被保证顺序执行.


### walking

选择ord最小的dep, 因此walking只跟当前节点相关.

### Def-local-min

一个 local-min 的instance `x`指: x所有的dependency的ord都大于x, 且每个dependency都有一条路径回到`x`, 且这些路径上的每个instance `y`的ord都大于`x.ord`.

显然一个没有出向边的节点也是一个trivial的local-min.

`x`,以及每个回到`x`的路径, 称为`L(x)`.

显然2个local-min的instance的`L()`图没有公共节点. 否则其中一个会在`L()`中看到更小ord的instance.


### cycle-order-consistency

过instance `x` 发现的环的顺序是consistent的.
经过x的环C, 


### DAG of local-min

一个图中, 找出所有local-min的节点. 如果删掉其中一个local-min节点`x`.

则可能会出现新的local-min节点`y`. 因此`y ↦ x`.

因此在删除-发现的过程中,最终所有节点被删除. 这个过程中的先后顺序构成一个DAG.

我们以这个DAG来作为exec的拓扑顺序.

一个图中一定有local-min节点存在: SCC中最小ord的instance是其中一个local-min,
没有出向边的instance是一个local-min.

linearizability: 如果`y → x and y.ord > x.ord`, `y`一定在`x`之后被发现.
`y → x and y.ord > x.ord ⇒ y ↦ x`.


连通图中所有节点都是连通的. 逐个去掉local-min, 可以删掉图中所有节点, 

consistency:
`x → y` 一定有确定的执行顺序,
如果不存在`y` 到 `x`的路径, `y`在x之后执行.
如果x y之间有环,
则发现环的顺序是确定的.在这些环上删除的local-min的节点顺序也是确定的.
如果删除几个节点之后再没有环, 则`y`一定先执行.
否则直到删除`x` 或 `y`, 删除的顺序也是确定的

finite:
一个环的形成要求: `a₀ → a₁ → a₂ ... aᵢ → a₀`
对`aᵢ, aᵢ₊₁`, 假设产生`aᵢ ←  aᵢ₊₁`的概率是p, `aᵢ.seq <  aᵢ₊₁.seq` 的概率是0.5
则`aᵢ → aᵢ₊₁`或`aᵢ.seq > aᵢ₊₁.seq`的概率是k=1-0.5p
形成一个长度为n的环的几率是kⁿ=(1-0.5p)ⁿ
平均换的长度为`1 k + 2 k² + ...` = `k/(1-k)²`
假设p=0.5, 平均环长度是12.

### 找到local-min

对于一个local-min instance, 删除它的一个出向边, 它还是一个local-min

且不影响它其他dep返回x的路径.

且不影响未来的local-min的路径.

因此, 找local-min的方式就是以xxx遍历图, 如果发现一个环, 则删除环中最小ord的instance对应的边. 直到找到一个没有出向边的节点.



---


## Algorithm


从一个instance `x` 开始选择指向最小`ord`的边. 走到下一个节点.

-   如果遇到一个没有出向边的instance `z`, 执行`z`.

-   如果遇到一个环,删除环中最小ord的instance为起点的边.




## Cases, do not read


<!-- vim: iskeyword+=-
-->
