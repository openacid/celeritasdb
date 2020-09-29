> `instance` and `vertex` are equivalent concepts in this doc.
>
> DAG: directed acyclic graph.

# Algorithm

en: Let `x.seq=(x.seq, x.leader_id, x.index)`, thus any two instances have different `seq`.
cn: 令 `x.seq=(x.seq, x.leader_id, x.index)`, 这样任何两个instance都有不同的 `seq`.

en: Select one non-executed instance `x`.
cn: 选择一个未执行的instance `x`.

en: Depth-first walk along the dependency graph, choose next instance with the **smallest** `seq`.  Mark accessed instances as **accessed**.
cn: 深度优先遍历， 选择具有 **最小** `seq` 的下一个instance。 将已访问的instance标记为 **accessed**。

-   en: If it walks to an instance without outgoing edges, execute it and remove it, then backtrack to previous instance and go on.
-   cn: 如果它走到一个没有出向边的instance，执行并删除它，然后回溯到先前的instance并继续。

-   en: If it walks to an **accessed** instance, then we know that a cycle is found: Walk through the cycle again, find the instance `y` that has the smallest `seq` in the cycle. Remove the edge `y → z` from the cycle.
-   cn: 如果它走到一个 **accessed** 的instance，那么我们知道找到了一个环: 再沿着这个循环走一遍，找到环上具有最小`seq`的节点`y`, 从环中删掉以`y`起始的边.

    en: Then continue walking from `y`.
    cn: 然后从`y`继续。


## Example of execution

en: One of the properties of this algorithm is
en: it does not need to walk through the entire SCC
en: before executing an instance.
cn: 这个算法的一个特点是
cn: 在执行某个instance 之前，它不必遍历SCC中所有的环。

en: As shown blow:
cn: 如下图所示:

```
          .-> 4
          |
1 -> 6 -> 3 -> 5 -> 2 -> 8
     ^              |
     `--------------'
```

en: Starting from `1`, it walks along **min-edge**s to `4`, And finds out that `4` is the first instance to execute.
en: The effective path it walked is: `[1, 6, 3, 4]`, The cycle `6, 3, 5, 2` has not yet been discovered.
cn: 从 `1` 开始执行，它通过**min-edge**遍历到 `4`,
cn: 发现`4`是第一个应该被执行的,
cn: 这时遍历过的有效路径为`[1, 6, 3, 4]`,
cn: 环 `6, 3, 5, 2` 还没被看到.

en: `4` is executed and removed.
en: It backtrack to `3` and continue the walking:
cn: `4` 被执行并从图中删掉.
cn: 然后回溯到`3`继续遍历:

```
1 -> 6 -> (3) -> 5 -> 2 -> 8
     ^                |
     `----------------'
```

en: Then it found a cycle `6, 3, 5, 2`, After removing the edge of the smallest vertex: `(2, 6)`, the effective path now is `[1, 6, 3, 4, 3, 5, 2]`(cycles do not count).
cn: 继续遍历, 找到一个环 `6, 3, 5, 2`,
cn: 然后删掉环中的最小vertex对应的边 `(2, 6)`,
cn: 这时遍历过的有效路径为`[1, 6, 3, 4, 3, 5, 2]`(环不计入).

en: Continue to walk from `2`
en: until it finally finds `8` as the second instance to be executed.
cn: 继续从`2`开始遍历, 最终找到`8`作为第2个被执行的instance:

en: And then it backtracks to `2, 5, 3, 6, 1`, and executes them in this order.
cn: 然后再回溯到`2, 5, 3, 6, 1`, 并依次执行.


## en: Optionally replaceable concept
## cn: 可替换概念

en: This algorithm also applies to scenarios where `seq` is not used, as long as the following conditions are met:
cn: 这个算法也适用于不使用seq的场景, 只需要实现满足以下条件:

-   en: It can distinguish which edges can be deleted to ensure that
    en: linearizability is not broken.
-   cn: 可以区分出哪些边是可以删除的, 以保证linearizability不被破坏.

-   en: There must be a way to decide which edges to delete on different replicas
    en: (for example, select the smallest instance-id).

    en: Moreover, the way needs to make the walking prefer earlier proposed instances.
-   cn: 在不同的replica上有确定的方式决定删除哪些边(例如选择instance-id最小的).
    cn: 并且选择的方式需要让遍历趋向于优先遍历到较早被propose的instance.

en: As long as the above conditions are met, this algorithm applies.
en: In this document it uses `seq`.
cn: 只要满足以上条件, 就可以使用同样的算法确定执行顺序.
cn: 本文档中是通过`seq`来实现以上保证.


# Properties

en: Besides the linearizability, consistency, and avoiding the livelock naturally, it also provides some other guarantees:
cn: 除了linearizability、consistency和避免livelock，
cn: 它还提供了一些其他保证:

-   en: Execution is safe to stop at any time.
    en: Instances in An SCC does not need to be executed in a transaction.
-   cn: 任何时候停止执行都是安全的。SCC中的instance不需要在一个事务中执行。

-   en: Allow multiple threads to execute simultaneously.
    en: The only requirement is that modifications to an instance(removing an edge or
    en: removing an instance) must be atomic.
-   cn: 允许多个线程同时执行。
    cn: 惟一的要求是对instance的修改(删掉边或删掉instance的操作)必须是原子性的。

-   en: Time complexity can be optimized to about `O(2m)`, where m is the number of instances.
-   cn: 时间复杂度可以优化到`O(2m)`左右，其中m为instance数。


# Definition

-   en: `G = (vertices, edges)`, an edge has a weight.
-   cn: `G = (vertices, edges)`, 边有权重.

-   en: Edge and edge weight: With given instances `x, y`, if x depends on y, there is an edge: `(x, y)`, the weight of the edge is: `(x.seq, y.seq)`.
-   cn: 边和边的权重: 对2个instance`x, y`, 如果x依赖y: 则定义一条边edge: `(x, y)`, 边的权重为: `(x.seq, y.seq)`.

-   **min-edge**:
    en: is the edge with the smallest `weight` in a set of edges.
    cn: 具有最小权重的边:

    - en: for a vertex `x`, it is the smallest of all outgoing edges:
    - cn: 对一个vertex `x`, 它是所有出向边中最小weight的:
      `MinEdge(x)`

    - en: for a graph `G`, it is the smallest of all edges in the graph:
    - cn: 对一个图 `G`, 它是所有图中边中最小weight的:
      `MinEdge(G)`

-   **min-cycle** `Cᵢ(x, G)`:
    en: is the i-th cycle found by walking along vertex **min-edge** from vertex `x`.
    cn: 从`x`出发, 沿着节点的**min-edge**遍历发现的第i个环。

    en: `C(G)`: is the set of all **unrelated** cycles in `G`:
    cn: `C(G)`图`G`中所有 **不相关** 的环的集合:
    `C(G) = {C₁(x, G) | x ∈ G.vertices}`


# Proof


## Proof: execution linearizability

en: If `u ~ v` and `u` is initiated after `v` is committed, then `u.seq > v.seq`.
cn: 如果 `u ~ v` 且 `u` 在 `v` 之后被初始化,
cn: 那么一定有 `u.seq > v.seq`.

en: ∴ If `u.seq <= v.seq`, `u` does not have to execute after `v` thus removing
en: a **min-edge** does not affect linearizability.
cn: ∴ 如果 `u.seq <= v.seq`, `u` 不需要在 `v` 之后被执行.
cn: 因此去掉这样一个 **min-edge** `(u, v)` 不会影响linearizability.

QED.


### Def-After

This algo removes edges and vertices in some order.
Define a relation **After** as:
an edge of a vertex is After another edge or vertex if they are always removed
in the same order with any walking.

A vertex `x` is After all its edges: `x ↦ x.edges`.

An edge `e` is After its destination vertex if it isnot removed as a min-edge.


## edge-DAG

en: The order in which edges are removed follows a topological order: `edge-DAG(G)`.
cn:

en: First find this DAG:
cn: 首先找到这个DAG:

en: Walk from one vertex, till an edge or a vertex is removed. Define the set of edges and vertices that would be removed by walking from every vertex as `E₀(G)`.
en: Elements in `E₀(G)` are unrelated to each other, i.e., given `e₀ᵢ ∈ E₀(G), e₀ⱼ ∈ E₀(G)`, they could be removed in arbitrary order.
cn: 从每一个节点开始遍历, 直到删除一个最小边, 所有被删除边的集合`C(G₀)`, 容易看出,
cn: `C(G₀)`中的环是互不相关的:
cn: 即, 从一个vertex开始的遍历先后访问这两个环`cᵢ, cⱼ ∈ C(G₀)`,
cn: 最终被删除的环中的min-edge都一样.

We use a function `rmSeq` to define the sequence of edges or vertices to remove by starting walking from vertex(`x`):
`rmSeq(G, x) ⇒ [e₁, e₂, v₃, e₄ ...]`,

For every `xᵢ ∈ G.vertices`, `rmSeq(G, xᵢ)` is a permutation of `G.vertices ∪ G.edges`.
If `u` is before `v` in every `rmSeq(G, x)`, then `v` is After `u`, i.e., `v ↦ u`.

Thus `E₀(G) = {rmSeq(G, x)[0] | x ∈ G.vertices}`

Define `E₁(G)` as the set of the first edge to remove that is not in `E₀(G)`.
`E₁(G) = {(rmSeq(G, x) \ E₀(G))[0] | x ∈ G.vertices}`.

`E₂(G) = {(rmSeq(G, x) \ E₀(G) \ E₁(G))[0] | x ∈ G.vertices}`.
...


Given `e₁ᵢ = (x₁ᵢ, y₁ᵢ) ∈ E₁(G), e₀ⱼ = (x₀ⱼ, y₀ⱼ) ∈ E₀(G)`, `e₁ᵢ` is After `e₀j`(`e₁ᵢ ↦ e₀j`) if a walking starting from `x₁ᵢ` passes `x₀ⱼ`.
Because if `e₀ⱼ` presents, the walk from `x₁ᵢ` always removes `e₀ⱼ` first then `e₁ᵢ`.

en: This way edges are removed following this topological order of relation
**After**.


## Proof: execution consistency

en: Two interfering instances `x → y` will be executed in the same order on
every replica.
cn: 任一replica上有依赖的2个instance`x → y`执行顺序相同.

Note:

> en: Because an instance only records the largest `dep` of a leader, Thus when executing, in addition to a `dep` of an instance, it also need to find out all the indirect dependencies that have the same leader of `dep`.
> cn: 因为实现中只记录了一个leader上最大的dep, 因此在执行时, 除了找到instance记录的dep之外, 还需要找出所有省略的依赖: 即同一leader上其他instnace-id更小的instance.

Proof:

If two min-cycle have common vertices, they are dicovered in the same order on
every replica.

Common vertex `x`: `(x, y₁)` `(x, y₂)`, `y₁.seq < y₂.seq`
Then any walking finds `C₁` before `C₂`

```
 .----------.
 |  C₂      y₂
 |        ↗
 `.. → x
 '        ↘
 |  C₁      y₁
 `----------'
```

(1) If one replica finds a min-cycle `C` by walking from some vertex, every replica
will find the same min-cycle `C`, no matter what vertex it starts walking from.

`C` must be broken thus every replica need to remove an edge in `C`.
If a replica removed an edge without walking along `C`, it must have walked
along another `C'`. This conflicts with 


en: Case-1: If `(x, y)` is the smallest weight edge of a min-cycle, i.e.,
cn: Case-1: 如果`(x, y)` 是一个min-cycle的 TODO
`x → y → z ... → w → x`:

When `(x, y)` is removed,
Any min-cycle that pass `y` always walks to `x`.
And `x` has the smallest `seq`.

∴ Any edge along the path `y → .. → x` wont be the smallest in any min-cycle,
thus wont be removed, unless `x` is removed.

∴ `(y, z)` wont be removed unless `x` is removed. By Def-After,
`(y, z) ↦ x`.

∴ `y ↦ (y, z) ↦ x`


Case-2: If `(x, y)` is not the smallest weight edge of any min-cycle: From the walking algo, an edge is removed either by dicovering a min-cycle or by removing its destination vertex. Thus `(x, y)` wont be removed unless `y` is removed.

∴ `x ↦ (x, y) ↦ y`


∴ In either case, `x` depends-on `y`, i.e., `x → y`, implies there is a consistent order in which to remove `x` and `y` on every replica.

QED.


## Proof: execution concurrency
TODO

en: Two concurrent walks do not affect each other.
cn: 两个并发的遍历不会相互影响。

en: By **edge-DAG**，
en: The walking in an SCC `G` is the same as the walking in `edge-DAG(G)`.
en: And since a walking only select min-edge.

en: As long as 2 walking process atomically delete vertex/edge, they execute instance in the same topological order of `edge-DAG(G)`.
cn: 根据 **edge-DAG**，
cn: 对SCC的遍历可以看做对`edge-DAG(G)`的遍历.
cn: 又因为遍历中只选择min-edge,
cn: 只要2个遍历进程原子的删除vertex/edge,
cn: 它们执行instance的拓扑顺序都是一致的.

QED.


## Proof: incremental execution

en: An execution process can be safely restarted at any time.
en: It does not need to execute all instances in an SCC once all together.
cn: 执行过程可以在任何时候安全地停止再重新启动。
cn: 它不需要一次性地执行SCC中所有的instance。

en: A restarted execution process is equivalent to:
en: a running execution process and another process paused for ever.
cn: 重启的执行过程与以下场景等同:
cn: 一个正在运行的执行进程和另一个永远被暂停的进程。

QED.

## Proof: finite steps to remove one vertex

The number of steps this algo takes to find the first vertex to remove is finite.

Proof:


Edges in `Eⱼ(G)` splits the `G.vertices` into groups:
A group `V₀ᵢ` is set of all vertices that a walk from it will remove `e₀ᵢ`:
`V₀ᵢ = {x ∈ G.vertices | e₀ᵢ ∈ rmSeq(G, x)}` for every `e₀ᵢ ∈ E₀(G)`.

V₀ = G.vertices

A walking that arrives at a vertex in `V₀ᵢ` always walks to `e₀ᵢ` then walks along one of the outgoing edges from `e₀ᵢ`.
Thus further walking is just like walking among `V₁ = {e.source | e ∈ E₀(G)}`, except there are multiple edges between two vertices, and a vertex has edges pointing to itself.

Assumes a vertex only has finite number(`k` at most) of outgoing edges.
When all of its edges are remove the total number of edges removed is finite:
first round: takes `c` steps to find a min-cycle.
second round: `c` steps to find a cycle in `{}`
...
`n = cᵏ`


<!-- vim: iskeyword+=-
-->
