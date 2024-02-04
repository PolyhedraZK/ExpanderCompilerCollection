# 编译流程

整体编译流程在 api.go 里面，解释如下：

1. gnark 的源码首先经过 builder，生成了 circuit ir。
2. ir 可以进行一些优化。
3. ir 经过 layering，生成了 layered circuit。
4. layered circuit 可以再进行一些优化。

## Builder

builder 和 gnark 内置的 builder 基本一致，不过 LinearExpression 改成了允许二次项的 Expression。

builder 会记录整体的树结构，然后生成 ir。

## IR

ir 是电路在编译过程中的中间表示，每个电路由许多 Instruction 组成，每个 Instruction 可以是：

1. InternalVariable，表示一个中间变量。
2. Hint，和 gnark 的 hint 相同。
3. SubCircuit，表示一次子电路的调用。

## Layering

layering 会将 ir 转换成分层电路。具体来说，会执行以下步骤（见 `layering/compile.go:Compile`）：

1. 对所有电路形成的（子电路调用关系图）DAG 拓扑排序。
2. 对于每个电路，计算其中每个变量需要在哪些层出现。含有子电路的情况，会默认子电路的输入/输出节点均在同一层。此时我们已经知道，每一层会出现哪些变量。
3. 对于每个电路，分配每一层变量的排布。目前的策略大致是，递归地将子电路的该层进行分层，然后按照大小倒序排序，再连起来。接下来把剩余变量放在空余的位置上。最后 pad 到 2^n。子电路的输入/输出层会有一些额外的处理，使得他们能尽量放在一起。
4. 对于每个电路，对于所有相邻的层，把这两层之间的连线编译成 `layered/Circuit`。这也是一个递归的过程，大致就是把子电路用到的变量拿出来，取最小能放下它的 2^n，然后递归求解。

还有一些细节：
1. Assertion 会被按层划分，归集到一个变量上。每层的这个变量也会累加起来，直到输出层。
2. 如果子电路用到 Hint，这些 Hint 需要从输入层一路 relay 过来。这些 relay 也会被编译成一个单独的子电路。

此过程中会删去大多数不必要的变量，并删除不必要的子电路调用。

## Layered

见 [Layered Circuit Format](./layered_circuit_format.md)。

电路的输出层里面，第 0 个是 Random Combined Assertions，后面的依次是 Public Input。

目前实现的优化有：展开出现次数或变量个数较少的 sub circuit。