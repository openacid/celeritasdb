This is an incomplete alog because it didnot specifies how to impl this:
> 如果一个instance `z`的所有出向边最终都走回到`z`,
> From Def-topo-order, 执行`z`.
And this step is critical and no obvious way to impl.
One of the impl of this is the same as qpaxos-exec, i.e., by removing min-edge
found in a cycle.

# Execution

TODO: deps only records highest interfering, need to find the minimal. proof
that only directly deps  need to inherit the deps.
TODO: newer instance may have lower seq. need to consider this.

这个算法实现了有环depends-on graph中有限步的执行算法,
解决了livelock问题.

需要replication 为每个instance提供1个信息:
用于判断2个instance是否有After关系的偏序的值, 这里选择了deps.
epaxos的seq也可以.


## Terminology

- `a → b`: `a` depends-on `b`.
- `a ↦ b`: execution order: `a` should exec after `b`.

### Def-ord

`ord` 是用于为SCC中instance排序的字段: deps的每个`instance_id`之和, 以及`instance_id`:
`ord = (|deps|, instance_id)`.

∵ `a.deps ⊃ b.deps ⇒ |a.deps| > |b.deps|`

∴ 可以保证After关系一定被保证顺序执行.


### Def-linear-order

`linear-order` 是当看到整个图时确定的执行顺序:

假设整个图的结构是多个SCC连成的(一个不在环内的节点也是一个SCC):

```
S₁ → S₂ → S₃ ..
     ↘   ↗
       S₄ ..

// SCC 之间没有环, 否则会构成一个更大的SCC.
```

`linear-order` 定义为:
- 首先在 SCC 之间以依赖关系确定执行顺序.
- 再在 SCC 内部以ord确定执行顺序.

E.g.,

```
S₁ = {a₁, b₁ ...}
S₂ = {a₂, b₂, c₂ ...}
S₃ = {a₃, b₃ ...}
S₄ = {a₄, b₄ ...}
...

linear-order:

a₁ ↦ b₁ ↦ a₂ ↦ b₂ ↦ c₂ ↦ a₄ ↦ b₄ ↦ a₃ ↦ b₃ ...
-------   ------------   -------   -------
S1        S2             S4        S3
```

### Exchangeable instances

在一个`linear-order` `a ↦ b ↦ ... w ↦ x ↦ y ↦ z ...` 中, 考虑`x, y`:

∵ 2个instance的执行结果只通过`depends-on`关系互相影响, 

∴ 如果 `x` 到 `y` 的每条`depends-on` 的路径都只经过`x`之后执行的instance, 则`x, y` 可以调换执行顺序(显然如果`x→y`则一定不能调换顺序):

e.g., `x` 到 `y` 有2条路径:
x → w → ... y
x → b → ... y

那么一个compatible order是:

```
a ↦ b ↦ ... w ↦ x ↦ z ...
            |       ↥
             `↦ y --'
```

找到所有可以调换的`x, y`, 最终将 `linear-order` 转换成DAG 的拓扑顺序.
因此:


### Def-topo-order

如果 SCC中的 `x` 所有depends-on的instance 的`ord`都 大于`x.ord`,
`x`可以直接执行.

推论: 如果 `x.deps` 中每个instance `dᵢ` 都有一条到`x`的depends-on的路径 `dᵢ → .. → x`,
且 `dᵢ.ord > x.ord` 则x可以执行.


## Algorithm


从一个instance `x` 开始 DFS, 优先选择指向最小`ord`的边.

-   如果遇到一个没有出向边的instance `z`, 执行`z`.

-   如果走到一个instance `z` 且`z.ord < x.ord`, 从z开始重新执行exec算法.

-   如果一个instance `z`的所有出向边最终都走回到`z`,
    From Def-topo-order, 执行`z`.

执行一个`z`之后, 需要把`z.deps`合并到指向`z`的所有instance中,
以保证SCC去掉一个节点后还是SCC, 来保证SCC中剩下的instance顺序不变.

实现方法是当从`a`遍历时, 如果发现它其中一个依赖`z`是被标记执行过了,
那么将`z`从`a.deps`中去掉, 并将`z.deps`写入到`a.deps`, i.e.:
`a.deps = a.deps - {z} ∪ z.deps`.

实现中不会在DFS过程中增加更多额外IO, `deps`的合并可以跟下一个instance执行的结果batch写入.


## Cases, do not read

```
commands:

a: get x
b: set x = z
c: set x = y + z
d: set y = 5
e: set z = 3


a → b  ←  d
     ↘  ↗
       c


a → b
 ↖  ↓
    c → d


a → b
 ↖  ↓    ↙ \
    c → d → e
```


<!-- vim: iskeyword+=-
-->
