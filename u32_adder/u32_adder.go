package u32adder

import "github.com/consensys/gnark/frontend"

func BrentKungAdder32Bits(api frontend.API, a, b []frontend.Variable, carryIn frontend.Variable) ([]frontend.Variable, frontend.Variable) {
	if len(a) != 32 || len(b) != 32 {
		panic("Input slices must be 32 bits long")
	}

	carry := carryIn

	sum := make([]frontend.Variable, 32)

	for i := 0; i < 8; i++ {
		start := i * 4
		end := start + 4

		groupSum, groupCarry := BrentKungAdder4Bits(
			api,
			a[start:end],
			b[start:end],
			carry,
		)

		copy(sum[start:end], groupSum)
		carry = groupCarry

	}

	return sum, carry
}
