package u32adder

import (
	"github.com/consensys/gnark/frontend"
)

// GnarkBrentKungAdder performs 4-bit addition using the Brent-Kung method
// with gnark frontend operations
func BrentKungAdder4Bits(api frontend.API, a, b []frontend.Variable, carryIn frontend.Variable) ([]frontend.Variable, frontend.Variable) {
	if len(a) != 4 || len(b) != 4 {
		panic("Input slices must be 4 bits long")
	}

	// Helper functions
	xor := func(x, y frontend.Variable) frontend.Variable {
		return api.Add(x, y)
	}

	and := func(x, y frontend.Variable) frontend.Variable {
		return api.Mul(x, y)
	}

	or := func(x, y frontend.Variable) frontend.Variable {
		return api.Add(api.Add(x, y), api.Mul(x, y))
	}

	// Step 1: Generate and propagate
	g := make([]frontend.Variable, 4)
	p := make([]frontend.Variable, 4)
	for i := 0; i < 4; i++ {
		g[i] = and(a[i], b[i])
		p[i] = xor(a[i], b[i])
	}

	// Step 2: Prefix computation
	g10 := or(g[1], and(p[1], g[0]))
	g20 := or(g[2], and(p[2], g10))
	g30 := or(g[3], and(p[3], g20))

	// Step 3: Calculate carries
	c := make([]frontend.Variable, 5)
	c[0] = carryIn
	c[1] = or(g[0], and(p[0], c[0]))
	c[2] = or(g10, and(and(p[0], p[1]), c[0]))
	c[3] = or(g20, and(and(and(p[0], p[1]), p[2]), c[0]))
	c[4] = or(g30, and(and(and(and(p[0], p[1]), p[2]), p[3]), c[0]))

	// Step 4: Calculate sum
	sum := make([]frontend.Variable, 4)
	for i := 0; i < 4; i++ {
		sum[i] = xor(p[i], c[i])
	}

	return sum, c[4]
}
