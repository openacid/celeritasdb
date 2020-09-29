
mean cycle size is finite:
一个环的形成要求: `a₀ → a₁ → a₂ ... aᵢ → a₀`
对`aᵢ, aᵢ₊₁`, 假设产生`aᵢ ←  aᵢ₊₁`的概率是p, `aᵢ.seq <  aᵢ₊₁.seq` 的概率是0.5
则`aᵢ → aᵢ₊₁`或`aᵢ.seq > aᵢ₊₁.seq`的概率是k=1-0.5p
形成一个长度为n的环的几率是kⁿ=(1-0.5p)ⁿ
平均换的长度为`1 k + 2 k² + ...` = `k/(1-k)²`
假设p=0.5, 平均环长度是`c = 12`.

finite proof 2

All walking from `V₀ᵢ` always walks to `e₀ᵢ` then walks along one of the outgoing
edges from `e₀ᵢ`.
Thus subsequent walking is just like walking among `{e₀ᵢ}`, except there are
multiple edges between two vertices, and a vertex has edges pointing to itself.

Assumes `e₀ᵢ` only has finite number(`k` at most) of outgoing edges.
When all of its edges are remove the total number of edges removed is finite:
first round: `n = c`
second: `n = n * c`
...
`n = cᵏ`


### Proof: finite steps

The number of steps this algo takes to find a vertex to remove from a SCC is finite.

Proof:

Edges in `Eⱼ(G)` splits the `G.vertices` into groups,
e.g., group `V₀ᵢ` is set of vertices that a walk from it will remove `e₀ᵢ`:
- `V₀ᵢ = {x ∈ G.vertices | e₀ᵢ ∈ rmSeq(G, x)}` for every `e₀ᵢ ∈ E₀(G)`.
- `V₁ᵢ = {x ∈ G.vertices | e₁ᵢ ∈ rmSeq(G, x)}` for every `e₁ᵢ ∈ E₁(G)`.
- ...

(1): the upper bound of `seq` of vertices that `x` depends-on,
i.e., `b = max({y.seq | ∃(x, y)})` is finite.
Because an instance will always be committed in finite time, by its leader or by a recovery.

(2): With a given upper bound of `seq`: `b`,
the number of vertices with a `seq` smaller than `b` is finite:
i.e., `Nv(b) = |{x | x.seq < b}|` is finite:
Because A replica always proposes new instance with monotonic incremental `seq`.


(3): From (2), the the number of groups that have a vertex with `seq: seq < b`,
i.e., `Ng(b) = |{g | ∃x ∈ g, x.seq < b}|` is also finite.

If it walks to a same group, it only removes edges from vertex set `V = {v |
v.seq <= x.seq}`.
After removed an edge `(x, y)`, 
because every walking path always goes to `x`, then the next min-cycle either
includes `x` or another vertex with smaller `seq` than `x`.


- After removing an edge, e.g., `e₀ᵢ = (x, y)`,
  if the subsequent walking go back to `V₀ᵢ`, the next edge to remove also sources
  from `x`, or source from a vertex in `V₀ᵢ` with smaller `seq`.

  ∴ (4): If the walking involves only one group, it keeps removing edges from
  a set of vertex: `{v | v.seq <= x.seq}`.
  From (2), the size of this set, i.e., `Nv(x.seq)` is finite.

  ∴ (5): Within one group, from (1) and (4), before any vertex is removed, the number of edges to remove is finite and is at most `Nv(x.seq)` times the max number of edges a vertex could have, where `x` is the source vertex of a removed edge in this group.

∴ After removing the first edge `(x, y)`,
If the walking does not union another group, From (3), for `Nv(x.seq) * C` steps, a vertex will be
removed.
If it unioned `Ng(x.seq)` group, it always walks back to the same group.
it takes at most `Nv(x.seq) * C` steps to remove a vertex from a
SCC.

TODO: need to prove that it takes finite steps to go from e₀ᵢ to e₀ⱼ
TODO: need to prove that the after unioned `Ng(x.seq)` groups, the chance going
to another group is low.

QED.

Example is shown in the following digraph:
A walking starts from `8` will go back to V₀₀ after removing the first edge `(1, 2)`.
A walking starts from `7` will union `V₀₁` and `V₀₂` after removing edge `(3, 4)`.

```
     .→ 8 ←--.       7 ←----------.
     |  ↓    |       ↓            |
     |  2    '-----→ 4      .→ 6  |
     |  ↕            ↕      |  ↕  |
     `- 1            3 -----'  5 -'
       ---          ---       ---
       V₀₀          V₀₁       V₀₂
       e₀₀=(1,2)    e₀₁=(3,4) e₀₂=(5,6)
       ---          -------------
       V₁₀          V₁₁ = V₀₁ ∪ V₀₂
       e₁₀=(1,8)    e₁₁=(3,6)
```
