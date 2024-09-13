package main

import (
	"fmt"
)

// generateAndPropagate calculates the generate and propagate signals
func generateAndPropagate(a, b bool) (bool, bool) {
	return a && b, a != b
}

// groupGenerate calculates the group generate signal
func groupGenerate(g1, p1, g0 bool) bool {
	return g1 || (p1 && g0)
}

// BrentKungAdder performs 4-bit addition using the Brent-Kung method
func BrentKungAdder(a []bool, b []bool, carryIn bool) (sum uint8, carryOut bool) {
	if len(a) != 4 || len(b) != 4 {
		panic("Input slices must be 4 bits long")
	}

	// Step 1: Generate and propagate
	g := make([]bool, 4)
	p := make([]bool, 4)
	for i := 0; i < 4; i++ {
		g[i], p[i] = generateAndPropagate(a[i], b[i])
	}

	// Step 2: Prefix computation
	g10 := groupGenerate(g[1], p[1], g[0])
	g20 := groupGenerate(g[2], p[2], g10)
	g30 := groupGenerate(g[3], p[3], g20)

	// Step 3: Calculate carries
	c := make([]bool, 5)
	c[0] = carryIn
	c[1] = g[0] || (p[0] && c[0])
	c[2] = g10 || (p[0] && p[1] && c[0])
	c[3] = g20 || (p[0] && p[1] && p[2] && c[0])
	c[4] = g30 || (p[0] && p[1] && p[2] && p[3] && c[0])

	// Step 4: Calculate sum
	for i := 0; i < 4; i++ {
		if p[i] != c[i] {
			sum |= 1 << i
		}
	}

	return sum, c[4]
}

// intToBoolSlice converts an integer to a 4-bit boolean slice
func intToBoolSlice(n int) []bool {
	return []bool{
		n&1 != 0,
		n&2 != 0,
		n&4 != 0,
		n&8 != 0,
	}
}

// boolSliceToInt converts a boolean slice to an integer
func boolSliceToInt(bs []bool) int {
	result := 0
	for i, b := range bs {
		if b {
			result |= 1 << i
		}
	}
	return result
}

func main() {
	for a := 0; a < 16; a++ {
		for b := 0; b < 16; b++ {
			aSlice := intToBoolSlice(a)
			bSlice := intToBoolSlice(b)

			sum, carryOut := BrentKungAdder(aSlice, bSlice, false)

			expectedSum := (a + b) & 0xF
			expectedCarry := a+b > 15

			if int(sum) != expectedSum || carryOut != expectedCarry {
				fmt.Printf("Error: %d + %d = %d (carry: %v), expected %d (carry: %v)\n",
					a, b, sum, carryOut, expectedSum, expectedCarry)
			} else {
				fmt.Printf("Correct: %d + %d = %d (carry: %v)\n",
					a, b, sum, carryOut)
			}
		}
	}
}
