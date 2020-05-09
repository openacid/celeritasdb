> `instance` and `vertex` are equivalent concepts in this doc.
>
> DAG: directed acyclic graph.

# Algorithm

Let `x.seq=(x.seq, x.leader_id, x.index)`, thus any two instances have different `seq`.

Select one non-executed instance `x`.

Depth-first walk along the dependency graph, choose next instance with the **smallest** `seq`.  Mark accessed instances as **accessed**.

-   If it walks to an instance without outgoing edges, execute it and remove it, then backtrack to previous instance and go on.

-   If it walks to an **accessed** instance, then we know that a cycle is found:
    Walk through the cycle again, find the instance `y` that has the smallest `seq` in the cycle.
    Remove the edge `y -> z` from the cycle.

    Then continue walking from `y`.


## Example of execution

One of the properties of this algorithm is
it does not need to walk through the entire SCC
before executing an instance.

As shown blow:

```
          .-> 4
          |
1 -> 6 -> 3 -> 5 -> 2 -> 8
     ^              |
     `--------------'
```

Starting from `1`, it walks along **min-edge**s to `4`, And finds out that `4` is the first instance to execute.
The effective path it walked is: `[1, 6, 3, 4]`, The cycle `6, 3, 5, 2` has not yet been discovered.

`4` is executed and removed.
It backtrack to `3` and continue the walking:

```
1 -> 6 -> (3) -> 5 -> 2 -> 8
     ^                |
     `----------------'
```

Then it found a cycle `6, 3, 5, 2`, After removing the edge of the smallest vertex: `(2, 6)`, the effective path now is `[1, 6, 3, 4, 3, 5, 2]`(cycles do not count).

Continue to walk from `2`
until it finally finds `8` as the second instance to be executed.

And then it backtracks to `2, 5, 3, 6, 1`, and executes them in this order.


## Optionally replaceable concept

This algorithm also applies to scenarios where `seq` is not used, as long as the following conditions are met:

-   It can distinguish which edges can be deleted to ensure that
    linearizability is not broken.

-   There must be a way to decide which edges to delete on different replicas
    (for example, select the smallest instance-id).

    Moreover, the way needs to make the walking prefer earlier proposed instances.

As long as the above conditions are met, this algorithm applies.
In this document it uses `seq`.


# Properties

Besides the linearizability, consistency, and avoiding the livelock naturally, it also provides some other guarantees:

-   Execution is safe to stop at any time.
    Instances in An SCC does not need to be executed in a transaction.

-   Allow multiple threads to execute simultaneously.
    The only requirement is that modifications to an instance(removing an edge or
    removing an instance) must be atomic.

-   Time complexity can be optimized to about `O(2m)`, where m is the number of instances.


# Definition

-   `G = (vertices, edges)`, an edge has a weight.

-   `|G|`:
    the number of vertices in a set or a graph `G`.


-   With given instances `x, y`, if x depends on y, there is an edge: `(x, y)`, the weight of the edge is: `(x.seq, y.seq)`.

-   **min-edge**:
    is the edge with the smallest `weight`.

    - for a vertex, it is the smallest of all outgoing edges:
      `MinEdge(x) = MinEdge(x.edges) = e: e ∈ x.edges ∧ weight(e) is smallest`

    - for a graph, it is the smallest of all edges in the graph:
      `MinEdge(G) = MinEdge(G.edges) = e: e ∈ G.edges ∧ weight(e) is smallest`

-   **min-cycle** `Cᵢ(x, G)`:
    is the i-th cycle found by walking along **min-edge**s from vertex `x`.

    `C(G)`: is the set of all **unrelated** cycles in `G`:
    `C(G) = {C₁(x, G) | x ∈ G.vertices}`

    Refer to **Lemma-cycle-DAG** for a definition of **related**.

-   key-vertex: `Kᵢ(x, G)`:
    is the i-th **key-vertex** found by walking along **min-edge**s from vertex `x`.

    key-vertex: is the min-edge of cycle `Cᵢ(x, G)`:
    `Kᵢ(x, G) = u: (u, v) = MinEdge(Cᵢ(x, G))`

-   `Gᵢ`: the graph obtained by removing all cycle `C(Gᵢ₋₁)` and all vertices
    without outgoing edge from `Gᵢ₋₁`.
    And `G₀ = G`.

-   `N₀(G)`: non-key-vertex: the set of all non key-vertex.

-   Walking path `P(x, G)`:
    the sequence of vertices obtained by walking along **min-edge** from `x`.

-   Effective path `Pe(x, G)`:
    the result of removing all cycles from `P(x, G)`.

For an example of the concepts above, the vertices in it are named with `seq`:

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

If `u ~ v` and `u` is initiated after `v` is committed, then `u.seq > v.seq`.

∴ If `u.seq <= v.seq`, `u` does not have to execute after `v` thus removing
a **min-edge** does not affect linearizability.

QED.


## Lemma-cycle-DAG


Walking from any vertex, the order of the discovering cycles satisfies a
topological order `DAG(C)`.

First find this DAG:

1.  From every vertex, before removing any min-edge, the walking found a set of cycle `C(G₀)`.
    Easy to see that cycles in `C(G₀)` are unrelated to each other:
    E.g., walking through two cycles `cᵢ, cⱼ ∈ C(G₀)` in any order
    always removes the same **min-edge**s.

2.  After removing min-edge and from `C(G₀)`
    and the vertices without outgoing edges
    (walking to these vertices is a simple backtrace, which does not affect cycle discovery).

    In the resulting graph `G₁` continue walking as described in step 1, find another cycle set `C(G₁)`.
    Cycles in `C(G₁)` are not related to each other, but may depends on cycles in `C(G₀)`.

    `cᵢ -> cⱼ`, depend-on for cycle is defined as:
    cᵢ depends on cⱼ if cⱼ can not be walked through if `MinEdge(cⱼ)` is not
    removed.

    Easy to see the cycles in `C(Gᵢ)` does not depend on each other.

Repeat until all vertices in `G` are removed.

Then we find a `DAG` of the dependency relationship between cycles.

The order of cycle discovery is in the topological order defined by `DAG(C)` :
A walking, since walked to a vertex on a cycle `c`, have to remove all the **min-edge**s of the cycles that `c` depends on, before discovering `c`.

∴ The order of cycle discovery satisfies the topological order of `DAG(C)` no
matter from what vertex the walking starts from.

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

The effective path through the instance dependency graph `G`, is equivalent to walking through `DAG(G)` which is obtained by removing all min-edge from `G`.

1.  By **Lemma-cycle-DAG**,

    ∴ The set of the smallest edges removed is the same for any walking.

2.  After removing all **min-edge**s, `G` becomes a DAG: `DAG(G)`,

3. By 1 and 2, `DAG(G)`s for all walking are the same.

4.  With a given **min-edge** `e = MinEdge(Cᵢ(x, G))`:
    A walking goes back to the source vertex of `e` after walking through the
    containing cycle `Cᵢ(x, G)`.

    ∴ The effective path of a walking is the same whether `e` exists or not.

By 3 and 4:
the effective path of a walking in `G` is equivalent to a walking in `DAG(G)`.

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

Two concurrent walks do not affect each other.

By **Lemma-dep-DAG**，
The walking in an SCC `G` is the same as the walking in `DAG(G)`.
And since a walking only select min-edge.

As long as 2 walking process atomically delete vertex/edge, they execute instance in the same topological order of `DAG(G)`.

QED.


## Proof: incremental execution

An execution process can be safely restarted at any time.
It does not need to execute all instances in an SCC once all together.

A restarted execution process is equivalent to:
a running execution process and another process paused for ever.

QED.


## Proof: execution consistency

Two dependent instances on any replica will be executed in the same order.

Because an instance only records the largest `dep` of a leader, Thus when executing, in addition to a `dep` of an instance, it also need to find out all the indirect dependencies that have the same leader
of `dep`.

1.  Two interfering vertices `x, y` has at least one path in `DAG(G)`:

    If instance x depends on instance y, there is an edge `x -> y` in `G`.

    ∵ The min-edge is removed only when there is a cycle through `x, y`.

    ∴ There is at least one path from `x` to `y` after removing the min-edge
    from the cycle.

2.  If there is a path between `x, y` in `DAG(G)`, their order is determined by this path.

3.  By **Lemma-dep-DAG**, the effective path of a walking in `G` is equivalent to the walking in `DAG(G)`.

∴ By 1, 2, 3:
two interfering instances have the same execution order on every replica.
The order is the same as topological order of `x` and `y` in `DAG(G)`.

QED.


## Lemma-persistent-edge

With a given edge `(u, v)`, if `u.seq > v.seq`, this edge will not be removed until an vertex is executed and removed.

For two adjacent edges `(u, v)` and `(v, w)`:
we have that
`weight(u, v) = (u.seq, v.seq) > weight(v, w) = (v.seq, w.seq)`.

∴ In any cycle that contains edge `(u, v)`, the weight of `(u, v)` will never be the smallest.

∴ `(u, v)` will not be removed as a **min-edge**.
Since there is no vertex is executed and removed, there is also no other way
this edge will be removed.

QED.


## Lemma-key-vertex-mapping

With a walking from vertex `x₀`, the i-th key-vertex found, `xᵢ = Kᵢ(x₀, G)`
and `G'` that is obtained by removing several **min-edge**:
before any vertex is executed and removed, we have:
there is at least one edge `(n₀, xᵢ)` in `G'`, where `n₀ ∈ N₀(G)`.
And `|N₀| >= 0.5 * |G|`.

1.  Case 1: `Cᵢ(x₀, G)` does not contain any key vertex: `Kⱼ(x₀, G), j<i`:

    Because a vertex does not depend on itself, a cycle has at least 2 vertices in it.

    ∴ `Cᵢ(x₀, G)` has at least one `n₀ ∈ N₀(G)`.

    ∴ With `n₀.seq > xᵢ.seq`, by **Lemma-persistent-edge** we see that
    there is an edge `(n₀, xᵢ)` in `G'`.

2.  Case 2: `Cᵢ(x₀, G)` contains some key vertex `Kⱼ(x₀, G), j<i`:

    We will see that along `Cᵢ(x, G)`, the vertex before `xᵢ` must be a vertex in `N₀(G)`, e.g.  `(n₀, xᵢ): n ∈ N₀(G)`.

    Assumes:
    - there is an edge `(xⱼ, xᵢ)` in cycle `Cᵢ(x₀, G)`, where `xⱼ = Kⱼ(x₀, G), j<i`,
    - and edge `(xⱼ, a)` is the removed min-edge in cycle `Cⱼ(x₀, G)`.

    Then we have:

    -  `xᵢ.seq < xⱼ.seq` : because `xᵢ` has smallest `seq` in cycle `Cᵢ(x₀, G)`.
    -  `xⱼ.seq < a.seq`  : because `xⱼ` has smallest `seq` in cycle `Cⱼ(x₀, G)`.
    -  `a.seq < xᵢ.seq`  : because `(xⱼ, a)` has been chosen before `(xⱼ, xᵢ)`.

    ```
    .-- .. --.
    |        v
    '-- a <- xⱼ -> xᵢ --.
             ^          |
             `-- .. ----'
    ```

    It is impossible to form a cycle with greater-than relation.
    Thus our assumption about edge `(xⱼ, xᵢ)` does not hold.

    ∴ There must be an edge `(n₀, xᵢ)` in `Cᵢ(x₀, G)` where `n₀ ∈ N₀(G)`,

    ∴ With `n₀.seq > xᵢ.seq`, by **Lemma-persistent-edge** we see that
    there is an edge `(n₀, xᵢ)` in `G'`.

∴ From 1 and 2, for `i>0`, every `Kᵢ(x₀, G)` has a vertex in `N₀(G)` before it in `G'`.

∵ In `G'` every `n₀ ∈ N₀(G)` has only one outgoing edge

∴ There is a one-to-one map from key-vertex and non-key-vertex.

∴ `|N₀(G)| >= 0.5 * |G|`

QED.



## Proof: execution in finite number of steps

In an infinite SCC, starting from an instance with a finite `seq`, this algorithm can always find the first instance to execute in a finite number of steps.

**Define**:
`Seqs(x) = {y.seq | x -> y}`

By **Lemma-key-vertex-mapping**, in the worst case, when going to the next vertex, it has the same chance of choosing either a key-vertex or a non-key-vertex.

Assuming the value of `Seqs(x)` is uniformly distributed around `x.seq`.
The `seq` of next vertex has a 50% chance of being `min(Seqs(x))`(non-key-vertex)
and 50% chance of being `max(Seqs(x))`(key-vertex, the worst case is the max
edge is chosen).

The change in `seq` can be simplified to a process of [Random-walk].

A random walk from the origin access any point within a finite distance in a
finite number of steps(Refer to next chapter for a proof).

∴ No matter from what vertex it starts the algorithm, It only takes a finite number of steps to find a vertex to execute.

However, having 50% chance of choosing `max(Seqs(x))` is just the worst case.

A realistic scenario would be:
The walking quickly converges to lower `seq` region.
Then it walks around the region until finding a vertex to execute.

QED.

### Random walk guarantees that it only takes a finite number of steps to get to a point within a finite distance

The distribution of the position after k steps random walk, is derived from Pascal's triangle:

```

     k     −5   −4  −3   −2   −1   0   1    2   3    4   5
  P[S₀=k]                          1
 2P[S₁=k]                     1        1
2²P[S₂=k]                 1        2        1
2³P[S₃=k]            1        3        3        1
2⁴P[S₄=k]        1        4        6        4        1
2⁵P[S₅=k]    1       5        10       10       5        1
```

After several steps, the probability of reaching the point `2n` is:

- after 2n steps: `C(2n, 2n)/2²ⁿ`
- after 2n+2 steps: `C(2n+2, 2n+1)/2²ⁿ⁺²`
- after 2n+4 steps: `C(2n+4, 2n+2)/2²ⁿ⁺⁴`
- ...

`C(m, n) = m!/n!/(m-n)!` is the combination number of choosing n items from m items.

∴ the expected total number of times reaching the point `2n` is:

```
       k
E(s) = ∑  C(2n+2i, 2n+i)/2²ⁿ⁺²ⁱ
       i=0

```

And approximated by [Stirling-approximation]:


```
       k       √(2n+2i)
E(s) = ∑   -----------------  (1-n/(2n+i))²ⁿ⁺²ⁱ (1+n/i)ⁱ
       i=0  √(2π) √(2n+i) √i
```

When `i` becomes large, by `(1+1/i)ⁱ = e`, it is approximated to

```
         1   k      1
E(s) = ----- ∑   -------
       √π eⁿ i=0 √(2n+i)

```

Easy to see that when `i` becomes big enough, `E(s)` does not converge.
Thus we can always find a finite `k` that satisfies:
`E(s) > n`, for any finite number `n`.

[Random-walk]: https://en.wikipedia.org/wiki/Random_walk
[Stirling-approximation]: https://en.wikipedia.org/wiki/Stirling's_approximation
