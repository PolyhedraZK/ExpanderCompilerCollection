package u32adder

import (
	"github.com/consensys/gnark/frontend"
)

// Adder32Bits performs 32-bit addition using gnark frontend operations
func NaiveAdder32Bits(api frontend.API, a, b []frontend.Variable, cin frontend.Variable) ([]frontend.Variable, frontend.Variable) {
	if len(a) != 32 || len(b) != 32 {
		panic("Input slices must be 32 bits long")
	}

	sum := make([]frontend.Variable, 32)
	carry := cin

	add := func(x ...frontend.Variable) frontend.Variable {
		result := x[0]
		for i := 1; i < len(x); i++ {
			result = api.Add(result, x[i])
		}
		return result
	}

	mul := api.Mul

	// Pre-compute a[i] * b[i] for all bits
	amulb := make([]frontend.Variable, 32)
	for i := 0; i < 32; i++ {
		amulb[i] = mul(a[i], b[i])
	}

	// Pre-compute a[i] + b[i] for all bits
	aaddb := make([]frontend.Variable, 32)
	for i := 0; i < 32; i++ {
		aaddb[i] = add(a[i], b[i])
	}

	// Perform addition bit by bit
	for i := 0; i < 32; i++ {
		// Calculate sum for this bit
		sum[i] = add(aaddb[i], carry)

		// Calculate carry for the next bit
		if i < 31 {
			carry = add(amulb[i], mul(aaddb[i], carry))
		}
	}

	// The final carry becomes cout
	cout := api.Add(
		amulb[31],
		mul(aaddb[31], carry))

	return sum, cout
}
