package u32adder

import "github.com/consensys/gnark/frontend"

func BrentKungAdder32Bits(api frontend.API, a, b []frontend.Variable, carryIn []frontend.Variable) ([]frontend.Variable, frontend.Variable) {
	if len(a) != 32 || len(b) != 32 {
		panic("Input slices must be 32 bits long")
	}

	if len(carryIn) != 8 {
		panic("CarryIn slice must be 8 bits long")
	}

	var carry frontend.Variable

	sum := make([]frontend.Variable, 32)

	for i := 0; i < 8; i++ {
		start := i * 4
		end := start + 4

		groupSum, groupCarry := BrentKungAdder4Bits(
			api,
			a[start:end],
			b[start:end],
			carryIn[i],
		)

		copy(sum[start:end], groupSum)
		if i != 7 {
			api.AssertIsEqual(carryIn[i+1], groupCarry)
		}
		if i == 7 {
			carry = groupCarry
		}

	}

	return sum, carry
}
