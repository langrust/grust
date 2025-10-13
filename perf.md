# Performance tracking

On `examples/two_speed_limiters/`.

- [Performance tracking](#performance-tracking)
  - [`e328e366`](#e328e366)
  - [`7e54a883`](#7e54a883)


## `e328e366`

```text
Stats:
|               parsing (ir0 → ir1) |       0.1621875 |
|               type-checking (ir1) |        0.262208 |
| dependency graph generation (ir1) |        0.660334 |
|          causality analysis (ir1) |        0.146208 |
|               normalization (ir1) |       0.1495792 |
|                         ir1 → ir2 |      0.64158667 | better |
|       codegen (ir2 → rust tokens) |     2.658927458 |

Stats:
|               parsing (ir0 → ir1) |        0.928291 |
|               type-checking (ir1) |        0.229916 |
| dependency graph generation (ir1) |        0.624709 |
|          causality analysis (ir1) |        0.130125 |
|               normalization (ir1) |       0.1416291 |
|                         ir1 → ir2 |      0.63314250 | better |
|       codegen (ir2 → rust tokens) |     2.648718083 |
```


## `7e54a883`

Already optimized a few things.

```text
Stats:
|               parsing (ir0 → ir1) |       0.1610125 |
|               type-checking (ir1) |        0.246875 |
| dependency graph generation (ir1) |        0.635542 |
|          causality analysis (ir1) |        0.128250 |
|               normalization (ir1) |       0.1355791 |
|                         ir1 → ir2 |      0.88985125 |
|       codegen (ir2 → rust tokens) |     2.602312042 |

Stats:
|               parsing (ir0 → ir1) |        0.930958 |
|               type-checking (ir1) |        0.254667 |
| dependency graph generation (ir1) |        0.639167 |
|          causality analysis (ir1) |        0.128666 |
|               normalization (ir1) |       0.1434667 |
|                         ir1 → ir2 |      0.63738584 |
|       codegen (ir2 → rust tokens) |     2.647539000 |
```
