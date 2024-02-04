# field

gnark 的 builder 中，调用了 R1CS（如 `gnark/constraint/bn254`）。而 R1CS 实现了 field。

由于 R1CS 实现的是 private 的，而我们又需要支持别的 field，所以有了这个独立出来的库。

目前支持的 field 有 `bn254` 和 `m31`，其中 `m31` 的模数是 $2^{31}-1$。