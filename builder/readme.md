# Builder

builder 是基于 gnark 内置的 builder 实现的，有以下几点改动：

- LinearExpression 改成了允许二次项的 Expression。
- Assert 系列函数，会先记录下来，留到最后再固化到 ir 里。
- 支持子电路。