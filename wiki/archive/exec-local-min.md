This a unfinished yet and can be proved to be an analogue to qpaxos-exec.md

# Execution

TODO: deps only records highest interfering, need to find the minimal. proof
that only directly deps  need to inherit the deps.

这个算法实现了有环depends-on graph中有限步的执行算法,
解决了livelock问题.

## Algorithm


从一个instance `x` 开始选择指向最小`ord`的边. 走到下一个节点.

-   如果遇到一个没有出向边的instance `z`, 执行`z`.

-   如果遇到一个环,删除环中最小ord的instance为起点的边.


## Terminology

- `a → b`: `a` depends-on `b`.
- `a ↦ b`: `a` execute after `b`.

### Def-ord

`ord` 是用于为SCC中instance排序的字段: deps的每个`instance_id`之和, 以及`instance_id`:
`ord = (seq, instance_id)`.

`seq` 是一个用来保证先后顺序的变量, 它必须满足:
that `a` is proposed after `b` is committed implies `a.seq > b.seq`


### Def-local-min

一个 local-min 的instance `x`指: x所有的dependency的ord都大于x.ord, 且每个dependency都有一条路径回到`x`, 且这些路径上的每个instance `y`的ord都大于`x.ord`.

显然一个没有出向边的节点也是一个trivial的local-min.

`x`,以及每个回到`x`的路径, 称为`L(x)`.

显然2个local-min的instance的`L()`图没有公共节点. 否则其中一个会在`L()`中看到更小ord的instance.


### Def-DAG-lm

根据algo walk, 在图G中从任一节点`x`出发最终都能找到一个没有出向边的节点,
记为 `lm(G, x)`.

从x出发直到`lm(G, x)`删掉所有出向边后, 经过的所有节点的序列: `P(G, x)`.

TODO `lm(G, x₁) != lm(G, x₂)` implies `x₁ != x₂`

一个图中, 从所有节点开始遍历, 找出所有local-min的节点的集合`V0`:
`V0 = {lm(G, x) | x ∈ G}`.

TODO `P(G, x)` 不经过V0中其他节点. 如果经过v₀ᵢ, 则v₀ᵢ ∉ V0.

∴ V0中的节点没有依赖关系, i.e., 它们可以以任意顺序被发现.

从G中去掉V0, 得到G1, 再次找到所有的local-min节点:
`V1 = {lm(G\V0, x) | x ∈ G\V0}`.

根据algo walk, 从V1中的一个节点`y ∈ V1`一定走到V0中的一个节点`x₀ ∈ V0`.
定义这个y和x之间有一个依赖关系`y ↦ x₀`: i.e., x₀被删除之前y不会被发现为local-min

从G中删除x₀, y通过walk-algo走到另一个节点x₁, 继续删除x₁,
直到`lm(G'\{x₀, x₁, x₂..}, y) = y`

则我们得到一组y到V0节点的依赖关系: `y ↦ x₁, y ↦ x₂, ...`

重复这个步骤直到所有节点都被删除, 则, 所有节点组成一个`↦`关系组成的DAG:
DAG-lm.


### 执行顺序遵从DAG-lm 的 topology 顺序


对一个G, 它对应的DAG-lm是确定的, 因此exec也是确定.
所有replica都按照DAG-lm的topology顺序执行, 执行顺序也是确定的.



一个图中一定有local-min节点存在: SCC中最小ord的instance是其中一个local-min,
没有出向边的instance是一个local-min.

linearizability: 如果`y → x and y.ord > x.ord`, `y`一定在`x`之后被发现.
`y → x and y.ord > x.ord ⇒ y ↦ x`.

consistency: if `y → x`, 则在DAG-lm中一定有一条路径`y ..→ x`.
因为删掉`y → x`只当x, y之间有环时(x, y之间有2条路径)
因此x, y在每个replica上执行顺序一样

### DAG-lm 的例子

下图中数字代表seq

```

G:

2 -→ 3 -→ 4 -→ 1
 ↖        | ↖ /
   `------'

DAG-edge:

// 如果一个节点x的所有边的所有依赖的边中, 包括另一个节点y的所有边, 那么
x ↦ y

34
↓
42
↓
23 → 41
     ↓
     14

DAG-lm:

.------------.
|             ↘
2    3 -→ 4 -→ 1
 ↖        |
   `------'

```


finite:
一个环的形成要求: `a₀ → a₁ → a₂ ... aᵢ → a₀`
对`aᵢ, aᵢ₊₁`, 假设产生`aᵢ ←  aᵢ₊₁`的概率是p, `aᵢ.seq <  aᵢ₊₁.seq` 的概率是0.5
则`aᵢ → aᵢ₊₁`或`aᵢ.seq > aᵢ₊₁.seq`的概率是k=1-0.5p
形成一个长度为n的环的几率是kⁿ=(1-0.5p)ⁿ
平均换的长度为`1 k + 2 k² + ...` = `k/(1-k)²`
假设p=0.5, 平均环长度是12.


<!-- vim: iskeyword+=-
-->
