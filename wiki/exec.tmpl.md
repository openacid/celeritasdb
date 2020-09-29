> `instance` and `vertex` are equivalent concepts in this doc.
>
> DAG: directed acyclic graph.

# Algorithm

en: Let `x.seq=(x.seq, x.leader_id, x.index)`, thus any two instances have different `seq`.
cn: 令 `x.seq=(x.seq, x.leader_id, x.index)`, 这样任何两个instance都有不同的 `seq`.

en: Select one non-executed instance `x`.
cn: 选择一个未执行的instance `x`.

en: Depth-first walk along the dependency graph, choose next instance with the **smallest** `seq`.  Mark accessed instances as **accessed**.
cn: 深度优先遍历，
cn: 选择具有 **最小** `seq` 的下一个instance。
cn: 将已访问的instance标记为 **accessed**。

-   en: If it walks to an instance without outgoing edges, execute it and remove it, then backtrack to previous instance and go on.
-   cn: 如果它走到一个没有出向边的instance，
    cn: 执行并删除它，
    cn: 然后回溯到先前的instance并继续。

-   en: If it walks to an **accessed** instance, then we know that a cycle is found:
    en: Walk through the cycle again, find the instance `y` that has the smallest `seq` in the cycle.
    en: Remove the edge `y -> z` from the cycle.
-   cn: 如果它走到一个 **accessed** 的instance，那么我们知道找到了一个环:
    cn: 再沿着这个循环走一遍，找到环上具有最小`seq`的节点`y`,
    cn: 从环中删掉以`y`起始的边.

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

-   `|G|`:
    en: the number of vertices in a set or a graph `G`.
    cn: 表示集合G或图G中的顶点数。

-   en: With given instances `x, y`, if x depends on y, there is an edge: `(x, y)`, the weight of the edge is: `(x.seq, y.seq)`.
-   cn: 对2个instance`x, y`, 如果x依赖y:
    cn: 则定义一条边edge: `(x, y)`,
    cn: 边的权重为: `(x.seq, y.seq)`.

-   **min-edge**:
    en: is the edge with the smallest `weight`.
    cn: 具有最小权重的边:

    - en: for a vertex, it is the smallest of all outgoing edges:
    - cn: 对一个vertex, 它是所有出向边中最小的:
      `MinEdge(x) = MinEdge(x.edges) = e: e ∈ x.edges ∧ weight(e) is smallest`

    - en: for a graph, it is the smallest of all edges in the graph:
    - cn: 对一个图, 它是所有图中边中最小的:
      `MinEdge(G) = MinEdge(G.edges) = e: e ∈ G.edges ∧ weight(e) is smallest`

-   **min-cycle** `Cᵢ(x, G)`:
    en: is the i-th cycle found by walking along **min-edge**s from vertex `x`.
    cn: 从`x`出发, 沿着**min-edge**遍历发现的第i个环。

    en: `C(G)`: is the set of all **unrelated** cycles in `G`:
    cn: `C(G)`图`G`中所有 **不相关** 的环的集合:
    `C(G) = {C₁(x, G) | x ∈ G.vertices}`

    en: Refer to **Lemma-cycle-DAG** for a definition of **related**.
    cn: 环的相关性参考**Lemma-cycle-DAG**.

-   key-vertex: `Kᵢ(x, G)`:
    en: is the i-th **key-vertex** found by walking along **min-edge**s from vertex `x`.
    cn: 从`x`开始遍历图`G`找到的第i个关键顶点.

    en: key-vertex: is the min-edge of cycle `Cᵢ(x, G)`:
    cn: 关键顶点 key-vertex 定义为环`Cᵢ(x, G)`中的**min-edge**的源顶点:
    `Kᵢ(x, G) = u: (u, v) = MinEdge(Cᵢ(x, G))`

-   en: `Gᵢ`: the graph obtained by removing all cycle `C(Gᵢ₋₁)` and all vertices
    en: without outgoing edge from `Gᵢ₋₁`.
-   cn: `Gᵢ`: 从`Gᵢ₋₁`中删掉所有没有出向边, 和所有环`C(Gᵢ₋₁)`中的min-edge后得到的图.
    en: And `G₀ = G`.
    cn: 其中`G₀ = G`.

-   en: `N₀(G)`: non-key-vertex: the set of all non key-vertex.
-   cn: `N₀(G)`: non-key-vertex: key-vertex之外的其他顶点集合:

-   en: Walking path `P(x, G)`:
-   cn: 路径 `P(x, G)`:
    en: the sequence of vertices obtained by walking along **min-edge** from `x`.
    cn: 在图`G`中, 从`x`沿着**min-edge**遍历而经过的顶点序列,

-   en: Effective path `Pe(x, G)`:
    en: the result of removing all cycles from `P(x, G)`.
    cn: 有效路径, effective path: `Pe(x, G)`:
    cn: 从`P(x, G)`中去掉经过的环得到的路径.

en: For an example of the concepts above, the vertices in it are named with `seq`:
cn: 关于以上概念的一个例子, 图中顶点以`seq`命名:

```
     .-4<-.           .> 9
     v    |          /
1 -> 6 -> 3 -> 5 -> 2 -> 8
     ^              |
     `--------------'

Pe(5, G) = [5, 2,                                  8]
P(5, G)  = [5, 2, 6, 3, 4, 6, 3, 5, 2, 6, 3, 5, 2, 8]
                        ------- C₁(5, G)
                  ====           ==== C₂(5, G)

Pe(1, G) = [1, 6, 3,          5, 2,             8]
P(1, G)  = [1, 6, 3, 4, 6, 3, 5, 2, 6, 3, 5, 2, 8]
                     ------- C₁(1, G)
                                    ========== C₂(1, G)

K₁(1, G) = 3
K₂(1, G) = 2
K₃(1, G) = 8
N₀(G)    = {1, 6, 4, 5}

G₁: // MinEdge(C₁) and 8, 9 are removed.
     .-4
     v
1 -> 6 -> 3 -> 5 -> 2
     ^              |
     `--------------'

G₂: // empty, all vertices are removed.
```


# Proof


## Proof: execution linearizability

en: If `u ~ v` and `u` is initiated after `v` is committed, then `u.seq > v.seq`.
cn: 如果 `u ~ v` 且 `u` 在 `v` 之后被初始化,
cn: 那么一定有 `u.seq > v.seq`.

∴ en: If `u.seq <= v.seq`, `u` does not have to execute after `v` thus removing
en: a **min-edge** does not affect linearizability.
∴ cn: 如果 `u.seq <= v.seq`, `u` 不需要在 `v` 之后被执行.
cn: 因此去掉这样一个 **min-edge** `(u, v)` 不会影响linearizability.

QED.


## Lemma-cycle-DAG


en: Walking from any vertex, the order of the discovering cycles satisfies a
en: topological order `DAG(C)`.
cn: 从任一vertex出发的遍历, 发现环的顺序都满足一个的拓扑顺序`DAG(C)`.

en: First find this DAG:
cn: 首先找到这个DAG:

1.  en: From every vertex, before removing any min-edge, the walking found a set of cycle `C(G₀)`.
    en: Easy to see that cycles in `C(G₀)` are unrelated to each other:
    en: E.g., walking through two cycles `cᵢ, cⱼ ∈ C(G₀)` in any order
    en: always removes the same **min-edge**s.
1.  cn: 从所有节点开始遍历, 删除最小边之前, 遍历先发现一个环的集合`C(G₀)`, 容易看出,
    cn: `C(G₀)`中的环是互不相关的:
    cn: 即, 从一个vertex开始的遍历先后访问这两个环`cᵢ, cⱼ ∈ C(G₀)`,
    cn: 最终被删除的环中的min-edge都一样.

2.  en: After removing min-edge and from `C(G₀)`
    en: and the vertices without outgoing edges
    en: (walking to these vertices is a simple backtrace, which does not affect cycle discovery).
2.  cn: 将`C(G₀)`中的环内的min-edge删除, 由此产生的没有出向边的顶点也删除,
    cn: 因为遍历这些顶点是一个简单的回溯, 和环无关.

    en: In the resulting graph `G₁` continue walking as described in step 1, find another cycle set `C(G₁)`.
    en: Cycles in `C(G₁)` are not related to each other, but may depends on cycles in `C(G₀)`.
    cn: 在剩下的图`G₁`中继续步骤1, 找到环的集合`C(G₁)`.
    cn: `C(G₁)`中的环互相不相关, 但依赖`C(G₀)`中的环.

    en: `cᵢ -> cⱼ`, depend-on for cycle is defined as:
    en: cᵢ depends on cⱼ if cⱼ can not be walked through if `MinEdge(cⱼ)` is not
    en: removed.
    cn: 环依赖的定义为: `cᵢ -> cⱼ`,
    cn: cᵢ 依赖 cⱼ 如果`MinEdge(cⱼ)`删除前无法遍历出`cᵢ`.

    en: Easy to see the cycles in `C(Gᵢ)` does not depend on each other.
    cn: 容易看出同一个Gᵢ中发现的所有环`C(Gᵢ)`都不互相依赖.

en: Repeat until all vertices in `G` are removed.
cn: 重复直到G中所有节点都被删除.

en: Then we find a `DAG` of the dependency relationship between cycles.
cn: 由此可以得到一个关于环之前依赖关系的DAG.

en: The order of cycle discovery is in the topological order defined by `DAG(C)` :
en: A walking, since walked to a vertex on a cycle `c`, have to remove all the **min-edge**s of the cycles that `c` depends on, before discovering `c`.
cn: 环的发现顺序遵循`DAG(C)`的依赖关系:
cn: 任一遍历, 在访问到某个环上的1个vertex之后,
cn: 必须将所有其依赖的环中最小边都删除才能找到`c`

∴ en: The order of cycle discovery satisfies the topological order of `DAG(C)` no
en: matter from what vertex the walking starts from.
∴ cn: 从任一vertex出发的遍历发现环的顺序都满足DAG(C)的拓扑顺序.

QED.

Example:
```
     .-4<-.           .> 9
     v    |          /
1 -> 6 -> 3 -> 5 -> 2 -> 8
     ^              |
     `--------------'

c1 = [3, 4, 6]
c2 = [6, 3, 5, 2]

c2 -> c1
```

## Lemma-dep-DAG

en: The effective path through the instance dependency graph `G`, is equivalent to walking through `DAG(G)` which is obtained by removing all min-edge from `G`.
cn: 在instance 依赖图G中的遍历的effective path,
cn: 等价于遍历去掉所有min-edge后得到的`DAG(G)`.

1.  en: By **Lemma-cycle-DAG**,
1.  cn: 从**Lemma-cycle-DAG** 看出,
    cn: 所有遍历, 不论从哪里开始, 可能互相影响的cycle被发现的顺序都一样.

    ∴ en: The set of the smallest edges removed is the same for any walking.
    ∴ cn: 对任一遍历, 删除的最小边的集合都一样.

2.  en: After removing all **min-edge**s, `G` becomes a DAG: `DAG(G)`,
2.  cn: 删除所有最小边后`G`变成一个DAG: `DAG(G)`,

3. en: By 1 and 2, `DAG(G)`s for all walking are the same.
3. cn: 由1, 2得出任一遍历得出的`DAG(G)`都一样.

4.  en: With a given **min-edge** `e = MinEdge(Cᵢ(x, G))`:
    en: A walking goes back to the source vertex of `e` after walking through the
    en: containing cycle `Cᵢ(x, G)`.
4.  cn: 对一个环中的min-edge `e = MinEdge(Cᵢ(x, G))`:
    cn: 遍历走过一个`e`后一定会回到`e`的起点,

    ∴ en: The effective path of a walking is the same whether `e` exists or not.
    ∴ cn: 不论`e`存在与否, 遍历的effective path 都一样, 等同于`e`不存在时的遍历.

en: By 3 and 4:
en: the effective path of a walking in `G` is equivalent to a walking in `DAG(G)`.
cn: 从3, 4 得到:
cn: 遍历算法的effective path等价于是直接遍历`DAG(G)`.

QED.

Example:
```
     .-4<-.           .> 9
     v    |          /
1 -> 6 -> 3 -> 5 -> 2 -> 8
     ^              |
     `--------------'

Pe(5, G) = [5, 2,                      8]
P(5, G)  = [5, 2, 6, 3, 4, 6, 3, 5, 2, 8]
                        ------- C₁(5, G)
                  ====           ==== C₂(5, G)

Pe(1, G) = [1, 6, 3,          5, 2,             8]
P(1, G)  = [1, 6, 3, 4, 6, 3, 5, 2, 6, 3, 5, 2, 8]
                     ------- C₁(1, G)
                                    ========== C₂(1, G)
DAG(G):
     .-4              .> 9
     v               /
1 -> 6 -> 3 -> 5 -> 2 -> 8
```


## Proof: execution concurrency

en: Two concurrent walks do not affect each other.
cn: 两个并发的遍历不会相互影响。

en: By **Lemma-dep-DAG**，
en: The walking in an SCC `G` is the same as the walking in `DAG(G)`.
en: And since a walking only select min-edge.

en: As long as 2 walking process atomically delete vertex/edge, they execute instance in the same topological order of `DAG(G)`.
cn: 根据 **Lemma-dep-DAG**，
cn: 对SCC的遍历可以看做对`DAG(G)`的遍历.
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


## Proof: execution consistency

en: Two dependent instances on any replica will be executed in the same order.
cn: 任一replica上有依赖的2个instance执行顺序相同.

en: Because an instance only records the largest `dep` of a leader, Thus when executing, in addition to a `dep` of an instance, it also need to find out all the indirect dependencies that have the same leader
en: of `dep`.
cn: 因为实现中只记录了一个leader上最大的dep,
cn: 因此在执行时, 除了找到instance记录的dep之外,
cn: 还需要找出所有省略的依赖: 即同一leader上其他instnace-id更小的instance.

1.  en: Two interfering vertices `x, y` has at least one path in `DAG(G)`:
1.  cn: 互相依赖的vertex `x, y`在`DAG(G)`中至少有1条路径:

    en: If instance x depends on instance y, there is an edge `x -> y` in `G`.
    cn: 如果instance x依赖instance y, 那么x和y在`G`中有一条边.

    ∵ en: The min-edge is removed only when there is a cycle through `x, y`.
    ∵ cn: 最小边的删除只在有环的时候发生.

    ∴ en: There is at least one path from `x` to `y` after removing the min-edge
    en: from the cycle.
    ∴ cn: 删除最小边之后x和y至少还有一条路径.

2.  en: If there is a path between `x, y` in `DAG(G)`, their order is determined by this path.
2.  cn: 对`DAG(G)`的遍历, 如果`x, y`之间有一条路径, 那他们的先后顺序由这个路径决定.

3.  en: By **Lemma-dep-DAG**, the effective path of a walking in `G` is equivalent to the walking in `DAG(G)`.
3.  cn: 从**Lemma-dep-DAG** 得到, 遍历的effective path等同于对DAG(G)的遍历

∴ en: By 1, 2, 3:
en: two interfering instances have the same execution order on every replica.
en: The order is the same as topological order of `x` and `y` in `DAG(G)`.
∴ cn: 从1, 2, 3: 任一replica上有依赖的2个instance执行顺序相同,
cn: 都等同于`DAG(G)`中x和y的拓扑顺序.

QED.


## Lemma-persistent-edge

en: With a given edge `(u, v)`, if `u.seq > v.seq`, this edge will not be removed until an vertex is executed and removed.
cn: 对一条边`(u, v)`, 如果`u.seq > v.seq`
cn: 且遍历过程中没有vertex被执行并删除,
cn: 那这条边就不会被删除.

en: For two adjacent edges `(u, v)` and `(v, w)`:
en: we have that
cn: 对2条edge `(u, v)` 和 `(v, w)`:
`weight(u, v) = (u.seq, v.seq) > weight(v, w) = (v.seq, w.seq)`.

∴ en: In any cycle that contains edge `(u, v)`, the weight of `(u, v)` will never be the smallest.
∴ cn: 任何一个包含`(u, v)`的环中, `(u, v)`的weight 一定不是最小的.

∴ en: `(u, v)` will not be removed as a **min-edge**.
∴ cn: `(u, v)` 不会被删除.
en: Since there is no vertex is executed and removed, there is also no other way
en: this edge will be removed.

QED.


## Lemma-key-vertex-mapping

en: With a walking from vertex `x₀`, the i-th key-vertex found, `xᵢ = Kᵢ(x₀, G)`
en: and `G'` that is obtained by removing several **min-edge**:
en: before any vertex is executed and removed, we have:
en: there is at least one edge `(n₀, xᵢ)` in `G'`, where `n₀ ∈ N₀(G)`.
en: And `|N₀| >= 0.5 * |G|`.
cn: 从`x₀`开始遍历,
cn: 在还没有执行并删除一个节点之前,
cn: 对遍历到的一个key-vertex, `xᵢ = Kᵢ(x₀, G)`, 和此时删掉若干边后得到的`G'`:
cn: `G'`中至少包含一个边`(n₀, xᵢ)`, 这里 `n₀ ∈ N₀(G)`.
cn: 且 `|N₀| >= 0.5 * |G|`

1.  en: Case 1: `Cᵢ(x₀, G)` does not contain any key vertex: `Kⱼ(x₀, G), j<i`:
1.  cn: 当 `Cᵢ(x₀, G)` 除了`xᵢ`外, 不包含任何 key-vertex: `Kⱼ(x₀, G), j<i`:

    en: Because a vertex does not depend on itself, a cycle has at least 2 vertices in it.
    cn: 因为一个顶点并不依赖它自己, 一个环至少有两个顶点.

    ∴ en: `Cᵢ(x₀, G)` has at least one `n₀ ∈ N₀(G)`.
    ∴ cn: `Cᵢ(x₀, G)`中至少包含一个`N₀`中的vertex: `n₀ ∈ N₀(G)`.

    ∴ en: With `n₀.seq > xᵢ.seq`, by **Lemma-persistent-edge** we see that
    en: there is an edge `(n₀, xᵢ)` in `G'`.
    ∴ cn: 而`n₀.seq > xᵢ.seq`, 从**Lemma-persistent-edge** 可知,
    cn: `(n₀, xᵢ)` 存在于`G'`中.

2.  en: Case 2: `Cᵢ(x₀, G)` contains some key vertex `Kⱼ(x₀, G), j<i`:
2.  cn: 当 `Cᵢ(x₀, G)` 除了`xᵢ`外, 包含一个 key-vertex: `Kⱼ(x₀, G), j<i`:

    en: We will see that along `Cᵢ(x, G)`, the vertex before `xᵢ` must be a vertex in `N₀(G)`, e.g.  `(n₀, xᵢ): n ∈ N₀(G)`.
    cn: 那么我们将看到, 在 `Cᵢ(x, G)` 这个环上, 节点`xᵢ` 之前的节点一定是一个 `N₀(G)` 中的节点:
    cn: `(n₀, xᵢ): n ∈ N₀(G)`.

    en: Assumes:
    cn: 假设:
    - en: there is an edge `(xⱼ, xᵢ)` in cycle `Cᵢ(x₀, G)`, where `xⱼ = Kⱼ(x₀, G), j<i`,
    - cn: `Cᵢ(x₀, G)` 中存在一条边 `(xⱼ, xᵢ)`, 满足 `xⱼ = Kⱼ(x₀, G), j<i`,
    - en: and edge `(xⱼ, a)` is the removed min-edge in cycle `Cⱼ(x₀, G)`.
    - cn: 以及 `(xⱼ, a)` 是`Cⱼ(x₀, G)` 中删除的min-edge,

    en: Then we have:
    cn: 那么我们可以得到:

    -  en: `xᵢ.seq < xⱼ.seq` : because `xᵢ` has smallest `seq` in cycle `Cᵢ(x₀, G)`.
    -  en: `xⱼ.seq < a.seq`  : because `xⱼ` has smallest `seq` in cycle `Cⱼ(x₀, G)`.
    -  en: `a.seq < xᵢ.seq`  : because `(xⱼ, a)` has been chosen before `(xⱼ, xᵢ)`.
    -  cn: `xᵢ.seq < xⱼ.seq` : 因为 `xᵢ` 在 `Cᵢ(x₀, G)` 中有最小的`seq`.
    -  cn: `xⱼ.seq < a.seq`  : 因为 `xⱼ` 在 `Cⱼ(x₀, G)` 中有最小的`seq`.
    -  cn: `a.seq < xᵢ.seq`  : 因为 `(xⱼ, a)` 先被选了, 然后才选择的 `(xⱼ, xᵢ)`.

    ```
    .-- .. --.
    |        v
    '-- a <- xⱼ -> xᵢ --.
             ^          |
             `-- .. ----'
    ```

    en: It is impossible to form a cycle with greater-than relation.
    en: Thus our assumption about edge `(xⱼ, xᵢ)` does not hold.
    cn: 大于关系不可能形成环, 所以我们关于边`(xⱼ, xᵢ)`存在的假设不成立.

    ∴ en: There must be an edge `(n₀, xᵢ)` in `Cᵢ(x₀, G)` where `n₀ ∈ N₀(G)`,
    ∴ cn: `Cᵢ(x₀, G)` 中必须存在一个边 `(n₀, xᵢ)` 并且 `n₀ ∈ N₀(G)`,

    ∴ en: With `n₀.seq > xᵢ.seq`, by **Lemma-persistent-edge** we see that
    en: there is an edge `(n₀, xᵢ)` in `G'`.
    ∴ cn: 而`n₀.seq > xᵢ.seq`, 从**Lemma-persistent-edge** 可知,
    cn: `(n₀, xᵢ)` 存在于`G'`中.

∴ en: From 1 and 2, for `i>0`, every `Kᵢ(x₀, G)` has a vertex in `N₀(G)` before it in `G'`.
∴ cn: 通过 1 和 2,
cn: 在遍历过程中的一个图`G'`中,
cn: 每个 key-vertex `Kᵢ(x₀, G)` 都有一个 `N₀(G)` 中的 vertex 在它之前.

∵ en: In `G'` every `n₀ ∈ N₀(G)` has only one outgoing edge
∵ cn: `G'`中沿着min-edge的遍历任一节点只有一个出向边,

∴ en: There is a one-to-one map from key-vertex and non-key-vertex.
∴  cn: key-vertex 和 non-key-vertex 之间存在一个一一映射.

∴ `|N₀(G)| >= 0.5 * |G|`

QED.



## Proof: execution in finite number of steps

en: In an infinite SCC, starting from an instance with a finite `seq`, this algorithm can always find the first instance to execute in a finite number of steps.
cn: 在一个无限大的SCC中, 从一个有限seq的instance开始,
cn: 总是能在有限步数内找到第一个执行的instance.

**Define**:
`Seqs(x) = {y.seq | x -> y}`

en: By **Lemma-key-vertex-mapping**, in the worst case, when going to the next vertex, it has the same chance of choosing either a key-vertex or a non-key-vertex.
cn: 通过 **Lemma-key-vertex-mapping**,
cn: 在最差情况下,
cn: 当遍历到下一个节点时, 它有相同的几率选择到一个key-vertex 或 non-key-vertex.

en: Assuming the value of `Seqs(x)` is uniformly distributed around `x.seq`.
en: The `seq` of next vertex has a 50% chance of being `min(Seqs(x))`(non-key-vertex)
en: and 50% chance of being `max(Seqs(x))`(key-vertex, the worst case is the max
en: edge is chosen).
cn: 假设`Seqs(x)`的值在`x.seq`附近两侧均匀分布，
cn: 一个顶点的`seq`有50%的概率是`min(Seqs(x))`(选到non-key-vertex),
cn: 50%是`max(Seqs(x))`(选到key-vertex, key-vertex的出向边被删除了,
cn: 于是最差情况是选择到最大的出向边).

en: The change in `seq` can be simplified to a process of [Random-walk].
cn: 遍历中, `seq`的变化可以简化成一个随机行走的过程: [Random-walk]

en: A random walk from the origin access any point within a finite distance in a
en: finite number of steps(Refer to next chapter for a proof).
∵ cn: 从原点开始的随机游走以在有限步数内访问到有限距离内的任意一个点(后面证明)。

∴ en: No matter from what vertex it starts the algorithm, It only takes a finite number of steps to find a vertex to execute.
∴ cn: 无论从哪个vertex开始执行算法，
cn: 它只需要有限步来找到一个顶点来执行。

en: However, having 50% chance of choosing `max(Seqs(x))` is just the worst case.
cn: 然而，有50%的机会选择`max(Seqs(x))`只是最坏的情况。

en: A realistic scenario would be:
en: The walking quickly converges to lower `seq` region.
en: Then it walks around the region until finding a vertex to execute.
cn: 一个现实的场景是:
cn: 遍历很快就会收敛到较低的`seq`区域，
cn: 然后在这个区域继续遍历，直到找到一个要执行的instance。

QED.

### Random walk guarantees that it only takes a finite number of steps to get to a point within a finite distance

en: The distribution of the position after k steps random walk, is derived from Pascal's triangle:
cn: k步随机游走后的位置分布， 可以从杨辉三角形得到:

```

     k     −5   −4  −3   −2   −1   0   1    2   3    4   5
  P[S₀=k]                          1
 2P[S₁=k]                     1        1
2²P[S₂=k]                 1        2        1
2³P[S₃=k]            1        3        3        1
2⁴P[S₄=k]        1        4        6        4        1
2⁵P[S₅=k]    1       5        10       10       5        1
```

en: After several steps, the probability of reaching the point `2n` is:
cn: 经过若干步后，到达点`2n`的概率为(`2n+1`位置的分析类似):

- en: after 2n steps: `C(2n, 2n)/2²ⁿ`
- en: after 2n+2 steps: `C(2n+2, 2n+1)/2²ⁿ⁺²`
- en: after 2n+4 steps: `C(2n+4, 2n+2)/2²ⁿ⁺⁴`
- cn: 2n   步之后: `C(2n, 2n)/2²ⁿ`
- cn: 2n+2 步之后: `C(2n+2, 2n+1)/2²ⁿ⁺²`
- cn: 2n+4 步之后: `C(2n+4, 2n+2)/2²ⁿ⁺⁴`
- ...

en: `C(m, n) = m!/n!/(m-n)!` is the combination number of choosing n items from m items.
cn: `C(m, n) = m!/n!/(m-n)!`: 从m个中选择n个的组合数。

∴ en: the expected total number of times reaching the point `2n` is:
∴ cn: 停在`2n`的总的期望次数是:

```
       k
E(s) = ∑  C(2n+2i, 2n+i)/2²ⁿ⁺²ⁱ
       i=0

```

en: And approximated by [Stirling-approximation]:
cn: 通过[Stirling-approximation]近似得到:


```
       k       √(2n+2i)
E(s) = ∑   ----------------- (1-n/(2n+i))²ⁿ⁺²ⁱ (1+n/i)ⁱ
       i=0  √(2π) √(2n+i) √i
```

en: When `i` becomes large, by `(1+1/i)ⁱ = e`, it is approximated to
cn: 当 `i` 变得很大时, 通过 `(1+1/i)ⁱ = e`, 将它近似为:

```
         1   k      1
E(s) = ----- ∑   -------
       √π eⁿ i=0 √(2n+i)

```

en: Easy to see that when `i` becomes big enough, `E(s)` does not converge.
en: Thus we can always find a finite `k` that satisfies:
en: `E(s) > n`, for any finite number `n`.
cn: 可以看出当i足够大后, `E(s)` 不收敛.
cn: 这意味着，对于任何需要的经过次数，我们总能找到一个达到这个访问次数的`k`的值。

[Random-walk]: https://en.wikipedia.org/wiki/Random_walk
[Stirling-approximation]: https://en.wikipedia.org/wiki/Stirling's_approximation
