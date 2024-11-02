package main

import (
	"math"
	"math/big"
	"math/rand"
	"os"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/test"
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

type LogUpCircuit struct {
	Table       [][]frontend.Variable
	QueryID     []frontend.Variable
	QueryResult [][]frontend.Variable
}

func NewRandomCircuit(
	n_table_rows uint,
	n_queries uint,
	n_columns uint,
	fill_values bool,
) *LogUpCircuit {
	c := &LogUpCircuit{}
	c.Table = make([][]frontend.Variable, n_table_rows)
	for i := 0; i < int(n_table_rows); i++ {
		c.Table[i] = make([]frontend.Variable, n_columns)
		if fill_values {
			for j := 0; j < int(n_columns); j++ {
				c.Table[i][j] = rand.Intn(math.MaxInt)
			}
		}
	}

	c.QueryID = make([]frontend.Variable, n_queries)
	c.QueryResult = make([][]frontend.Variable, n_queries)

	for i := 0; i < int(n_queries); i++ {
		c.QueryResult[i] = make([]frontend.Variable, n_columns)
		if fill_values {
			query_id := rand.Intn(int(n_table_rows))
			c.QueryID[i] = query_id
			c.QueryResult[i] = c.Table[query_id]
		}
	}

	return c
}

type ColumnCombineOptions int

const (
	Poly = iota
	FullRandom
)

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

// TODO: Do we need bits check for the count?
func QueryCountHintFn(field *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	for i := 0; i < len(outputs); i++ {
		outputs[i] = big.NewInt(0)
	}

	for i := 0; i < len(inputs); i++ {
		query_id := inputs[i].Int64()
		outputs[query_id].Add(outputs[query_id], big.NewInt(1))
	}
	return nil
}

func LogUpPolyValsAtAlpha(api ecgo.API, vec_1d []frontend.Variable, count []frontend.Variable, x frontend.Variable) RationalNumber {
	poly := make([]RationalNumber, len(vec_1d))
	for i := 0; i < len(vec_1d); i++ {
		poly[i] = RationalNumber{
			Numerator:   count[i],
			Denominator: api.Sub(x, vec_1d[i]),
		}
	}
	return SumRationalNumbers(api, poly)
}

func (c *LogUpCircuit) Check(api ecgo.API, column_combine_option ColumnCombineOptions) error {
	if len(c.Table) == 0 || len(c.QueryID) == 0 {
		panic("empty table or empty query")
	} // Should we allow this?

	// The challenge used to complete polynomial identity check
	alpha := api.GetRandomValue()
	// The randomness used to combine the columns
	column_combine_randomness := GetColumnRandomness(api, uint(len(c.Table[0])), column_combine_option)

	// Table Polynomial
	table_single_column := CombineColumn(api, c.Table, column_combine_randomness)
	query_count, hint_err := api.NewHint(
		QueryCountHintFn,
		len(c.Table),
		c.QueryID...,
	)
	if hint_err != nil {
		panic(hint_err.Error())
	}
	table_poly_at_alpha := LogUpPolyValsAtAlpha(api, table_single_column, query_count, alpha)

	// Query Polynomial
	query_single_column := CombineColumn(api, c.QueryResult, column_combine_randomness)
	dummy_count := make([]frontend.Variable, len(query_single_column))
	for i := 0; i < len(dummy_count); i++ {
		dummy_count[i] = 1
	}
	query_poly_at_alpha := LogUpPolyValsAtAlpha(api, query_single_column, dummy_count, alpha)

	api.AssertIsEqual(
		api.Mul(table_poly_at_alpha.Numerator, query_poly_at_alpha.Denominator),
		api.Mul(query_poly_at_alpha.Numerator, table_poly_at_alpha.Denominator),
	)
	return nil
}

const ColumnCombineOption ColumnCombineOptions = FullRandom

// Define declares the circuit's constraints
func (c *LogUpCircuit) Define(api frontend.API) error {
	return c.Check(api.(ecgo.API), ColumnCombineOption)
}

func main() {
	N_TABLE_ROWS := uint(128)
	N_QUERIES := uint(512)
	COLUMN_SIZE := uint(8)

	circuit, err := ecgo.Compile(ecc.BN254.ScalarField(), NewRandomCircuit(N_TABLE_ROWS, N_QUERIES, COLUMN_SIZE, false))
	if err != nil {
		panic(err.Error())
	}

	c := circuit.GetLayeredCircuit()
	os.WriteFile("circuit.txt", c.Serialize(), 0o644)

	assignment := NewRandomCircuit(N_TABLE_ROWS, N_QUERIES, COLUMN_SIZE, true)
	solver.RegisterHint(QueryCountHintFn)
	inputSolver := circuit.GetInputSolver()
	witness, err := inputSolver.SolveInput(assignment, 0)
	if err != nil {
		panic(err.Error())
	}

	if !test.CheckCircuit(c, witness) {
		panic("Circuit not satisfied")
	}

	// os.WriteFile("inputsolver.txt", inputSolver.Serialize(), 0o644)
	os.WriteFile("witness.txt", witness.Serialize(), 0o644)
}
