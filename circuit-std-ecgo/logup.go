package logup

import (
	"math"
	"math/big"
	"math/rand"

	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo"
)

var (
	Table           [][]frontend.Variable
	QueryID         []frontend.Variable
	QueryResult     [][]frontend.Variable
	LookupTableBits int
)

func init() {
	Table = make([][]frontend.Variable, 0)
	QueryID = make([]frontend.Variable, 0)
	QueryResult = make([][]frontend.Variable, 0)
	Hints := []solver.Hint{rangeProofHint, QueryCountHintFn, QueryCountBaseKeysHintFn}
	solver.RegisterHint(Hints...)
}
func Reset() {
	LookupTableBits = 0
	Table = make([][]frontend.Variable, 0)
	QueryID = make([]frontend.Variable, 0)
	QueryResult = make([][]frontend.Variable, 0)
}

// this interface write to a single (default) rangeProof table
func NewRangeProof(bits int) {
	LookupTableBits = bits
	rangeLogupTableSize := 1 << bits
	for i := 0; i < rangeLogupTableSize; i++ {
		Table = append(Table, []frontend.Variable{i})
	}
}

// this interface write to a single (default) table, the table size must be a power of 2
func NewTable(key []frontend.Variable, value [][]frontend.Variable) {

	if len(key) != len(value) {
		panic("key and value should have the same length")
	}
	if !IsPowerOf2(len(key)) {
		panic("the table size must be a power of 2")
	}
	if len(Table) != 0 {
		panic("table should be empty")
	}
	//append key as the first column, and value as the rest columns
	for i := 0; i < len(key); i++ {
		entry := append([]frontend.Variable{key[i]}, value[i]...)
		Table = append(Table, entry)
	}
}

// this interface write a query to the default table, Table
// a query is a pair of key and values
func Query(key frontend.Variable, value []frontend.Variable) {
	if len(value) != len(Table[0])-1 {
		panic("value length should be equal to the table column size")
	}
	QueryID = append(QueryID, key)
	//combine key and value as a query result
	queryResult := append([]frontend.Variable{key}, value...)
	QueryResult = append(QueryResult, queryResult)
}

func BatchQuery(keys []frontend.Variable, values [][]frontend.Variable) {
	if len(keys) != len(values) || len(keys) == 0 {
		panic("keys and values should have the same length and should not be empty")
	}
	if len(values[0]) != len(Table[0])-1 {
		panic("value length should be equal to the table column size")
	}
	for i := 0; i < len(keys); i++ {
		QueryID = append(QueryID, keys[i])
		//combine key and value as a query result
		queryResult := append([]frontend.Variable{keys[i]}, values[i]...)
		QueryResult = append(QueryResult, queryResult)
	}
}

// this interface write a query to the default table, Table
// For a range query that checks if a key is in a table while ignoring the value (RangeProof)
func QueryRange(key frontend.Variable) {
	if len(Table[0]) != 1 {
		panic("table should have only one column")
	}
	QueryID = append(QueryID, key)
	QueryResult = append(QueryResult, []frontend.Variable{key})
}

func RangeProof(api frontend.API, a frontend.Variable, n int) {
	//add a shift value
	if n%LookupTableBits != 0 {
		rem := n % LookupTableBits
		shift := LookupTableBits - rem
		constant := (1 << shift) - 1
		mulFactor := big.NewInt(1)
		mulFactor.Lsh(mulFactor, uint(n))
		a = api.Add(a, api.Mul(constant, mulFactor))
		n = n + shift
	}
	hintInput := make([]frontend.Variable, 2)
	hintInput[0] = n
	hintInput[1] = a
	witnesses, err := api.Compiler().NewHint(rangeProofHint, n/LookupTableBits, hintInput...)
	sum := witnesses[0]
	for i := 1; i < len(witnesses); i++ {
		constant := big.NewInt(1)
		constant.Lsh(constant, uint(LookupTableBits*i))
		sum = api.Add(sum, api.Mul(witnesses[i], constant))
	}
	api.AssertIsEqual(sum, a)
	if err != nil {
		panic(err)
	}
	for i := 0; i < n/LookupTableBits; i++ {
		QueryRange(witnesses[i])
	}
}

func FinalCheck(api frontend.Variable, column_combine_option ColumnCombineOptions) {
	if len(Table) == 0 || len(QueryID) == 0 {
		panic("empty table or empty query")
	} // Should we allow this?
	//if len(QueryID) != a power of 2, padding with query0
	if !IsPowerOf2(len(QueryID)) {
		nextPower2 := 1 << uint(math.Ceil(math.Log2(float64(len(QueryID)))))
		for i := len(QueryID); i < nextPower2; i++ {
			QueryID = append(QueryID, QueryID[0])
			QueryResult = append(QueryResult, QueryResult[0])
		}
	}
	ecgoApi := api.(ecgo.API)
	// The challenge used to complete polynomial identity check
	alpha := ecgoApi.GetRandomValue()

	column_combine_randomness := GetColumnRandomness(ecgoApi, uint(len(Table[0])), column_combine_option)

	// Table Polynomial
	table_single_column := CombineColumn(ecgoApi, Table, column_combine_randomness)
	// inputs :=
	inputs := []frontend.Variable{frontend.Variable(len(Table))}
	for i := 0; i < len(Table); i++ {
		inputs = append(inputs, Table[i][0])
	}
	inputs = append(inputs, QueryID...)
	query_count, _ := ecgoApi.NewHint(
		QueryCountBaseKeysHintFn,
		len(Table),
		inputs...,
	)

	table_poly := make([]RationalNumber, len(table_single_column))
	for i := 0; i < len(table_single_column); i++ {
		table_poly[i] = RationalNumber{
			Numerator:   query_count[i],
			Denominator: ecgoApi.Sub(alpha, table_single_column[i]),
		}
	}
	table_poly_at_alpha := SumRationalNumbers(ecgoApi, table_poly)
	// Query Polynomial
	query_single_column := CombineColumn(ecgoApi, QueryResult, column_combine_randomness)
	query_poly := make([]RationalNumber, len(query_single_column))
	for i := 0; i < len(query_single_column); i++ {
		query_poly[i] = RationalNumber{
			Numerator:   1,
			Denominator: ecgoApi.Sub(alpha, query_single_column[i]),
		}
	}
	query_poly_at_alpha := SumRationalNumbers(ecgoApi, query_poly)
	ecgoApi.AssertIsEqual(
		ecgoApi.Mul(table_poly_at_alpha.Numerator, query_poly_at_alpha.Denominator),
		ecgoApi.Mul(query_poly_at_alpha.Numerator, table_poly_at_alpha.Denominator),
	)
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

func (c *LogUpCircuit) Check(api ecgo.API, column_combine_option ColumnCombineOptions) error {
	if len(c.Table) == 0 || len(c.QueryID) == 0 {
		panic("empty table or empty query")
	} // Should we allow this?

	// The challenge used to complete polynomial identity check
	alpha := api.GetRandomValue()

	column_combine_randomness := GetColumnRandomness(api, uint(len(c.Table[0])), column_combine_option)

	// Table Polynomial
	table_single_column := CombineColumn(api, c.Table, column_combine_randomness)
	query_count, _ := api.NewHint(
		QueryCountHintFn,
		len(c.Table),
		c.QueryID...,
	)

	table_poly := make([]RationalNumber, len(table_single_column))
	for i := 0; i < len(table_single_column); i++ {
		table_poly[i] = RationalNumber{
			Numerator:   query_count[i],
			Denominator: api.Sub(alpha, table_single_column[i]),
		}
	}
	table_poly_at_alpha := SumRationalNumbers(api, table_poly)

	// Query Polynomial
	query_single_column := CombineColumn(api, c.QueryResult, column_combine_randomness)
	query_poly := make([]RationalNumber, len(query_single_column))
	for i := 0; i < len(query_single_column); i++ {
		query_poly[i] = RationalNumber{
			Numerator:   1,
			Denominator: api.Sub(alpha, query_single_column[i]),
		}
	}
	query_poly_at_alpha := SumRationalNumbers(api, query_poly)

	api.AssertIsEqual(
		api.Mul(table_poly_at_alpha.Numerator, query_poly_at_alpha.Denominator),
		api.Mul(query_poly_at_alpha.Numerator, table_poly_at_alpha.Denominator),
	)
	return nil
}

const ColumnCombineOption ColumnCombineOptions = FullRandom

type LogUpCircuit struct {
	Table       [][]frontend.Variable
	QueryID     []frontend.Variable
	QueryResult [][]frontend.Variable
}

// Define declares the circuit's constraints
func (c *LogUpCircuit) Define(api frontend.API) error {
	return c.Check(api.(ecgo.API), ColumnCombineOption)
}
