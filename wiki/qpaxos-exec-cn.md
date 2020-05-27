> `instance` and `vertex` are equivalent concepts in this doc.
>
> DAG: directed acyclic graph.

# Algorithm

令 `x.seq=(x.seq, x.leader_id, x.index)`, 这样任何两个 instance 都有不同的 `seq`.

选择一个未执行的 instance `x`.

深度优先遍历，
选择具有 **最小** `seq` 的下一个 instance。
将已访问的 instance 标记为 **accessed**。

- 如果它走到一个没有出向边的 instance，
  执行并删除它，
  然后回溯到先前的 instance 并继续。

- 如果它走到一个 **accessed** 的 instance，那么我们知道找到了一个环:
  再沿着这个循环走一遍，找到环上具有最小`seq`的节点`y`,
  从环中删掉以`y`起始的边.

  然后从`y`继续。

## Example of execution

这个算法的一个特点是
在执行某个 instance 之前，它不必遍历 SCC 中所有的环。

如下图所示:

```
          .-> 4
          |
1 -> 6 -> 3 -> 5 -> 2 -> 8
     ^              |
     `--------------'
```

从 `1` 开始执行，它通过**min-edge**遍历到 `4`,
发现`4`是第一个应该被执行的,
这时遍历过的有效路径为`[1, 6, 3, 4]`,
环 `6, 3, 5, 2` 还没被看到.

`4` 被执行并从图中删掉.
然后回溯到`3`继续遍历:

```
1 -> 6 -> (3) -> 5 -> 2 -> 8
     ^                |
     `----------------'
```

继续遍历, 找到一个环 `6, 3, 5, 2`,
然后删掉环中的最小 vertex 对应的边 `(2, 6)`,
这时遍历过的有效路径为`[1, 6, 3, 4, 3, 5, 2]`(环不计入).

继续从`2`开始遍历, 最终找到`8`作为第 2 个被执行的 instance:

然后再回溯到`2, 5, 3, 6, 1`, 并依次执行.

## 可替换概念

这个算法也适用于不使用 seq 的场景, 只需要实现满足以下条件:

- 可以区分出哪些边是可以删除的, 以保证 linearizability 不被破坏.
- 在不同的 replica 上有确定的方式决定删除哪些边(例如选择 instance-id 最小的).
  并且选择的方式需要让遍历趋向于优先遍历到较早被 propose 的 instance.

只要满足以上条件, 就可以使用同样的算法确定执行顺序.
本文档中是通过`seq`来实现以上保证.

# Properties

除了 linearizability、consistency 和避免 livelock，
它还提供了一些其他保证:

- 任何时候停止执行都是安全的。SCC 中的 instance 不需要在一个事务中执行。

- 允许多个线程同时执行。
  惟一的要求是对 instance 的修改(删掉边或删掉 instance 的操作)必须是原子性的。

- 时间复杂度可以优化到`O(2m)`左右，其中 m 为 instance 数。

# Definition

- `G = (vertices, edges)`, 边有权重.

- `|G|`:
  表示集合 G 或图 G 中的顶点数。

- 对 2 个 instance`x, y`, 如果 x 依赖 y:

  则定义一条边 edge: `(x, y)`,
  边的权重为: `(x.seq, y.seq)`.

- **min-edge**:
  具有最小权重的边:

  - 对一个 vertex, 它是所有出向边中最小的:
    `MinEdge(x) = MinEdge(x.edges) = e: e ∈ x.edges ∧ weight(e) is smallest`

  - 对一个图, 它是所有图中边中最小的:
    `MinEdge(G) = MinEdge(G.edges) = e: e ∈ G.edges ∧ weight(e) is smallest`

- **min-cycle** `Cᵢ(x, G)`:
  从`x`出发, 沿着**min-edge**遍历发现的第 i 个环。

  `C(G)`图`G`中所有 **不相关** 的环的集合:
  `C(G) = {C₁(x, G) | x ∈ G.vertices}`
  环的相关性参考**Lemma-cycle-DAG**.

- key-vertex: `Kᵢ(x, G)`:
  从`x`开始遍历图`G`找到的第 i 个关键顶点.

  关键顶点 key-vertex 定义为环`Cᵢ(x, G)`中的**min-edge**的源顶点:
  `Kᵢ(x, G) = u: (u, v) = MinEdge(Cᵢ(x, G))`

- `Gᵢ`: 从`Gᵢ₋₁`中删掉所有没有出向边, 和所有环`C(Gᵢ₋₁)`中的 min-edge 后得到的图.

  其中`G₀ = G`.

- `N₀(G)`: non-key-vertex: key-vertex 之外的其他顶点集合:

- 路径 `P(x, G)`:
  在图`G`中, 从`x`沿着**min-edge**遍历而经过的顶点序列,

  有效路径, effective path: `Pe(x, G)`:
  从`P(x, G)`中去掉经过的环得到的路径.

关于以上概念的一个例子, 图中顶点以`seq`命名:

```
     .-4<-.           .> 9
     v    |          /
1 -> 6 -> 3 -> 5 -> 2 -> 8
     ^              |
     `--------------'

Pe(5, G) = [5, 2,                      8]
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

如果 `u ~ v` 且 `u` 在 `v` 之后被初始化,
那么一定有 `u.seq > v.seq`.

∴ 如果 `u.seq <= v.seq`, `u` 不需要在 `v` 之后被执行.
因此去掉这样一个 **min-edge** `(u, v)` 不会影响 linearizability.

QED.

## Lemma-cycle-DAG

从任一 vertex 出发的遍历, 发现环的顺序都满足一个的拓扑顺序`DAG(C)`.

首先找到这个 DAG:

1.  从所有节点开始遍历, 删除最小边之前, 遍历先发现一个环的集合`C(G₀)`, 容易看出,
    `C(G₀)`中的环是互不相关的:
    即, 从一个 vertex 开始的遍历先后访问这两个环`cᵢ, cⱼ ∈ C(G₀)`,
    最终被删除的环中的 min-edge 都一样.

2.  将`C(G₀)`中的环内的 min-edge 删除, 由此产生的没有出向边的顶点也删除,
    因为遍历这些顶点是一个简单的回溯, 和环无关.

    在剩下的图`G₁`中继续步骤 1, 找到环的集合`C(G₁)`.
    `C(G₁)`中的环互相不相关, 但依赖`C(G₀)`中的环.

    环依赖的定义为: `cᵢ -> cⱼ`,
    cᵢ 依赖 cⱼ 如果`MinEdge(cⱼ)`删除前无法遍历出`cᵢ`.

    容易看出同一个 Gᵢ 中发现的所有环`C(Gᵢ)`都不互相依赖.

重复直到 G 中所有节点都被删除.

由此可以得到一个关于环之前依赖关系的 DAG.

环的发现顺序遵循`DAG(C)`的依赖关系:
任一遍历, 在访问到某个环上的 1 个 vertex 之后,
必须将所有其依赖的环中最小边都删除才能找到`c`

∴ 从任一 vertex 出发的遍历发现环的顺序都满足 DAG(C)的拓扑顺序.

QED.

例如:

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

在 instance 依赖图 G 中的遍历的 effective path,
等价于遍历去掉所有 min-edge 后得到的`DAG(G)`.

1.  从**Lemma-cycle-DAG** 看出,
    所有遍历, 不论从哪里开始, 可能互相影响的 cycle 被发现的顺序都一样.

    ∴ 对任一遍历, 删除的最小边的集合都一样.

2.  删除所有最小边后`G`变成一个 DAG: `DAG(G)`,

3.  由 1, 2 得出任一遍历得出的`DAG(G)`都一样.

4.  对一个环中的 min-edge `e = MinEdge(Cᵢ(x, G))`:
    遍历走过一个`e`后一定会回到`e`的起点,

    ∴ 不论`e`存在与否, 遍历的 effective path 都一样, 等同于`e`不存在时的遍历.

从 3, 4 得到:
遍历算法的 effective path 等价于是直接遍历`DAG(G)`.

QED.

例如:

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

两个并发的遍历不会相互影响。

根据 **Lemma-dep-DAG**，
对 SCC 的遍历可以看做对`DAG(G)`的遍历.
又因为遍历中只选择 min-edge,
只要 2 个遍历进程原子的删除 vertex/edge,
它们执行 instance 的拓扑顺序都是一致的.

QED.

## Proof: incremental execution

执行过程可以在任何时候安全地停止再重新启动。
它不需要一次性地执行 SCC 中所有的 instance。

重启的执行过程与以下场景等同:
一个正在运行的执行进程和另一个永远被暂停的进程。

QED.

## Proof: execution consistency

任一 replica 上有依赖的 2 个 instance 执行顺序相同.

因为实现中只记录了一个 leader 上最大的 dep,
因此在执行时, 除了找到 instance 记录的 dep 之外,
还需要找出所有省略的依赖: 即同一 leader 上其他 instnace-id 更小的 instance.

1.  互相依赖的 vertex `x, y`在`DAG(G)`中至少有 1 条路径:

    如果 instance x 依赖 instance y, 那么 x 和 y 在`G`中有一条边.

    ∵ 最小边的删除只在有环的时候发生.

    ∴ 删除最小边之后 x 和 y 至少还有一条路径.

2.  对`DAG(G)`的遍历, 如果`x, y`之间有一条路径, 那他们的先后顺序由这个路径决定.

3.  从**Lemma-dep-DAG** 得到, 遍历的 effective path 等同于对 DAG(G)的遍历

∴ 从 1, 2, 3: 任一 replica 上有依赖的 2 个 instance 执行顺序相同,
都等同于`DAG(G)`中 x 和 y 的拓扑顺序.

QED.

## Lemma-persistent-edge

对一条边`(u, v)`, 如果`u.seq > v.seq`
且遍历过程中没有 vertex 被执行并删除,
那这条边就不会被删除.

对 2 条 edge `(u, v)` 和 `(v, w)`:
`weight(u, v) = (u.seq, v.seq) > weight(v, w) = (v.seq, w.seq)`.

∴ 任何一个包含`(u, v)`的环中, `(u, v)`的 weight 一定不是最小的.

∴ `(u, v)` 不会被删除.

QED.

## Lemma-key-vertex-mapping

从`x₀`开始遍历,
在还没有执行并删除一个节点之前,
对遍历到的一个 key-vertex, `xᵢ = Kᵢ(x₀, G)`, 和此时删掉若干边后得到的`G'`:
`G'`中至少包含一个边`(n₀, xᵢ)`, 这里 `n₀ ∈ N₀(G)`.
且 `|N₀| >= 0.5 * |G|`

1.  当 `Cᵢ(x₀, G)` 除了`xᵢ`外, 不包含任何 key-vertex: `Kⱼ(x₀, G), j<i`:

    因为一个顶点并不依赖它自己, 一个环至少有两个顶点.

    ∴ `Cᵢ(x₀, G)`中至少包含一个`N₀`中的 vertex: `n₀ ∈ N₀(G)`.

    ∴ 而`n₀.seq > xᵢ.seq`, 从**Lemma-persistent-edge** 可知,
    `(n₀, xᵢ)` 存在于`G'`中.

2.  当 `Cᵢ(x₀, G)` 除了`xᵢ`外, 包含一个 key-vertex: `Kⱼ(x₀, G), j<i`:

    那么我们将看到, 在 `Cᵢ(x, G)` 这个环上, 节点`xᵢ` 之前的节点一定是一个 `N₀(G)` 中的节点:
    `(n₀, xᵢ): n ∈ N₀(G)`.

    假设:

    - `Cᵢ(x₀, G)` 中存在一条边 `(xⱼ, xᵢ)`, 满足 `xⱼ = Kⱼ(x₀, G), j<i`,
    - 以及 `(xⱼ, a)` 是`Cⱼ(x₀, G)` 中删除的 min-edge,

    那么我们可以得到:

    - `xᵢ.seq < xⱼ.seq` : 因为 `xᵢ` 在 `Cᵢ(x₀, G)` 中有最小的`seq`.
    - `xⱼ.seq < a.seq` : 因为 `xⱼ` 在 `Cⱼ(x₀, G)` 中有最小的`seq`.
    - `a.seq < xᵢ.seq` : 因为 `(xⱼ, a)` 先被选了, 然后才选择的 `(xⱼ, xᵢ)`.

    ```
    .-- .. --.
    |        v
    '-- a <- xⱼ -> xᵢ --.
             ^          |
             `-- .. ----'
    ```

    大于关系不可能形成环, 所以我们关于边`(xⱼ, xᵢ)`存在的假设不成立.

    ∴ `Cᵢ(x₀, G)` 中必须存在一个边 `(n₀, xᵢ)` 并且 `n₀ ∈ N₀(G)`,

    ∴ 而`n₀.seq > xᵢ.seq`, 从**Lemma-persistent-edge** 可知,
    `(n₀, xᵢ)` 存在于`G'`中.

∴ 通过 1 和 2,
在遍历过程中的一个图`G'`中,
每个 key-vertex `Kᵢ(x₀, G)` 都有一个 `N₀(G)` 中的 vertex 在它之前.

∵ `G'`中沿着 min-edge 的遍历任一节点只有一个出向边,

∴ key-vertex 和 non-key-vertex 之间存在一个一一映射.

∴ `|N₀| >= 0.5 * |G|`

QED.

## Proof: execution in finite number of steps

在一个无限大的 SCC 中, 从一个有限 seq 的 instance 开始,
总是能在有限步数内找到第一个执行的 instance.

**Define**:
`Seqs(x) = {y.seq | x -> y}`:

通过 **Lemma-key-vertex-mapping**,
在最差情况下,
当遍历到下一个节点时, 它有相同的几率选择到一个 key-vertex 或 non-key-vertex.

假设`Seqs(x)`的值在`x.seq`附近两侧均匀分布，
一个顶点的`seq`有 50%的概率是`min(Seqs(x))`(选到 non-key-vertex),
50%是`max(Seqs(x))`(选到 key-vertex, key-vertex 的出向边被删除了,
于是最差情况是选择到最大的出向边).

遍历中, `seq`的变化可以简化成一个随机行走的过程: [Random-walk]

∵ 从原点开始的随机游走以在有限步数内访问到有限距离内的任意一个点(后面证明)。

∴ 无论从哪个 vertex 开始执行算法，
它只需要有限步来找到一个顶点来执行。

然而，有 50%的机会选择`max(Seqs(x))`只是最坏的情况。

一个现实的场景是:
遍历很快就会收敛到较低的`seq`区域，
然后在这个区域继续遍历，直到找到一个要执行的 instance。

QED.

### Random walk guarantees that it only takes a finite number of steps to get to a point within a finite distance

k 步随机游走后的位置分布， 可以从杨辉三角形得到:

```

     k     −5   −4  −3   −2   −1   0   1    2   3    4   5
  P[S₀=k]                          1
 2P[S₁=k]                     1        1
2²P[S₂=k]                 1        2        1
2³P[S₃=k]            1        3        3        1
2⁴P[S₄=k]        1        4        6        4        1
2⁵P[S₅=k]    1       5        10       10       5        1
```

经过若干步后，到达点`2n`的概率为(`2n+1`位置的分析类似):

- 2n 步之后: `C(2n, 2n)/2²ⁿ`
- 2n+2 步之后: `C(2n+2, 2n+1)/2²ⁿ⁺²`
- 2n+4 步之后: `C(2n+4, 2n+2)/2²ⁿ⁺⁴`
- ...

`C(m, n) = m!/n!/(m-n)!`: 从 m 个中选择 n 个的组合数。

∴ 停在`2n`的总的期望次数是:

```
       k
E(s) = ∑  C(2n+2i, 2n+i)/2²ⁿ⁺²ⁱ
       i=0

```

通过[Stirling-approximation]近似得到:

```
       k       √(2n+2i)
E(s) = ∑   ----------------- (1-n/(2n+i))²ⁿ⁺²ⁱ (1+n/i)ⁱ
       i=0  √(2π) √(2n+i) √i
```

当 `i` 变得很大时, 通过 `(1+1/i)ⁱ = e`, 将它近似为:

```
         1   k      1
E(s) = ----- ∑   -------
       √π eⁿ i=0 √(2n+i)

```

可以看出当 i 足够大后, `E(s)` 不收敛.
这意味着，对于任何需要的经过次数，我们总能找到一个达到这个访问次数的`k`的值。

[random-walk]: https://en.wikipedia.org/wiki/Random_walk
[stirling-approximation]: https://en.wikipedia.org/wiki/Stirling's_approximation
