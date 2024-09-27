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
	p1g0 := and(p[1], g[0])
	p0p1 := and(p[0], p[1])
	p2p3 := and(p[2], p[3])

	g10 := or(g[1], p1g0)
	g20 := or(g[2], and(p[2], g10))
	g30 := or(g[3], and(p[3], g20))

	// Step 3: Calculate carries
	c := make([]frontend.Variable, 5)
	c[0] = carryIn
	c[1] = or(g[0], and(p[0], c[0]))
	c[2] = or(g10, and(p0p1, c[0]))
	c[3] = or(g20, and(p0p1, and(p[2], c[0])))
	c[4] = or(g30,
		and(
			and(
				p0p1,
				p2p3,
			),
			c[0]))

	// Step 4: Calculate sum
	sum := make([]frontend.Variable, 4)
	for i := 0; i < 4; i++ {
		sum[i] = xor(p[i], c[i])
	}

	return sum, c[4]
}

// NaiveAdder performs 4-bit addition using the Brent-Kung method
// with gnark frontend operations
func NaiveAdder4Bits(api frontend.API, a, b []frontend.Variable, carryIn frontend.Variable) ([]frontend.Variable, frontend.Variable) {
	if len(a) != 4 || len(b) != 4 {
		panic("Input slices must be 4 bits long")
	}

	// Helper functions
	add := func(x ...frontend.Variable) frontend.Variable {
		result := x[0]
		for i := 1; i < len(x); i++ {
			result = api.Add(result, x[i])
		}
		return result
	}

	mul := api.Mul

	// Pre-compute products
	ab := make([]frontend.Variable, 4)
	for i := 0; i < 4; i++ {
		ab[i] = mul(a[i], b[i])
	}

	// Calculate s0 and s0carry
	s0 := add(a[0], b[0], carryIn)
	s0carry := add(
		ab[0],
		mul(a[0], carryIn),
		mul(b[0], carryIn),
	)

	// Calculate s1 and s1carry
	s1 := add(a[1], b[1], s0carry)
	s1carry := add(
		ab[1],
		mul(a[1], s0carry),
		mul(b[1], s0carry),
	)

	// Calculate s2 and s2carry
	s2 := add(a[2], b[2], s1carry)
	s2carry := add(
		ab[2],
		mul(a[2], s1carry),
		mul(b[2], s1carry),
	)

	// Calculate s3 and s3carry (cout)
	s3 := add(a[3], b[3], s2carry)
	cout := add(
		ab[3],
		mul(a[3], s2carry),
		mul(b[3], s2carry),
	)

	return []frontend.Variable{s0, s1, s2, s3}, cout
}
