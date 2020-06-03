## Leveled quorum

Hierarchical quorum

Treat the cluster `a, b, c, d..x` as a 3-node cluster:

```
a,  B=(b, c...), x
```

A quorum is:
⅔ of a, B, x
and a major of `b, c...`

e.g.: for `a, b, c, d, e`
a quorum could be:
`a, (b, c)` 
`a, (c, d)`
`(b, d), e`
`a, e`

Higher level quorum is formed the same way.

## Build leveled quorum by latency

The graph of nodes has edges with distance set to be the latency between two
nodes.

Then add nodes close to each other into a group.
Then the graph is splitted into a leveled quorum.

## Recover only 0 or 1

If the value to recover can only be 0 or 1:

When recover:

- collect fast-status from a majority.
- decide 0 or 1. // prefer 0.
- send accept 0 with rnd=0, or prepare for 1 with rnd=1, then accept.
- commit.


## Fast commit

To fast commit the relation of `a > x`:
`v = 1 if a > x` a has seen x;
`v = 0 if a < x` a has not seen x.


```
a | b c d | x
```

Fast safe: if a value `v` constituted a leveled quorum
in `a, (b, c, d), x`.

The FP-condition: require x is committed and x not seen a
is another way to ensure majority in `(b, c, d)`


To be specific, if a see x on leader `L(x)`, x must be committed(or accepted?
TODO ) on `L(x)` and
`a > x`.

TODO: accepted or committed both impede `a < x` to be committed on other
replicas.




## TODO commit(determine the order) when read, not write





