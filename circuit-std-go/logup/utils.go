package logup

import (
	"fmt"

	"github.com/consensys/gnark/frontend"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo"
)

type RationalNumber struct {
	Numerator   frontend.Variable
	Denominator frontend.Variable
}

func (r *RationalNumber) Add(api frontend.API, other *RationalNumber) RationalNumber {
	return RationalNumber{
		Numerator:   api.Add(api.Mul(r.Numerator, other.Denominator), api.Mul(other.Numerator, r.Denominator)),
		Denominator: api.Mul(r.Denominator, other.Denominator),
	}
}

// 0 is considered a power of 2 in this case
func IsPowerOf2(n int) bool {
	return n&(n-1) == 0
}

// Construct a binary summation tree to sum all the values
func SumRationalNumbers(api frontend.API, rs []RationalNumber) RationalNumber {
	n := len(rs)
	if n == 0 {
		return RationalNumber{Numerator: 0, Denominator: 1}
	}

	if !IsPowerOf2(n) {
		fmt.Println(n)
		panic("The length of rs should be a power of 2")
	}

	cur := rs
	next := make([]RationalNumber, 0)

	for n > 1 {
		n >>= 1
		for i := 0; i < n; i++ {
			next = append(next, cur[i*2].Add(api, &cur[i*2+1]))
		}
		cur = next
		next = next[:0]
	}

	if len(cur) != 1 {
		panic("Summation code may be wrong.")
	}

	return cur[0]
}

func SimpleMin(a uint, b uint) uint {
	if a < b {
		return a
	} else {
		return b
	}
}

func GetColumnRandomness(api ecgo.API, n_columns uint, column_combine_options ColumnCombineOptions) []frontend.Variable {
	var randomness = make([]frontend.Variable, n_columns)
	if column_combine_options == Poly {
		beta := api.GetRandomValue()
		randomness[0] = 1
		randomness[1] = beta

		// Hopefully this will generate fewer layers than sequential pows
		max_deg := uint(1)
		for max_deg < n_columns {
			for i := max_deg + 1; i <= SimpleMin(max_deg*2, n_columns-1); i++ {
				randomness[i] = api.Mul(randomness[max_deg], randomness[i-max_deg])
			}
			max_deg *= 2
		}

		// Debug Code:
		// for i := 1; i < n_columns; i++ {
		// 	api.AssertIsEqual(randomness[i], api.Mul(randomness[i - 1], beta))
		// }

	} else if column_combine_options == FullRandom {
		randomness[0] = 1
		for i := 1; i < int(n_columns); i++ {
			randomness[i] = api.GetRandomValue()
		}
	} else {
		panic("Unknown poly combine options")
	}
	return randomness
}

func CombineColumn(api ecgo.API, vec_2d [][]frontend.Variable, randomness []frontend.Variable) []frontend.Variable {
	n_rows := len(vec_2d)
	if n_rows == 0 {
		return make([]frontend.Variable, 0)
	}

	n_columns := len(vec_2d[0])

	// Do not introduce any randomness
	if n_columns == 1 {
		vec_combined := make([]frontend.Variable, n_rows)
		for i := 0; i < n_rows; i++ {
			vec_combined[i] = vec_2d[i][0]
		}
		return vec_combined
	}

	if !IsPowerOf2(n_columns) {
		panic("Consider support this")
	}

	vec_return := make([]frontend.Variable, 0)
	for i := 0; i < n_rows; i++ {
		var v_at_row_i frontend.Variable = 0
		for j := 0; j < n_columns; j++ {
			v_at_row_i = api.Add(v_at_row_i, api.Mul(randomness[j], vec_2d[i][j]))
		}
		vec_return = append(vec_return, v_at_row_i)
	}
	return vec_return
}
