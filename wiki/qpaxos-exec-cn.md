> `instance` and `vertex` are equivalent concepts in this doc.

# Algorithm

令 `X.seq=(X.seq, X.leader_id, X.index)`, 这样任何两个instance都有不同的 `seq`.

选择一个未执行的instance `X`.

深度优先遍历，
选择具有 **最小** `seq` 的下一个instance。
将已访问的instance标记为**accessed**。

-   如果它走到一个没有出向边的instance，
    执行并删除它，
    然后回溯到先前的instance并继续。

-   如果它走到一个 **accessed** 的instance，那么我们知道找到了一个环:
    再沿着这个循环走一遍，找到环上具有最小`seq`的节点`Y`,
    从环中删掉`Y`起始的边.

    然后从`Y`继续。

## 可替换概念

这个算法也适用于不使用seq的场景, 只需要实现满足一下条件:

-   可以区分出哪些边是可以删除的, 以保证linearizability不被破坏.
-   在不同的replica上有确定的方式决定删除哪些边(例如选择instance-id最小的).
    并且选择的方式需要让遍历趋向于优先遍历到较早被propose的instance.

只要满足以上条件, 就可以使用同样的算法确定执行顺序.
本文档中是通过`seq`来实现以上保证.

# Properties

除了linearizability、consistency和避免livelock，
它还提供了一些其他保证:

-   任何时候停止执行都是安全的。SCC中的instance不需要在一个事务中执行。

-   允许多个线程同时执行。
    惟一的要求是对instance的修改(删掉边或删掉instance的操作)必须是原子性的。

-   时间复杂度可以优化到`O(2m)`左右，其中m为instance数。



# Definition

-   `|G|`:
    表示集合G或图G中的顶点数。

-   `Min(X.deps)`:
    X的依赖的instnace中具有最小`seq`的一个.

-   **min-edge**:
    从`X`指向`Min(X.deps)`的边.

-   **min-cycle** `C₁(X)`:
    沿着**min-edge**遍历发现的第一个环。
    一个没有出向边的顶点本身就是一个**min-cycle**。

-   `V₁(X)`: 第1轮中的关键顶点:
    环中具有最小`seq`的顶点.

-   `C₂(X)`: 删掉`C₁(X)`中从`V₁(X)`出发的边后, 沿着**min-edge**遍历发现的第2个环。
    ...

    `{C₁}`: 所有第1轮中找到的环的集合: `{C₁(X) | X ∈ G}`.
    `{C₂}`: 所有第2轮中找到的环的集合: `{C₂(X) | X ∈ G}`.
    ...

-   `Vᵢ(X)` 的定义跟 `V₁(X)` 类似.

    `{Vᵢ}`: 所有在第`i`轮找到的关键vertex:
    `{Vᵢ(X) | X ∈ G}`.

    `{V₀}`: 不在任何`{Vᵢ}, i >= 1`中的顶点集合

-   路径 `P(X)`:
    从`X`沿着**min-edge**遍历而经过的顶点序列。

-   关键路径`Pk(X)` (key path):
    从路径`P(X)`中删除所有的环而得到的

一个关于以上概念的例子, 图中顶点以`seq`命名:

```
     .-4<-.           .> 9
     v    |          /
1 -> 6 -> 3 -> 5 -> 2 -> 8
     ^              |
     `--------------'

P(1)  = [1, 6, 3, 4, 6, 3, 5, 2, 6, 3, 5, 2, 8]
Pk(1) = [1, 6, 3,          5, 2,             8]
C₁(1) = [         4, 6, 3]
C₂(1) = [                        6, 3, 5, 2]
V₁(1) = 3
V₂(1) = 2
V₃(1) = 8
{V₀}  = {1, 6, 4, 5}
{V₁}  = {3, 8, 9}
{V₂}  = {2}
{V₃}  = {8}
```

# Proof


## Lemma-path-intersect

如果两条路径 `P(X), P(Y)` 有一个公共顶点 `Z`,
从`Z`开始，它们都经过相同的顶点序列，
以及, 从 `Z` 开始 , `P(X), P(Y)` 有相同的 `Pk(), Cᵢ(), Vᵢ()`,
因为 `P(X)` 或 `P(Y)` 中的每个元素只有一个目标.

QED.


## Lemma-safe-remove

在执行exec算法之前, 删除一个不在任何`Pk(X)`中的边不会影响执行结果。

因为算法执行中遍历的目标仅由`Pk()`决定的,
并且, 由 **Lemma-path-intersect** 可知, 一个删除的边不会出现在任何`Pk()`中。

QED.


## Lemma-concurrency

两个并发的遍历不会相互影响。

-   如果两条路径没有交叉，它们不会相互影响

-   如果两条路径有交集，
    一个遍历会删除不在任何`Pk()`里的边,
    根据 **Lemma-safe-remove**， 这不影响其他遍历。

QED.


## Lemma-key-vertex-mapping

在一个SCC中, 对每一个`Vᵢ`中的节点, `X: X ∈ {Vᵢ}, i > 0`,
至少有一个路径`Pk`包含一个边`(U, X)`, 这里 `U ∈ {V₀}`.

1.  如果 `Cᵢ` 不包含任何`U: U ∈ {Vⱼ}, 0<j<i`:
    `Cᵢ` 至少有两个顶点,因为一个顶点并不依赖它自己,
    因此一个环至少有两个顶点。

    ∴ 存在一个key path `Pk` 包含边 `(A, X)`, 这里`A: A ∈ {V₀}`.

2.  如果 `Cᵢ` 包含一个`{Vᵢ}`节点U: `U: U ∈ {Vⱼ}, 0<j<i`:
    在 `Cᵢ` 这个环上, 节点`X` 之前的节点一定是一个 `V₀` 中的节点:
    `(U, X): U ∈ {V₀}`.

    假设: `Cᵢ` 中存在一条边 `(U, X)`, 满足 `U ∈ {Vⱼ}, 0<j<i`,
    以及边 `(U, A)` 是一条在`Cⱼ` 中删除的边,
    那么我们可以得到:

    -  `X.seq < U.seq` : 因为 `X` 在 `Cᵢ` 中有最小的`seq`.
    -  `U.seq < A.seq` : 因为 `U` 在 `Cⱼ` 中有最小的`seq`.
    -  `A.seq < X.seq` : 因为 `(U, A)` 先被选了, 然后才选择的 `(U, X)`.

    ```
    .-- .. --.
    |        v
    '-- A <- U -> X --.
             ^        |
             `-- .. --'
    ```

    大于关系不可能形成环, 所以我们关于边`(U, X)`的假设不成立.

    ∴ `Cᵢ` 中必须存在一个边 `(B, X)` 并且 `B ∈ {V₀}`,

    ∴ `(B, X)` 存在于某个key path `Pk`中.

∴ 通过 1 和 2, 对任意 `i>0`,
总能找到一个key path `Pk`, 使得对每个 `Vᵢ` vertex 都有一个 `V₀` vertex 在它之前.

∴  `{V₁} ∪ {V₂} ... ∪ {Vᵢ}` 和 `{V₀}` 之间存在一个一一映射.

∴ `|{V₀}| >= 0.5 * |G| >= |{V₁} ∪ {V₂} ∪ ... ∪ {Vᵢ}|`

QED.


## Proof: execution linearizability

如果  `U ~ V` 且 `U` 在 `V` 之后被初始化,
那么一定有 `U.seq > V.seq`.

∴ 如果 `U.seq <= V.seq`, `U` 不需要在 `V` 只有被执行.
因此去掉这样一个 **min-edge** `U->V` 不会影响linearizability.

QED.


## Proof: execution consistency

如果 `X -> Y`, `X` 被执行之前一定会先遍历到 `Y`.

∴ 根据 **Lemma-path-intersect**, 在`X` 或 `Y` 被执行之前,
后续的遍历都会找到相同的instance去执行.

∴ `X` 和 `Y` 在不同的replcia上总是有相同的执行顺序.

QED.


## Proof: execution concurrency

根据 **Lemma-concurrency**, 得到.

QED.


## Proof: incremental execution

执行过程可以在任何时候安全地停止再重新启动。
它不需要一次性地执行SCC中所有的instance。

Restarting execution process is just the same as
a running execution process and another process paused for ever.
By **Lemma-concurrency**, it does not affect execution behavior.

重启的执行过程与以下场景等同:
一个正在运行的执行进程和另一个永远被暂停的进程。

通过**Lemma-concurrency**，它不影响执行结果。

QED.


## Proof: execution in finite number of steps

**Define**:
`Seqs(X) = {Y.seq | X -> Y}`:

通过 **Lemma-key-vertex-mapping**,
在最差情况下,
当遍历到下一个节点时, 它有相同的几率选择到一个`V₀` 节点 或  `Vᵢ, i > 0` 节点.


假设`Seqs(X)`的值在`X.seq`附近均匀分布，
一个顶点的`seq`有50%的概率是`min(Seqs(X))`, 50%是`max(Seqs(X))`

`seq`的变化可以简化成一个随机行走的过程

从原点开始的随机游走以在有限步数内访问到有限距离内的任意一个点。

∴ 无论从哪个点开始执行算法，
它只需要有限步来找到一个顶点来执行。

然而，有50%的机会选择`max(Seqs(X))`只是最坏的情况。

一个现实的场景是:
遍历很快就会收敛到较低的`seq`区域，
然后在这个区域继续遍历，直到找到一个要执行的instance。

QED.


### Random walk guarantees that it only takes a finite number of steps to get to a point within a finite distance

k步随机游走后的位置分布， 可以从杨辉三角形得到:

```

     k     −5   −4  −3   −2   −1   0   1    2   3    4   5
  P[S₀=k]                          1
 2P[S₁=k]                     1        1
2²P[S₂=k]                 1        2        1
2³P[S₃=k]            1        3        3        1
2⁴P[S₄=k]        1        4        6        4        1
2⁵P[S₅=k]    1       5        10       10       5        1
```

经过若干步后，到达点`2n`的概率为:

- 2n   步之后: `C(2n, 2n)/2²ⁿ`
- 2n+2 步之后: `C(2n+2, 2n+1)/2²ⁿ⁺²`
- 2n+4 步之后: `C(2n+4, 2n+2)/2²ⁿ⁺⁴`
- ...

`C(m, n) = m!/n!/(m-n)!` 是从m个中选择n个的组合数。

∴ 停在`2n`的总的期望次数是:

```
       k
E(s) = ∑  C(2n+2i, 2n+i)/2²ⁿ⁺²ⁱ
       i=0

```

通过[Stirling-approximation]近似得到:

```
       k       √(2n+2i)
E(s) = ∑   -----------------  (1-n/(2n+i))²ⁿ⁺²ⁱ (1+n/i)ⁱ
       i=0  √(2π) √(2n+i) √i
```

当 `i` 变得很大时, 通过 `(1+1/i)ⁱ = e`, 将它近似为:

```
         1   k      1
E(s) = ----- ∑   -------
       √π eⁿ i=0 √(2n+i)

```

因此 `E(s)` 不收敛.
这意味着，对于任何需要的经过次数，我们总能找到一个达到这个访问次数的`k`的值。

[Random-walk]: https://en.wikipedia.org/wiki/Random_walk
[Stirling-approximation]: https://en.wikipedia.org/wiki/Stirling's_approximation
