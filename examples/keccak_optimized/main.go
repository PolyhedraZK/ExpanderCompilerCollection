package main

import (
	"math/big"
	"os"

	gkr "github.com/Zklib/gkr-compiler"
	"github.com/Zklib/gkr-compiler/builder"
	"github.com/Zklib/gkr-compiler/field/m31"
	"github.com/Zklib/gkr-compiler/test"
	"github.com/consensys/gnark/frontend"
)

const NHashes = 7

var rcs [][]uint

func init() {
	var rc [24]*big.Int
	rc[0], _ = new(big.Int).SetString("0000000000000001", 16)
	rc[1], _ = new(big.Int).SetString("0000000000008082", 16)
	rc[2], _ = new(big.Int).SetString("800000000000808A", 16)
	rc[3], _ = new(big.Int).SetString("8000000080008000", 16)
	rc[4], _ = new(big.Int).SetString("000000000000808B", 16)
	rc[5], _ = new(big.Int).SetString("0000000080000001", 16)
	rc[6], _ = new(big.Int).SetString("8000000080008081", 16)
	rc[7], _ = new(big.Int).SetString("8000000000008009", 16)
	rc[8], _ = new(big.Int).SetString("000000000000008A", 16)
	rc[9], _ = new(big.Int).SetString("0000000000000088", 16)
	rc[10], _ = new(big.Int).SetString("0000000080008009", 16)
	rc[11], _ = new(big.Int).SetString("000000008000000A", 16)
	rc[12], _ = new(big.Int).SetString("000000008000808B", 16)
	rc[13], _ = new(big.Int).SetString("800000000000008B", 16)
	rc[14], _ = new(big.Int).SetString("8000000000008089", 16)
	rc[15], _ = new(big.Int).SetString("8000000000008003", 16)
	rc[16], _ = new(big.Int).SetString("8000000000008002", 16)
	rc[17], _ = new(big.Int).SetString("8000000000000080", 16)
	rc[18], _ = new(big.Int).SetString("000000000000800A", 16)
	rc[19], _ = new(big.Int).SetString("800000008000000A", 16)
	rc[20], _ = new(big.Int).SetString("8000000080008081", 16)
	rc[21], _ = new(big.Int).SetString("8000000000008080", 16)
	rc[22], _ = new(big.Int).SetString("0000000080000001", 16)
	rc[23], _ = new(big.Int).SetString("8000000080008008", 16)

	rcs = make([][]uint, 24)
	for i := 0; i < 24; i++ {
		rcs[i] = make([]uint, 64)
		for j := 0; j < 64; j++ {
			rcs[i][j] = rc[i].Bit(j)
		}
	}
}

func compareBits(api frontend.API, a []frontend.Variable, b []frontend.Variable) {
	if len(a) != len(b) {
		panic("gg")
	}
	for i := 0; i < len(a); i++ {
		a[i] = fromMyBitForm(api, a[i])
		b[i] = fromMyBitForm(api, b[i])
		api.Compiler().MarkBoolean(a[i])
		api.Compiler().MarkBoolean(b[i])
	}
	n := api.Compiler().FieldBitLen() - 1
	for i := 0; i < len(a); i += n {
		r := i + n
		if r > len(a) {
			r = len(a)
		}
		api.AssertIsEqual(api.FromBinary(a[i:r]...), api.FromBinary(b[i:r]...))
	}
}

func fromMyBitForm(api frontend.API, x frontend.Variable) frontend.Variable {
	return api.Div(api.Sub(1, x), 2)
}

func toMyBitForm(x int) int {
	if x == 0 {
		return 1
	}
	return -1
}

func xorIn(api frontend.API, s [][]frontend.Variable, buf [][]frontend.Variable) [][]frontend.Variable {
	for y := 0; y < 5; y++ {
		for x := 0; x < 5; x++ {
			if x+5*y < len(buf) {
				s[5*x+y] = xor(api, s[5*x+y], buf[x+5*y])
			}
		}
	}
	return s
}

func keccakF(api frontend.API, a [][]frontend.Variable) [][]frontend.Variable {
	var b [25][]frontend.Variable
	for i := 0; i < len(b); i++ {
		b[i] = make([]frontend.Variable, 64)
		for j := 0; j < 64; j++ {
			b[i][j] = 0
		}
	}
	var c [5][]frontend.Variable
	for i := 0; i < len(c); i++ {
		c[i] = make([]frontend.Variable, 64)
		for j := 0; j < 64; j++ {
			c[i][j] = 0
		}
	}
	var d [5][]frontend.Variable
	for i := 0; i < len(d); i++ {
		d[i] = make([]frontend.Variable, 64)
		for j := 0; j < 64; j++ {
			d[i][j] = 0
		}
	}
	var da [5][]frontend.Variable
	for i := 0; i < len(d); i++ {
		da[i] = make([]frontend.Variable, 64)
		for j := 0; j < 64; j++ {
			da[i][j] = 0
		}
	}

	for i := 0; i < 24; i++ {
		c[0] = xor(api, xor(api, a[1], a[2]), xor(api, a[3], a[4]))
		c[1] = xor(api, xor(api, a[6], a[7]), xor(api, a[8], a[9]))
		c[2] = xor(api, xor(api, a[11], a[12]), xor(api, a[13], a[14]))
		c[3] = xor(api, xor(api, a[16], a[17]), xor(api, a[18], a[19]))
		c[4] = xor(api, xor(api, a[21], a[22]), xor(api, a[23], a[24]))

		for j := 0; j < 5; j++ {
			d[j] = xor(api, c[(j+4)%5], rotateLeft(c[(j+1)%5], 1))
			da[j] = xor(api, a[((j+4)%5)*5], rotateLeft(a[((j+1)%5)*5], 1))
		}

		for j := 0; j < 25; j++ {
			tmp := xor(api, da[j/5], a[j])
			a[j] = xor(api, tmp, d[j/5])
		}

		/*Rho and pi steps*/
		b[0] = a[0]

		b[8] = rotateLeft(a[1], 36)
		b[11] = rotateLeft(a[2], 3)
		b[19] = rotateLeft(a[3], 41)
		b[22] = rotateLeft(a[4], 18)

		b[2] = rotateLeft(a[5], 1)
		b[5] = rotateLeft(a[6], 44)
		b[13] = rotateLeft(a[7], 10)
		b[16] = rotateLeft(a[8], 45)
		b[24] = rotateLeft(a[9], 2)

		b[4] = rotateLeft(a[10], 62)
		b[7] = rotateLeft(a[11], 6)
		b[10] = rotateLeft(a[12], 43)
		b[18] = rotateLeft(a[13], 15)
		b[21] = rotateLeft(a[14], 61)

		b[1] = rotateLeft(a[15], 28)
		b[9] = rotateLeft(a[16], 55)
		b[12] = rotateLeft(a[17], 25)
		b[15] = rotateLeft(a[18], 21)
		b[23] = rotateLeft(a[19], 56)

		b[3] = rotateLeft(a[20], 27)
		b[6] = rotateLeft(a[21], 20)
		b[14] = rotateLeft(a[22], 39)
		b[17] = rotateLeft(a[23], 8)
		b[20] = rotateLeft(a[24], 14)

		/*Xi state*/

		a[0] = xor(api, b[0], and(api, not(api, b[5]), b[10]))
		a[1] = xor(api, b[1], and(api, not(api, b[6]), b[11]))
		a[2] = xor(api, b[2], and(api, not(api, b[7]), b[12]))
		a[3] = xor(api, b[3], and(api, not(api, b[8]), b[13]))
		a[4] = xor(api, b[4], and(api, not(api, b[9]), b[14]))

		a[5] = xor(api, b[5], and(api, not(api, b[10]), b[15]))
		a[6] = xor(api, b[6], and(api, not(api, b[11]), b[16]))
		a[7] = xor(api, b[7], and(api, not(api, b[12]), b[17]))
		a[8] = xor(api, b[8], and(api, not(api, b[13]), b[18]))
		a[9] = xor(api, b[9], and(api, not(api, b[14]), b[19]))

		a[10] = xor(api, b[10], and(api, not(api, b[15]), b[20]))
		a[11] = xor(api, b[11], and(api, not(api, b[16]), b[21]))
		a[12] = xor(api, b[12], and(api, not(api, b[17]), b[22]))
		a[13] = xor(api, b[13], and(api, not(api, b[18]), b[23]))
		a[14] = xor(api, b[14], and(api, not(api, b[19]), b[24]))

		a[15] = xor(api, b[15], and(api, not(api, b[20]), b[0]))
		a[16] = xor(api, b[16], and(api, not(api, b[21]), b[1]))
		a[17] = xor(api, b[17], and(api, not(api, b[22]), b[2]))
		a[18] = xor(api, b[18], and(api, not(api, b[23]), b[3]))
		a[19] = xor(api, b[19], and(api, not(api, b[24]), b[4]))

		a[20] = xor(api, b[20], and(api, not(api, b[0]), b[5]))
		a[21] = xor(api, b[21], and(api, not(api, b[1]), b[6]))
		a[22] = xor(api, b[22], and(api, not(api, b[2]), b[7]))
		a[23] = xor(api, b[23], and(api, not(api, b[3]), b[8]))
		a[24] = xor(api, b[24], and(api, not(api, b[4]), b[9]))

		///*Last step*/

		for j := 0; j < len(a[0]); j++ {
			if rcs[i][j] == 1 {
				a[0][j] = api.Sub(0, a[0][j])
			}
		}
	}

	return a
}

func xor(api frontend.API, a []frontend.Variable, b []frontend.Variable) []frontend.Variable {
	nbits := len(a)
	bitsRes := make([]frontend.Variable, nbits)
	for i := 0; i < nbits; i++ {
		bitsRes[i] = api.Mul(a[i], b[i])
		bitsRes[i] = api.(gkr.API).ToSingleVariable(bitsRes[i])
	}
	return bitsRes
}

func and(api frontend.API, a []frontend.Variable, b []frontend.Variable) []frontend.Variable {
	nbits := len(a)
	bitsRes := make([]frontend.Variable, nbits)
	for i := 0; i < nbits; i++ {
		bitsRes[i] = api.Div(api.Add(api.Neg(api.Mul(a[i], b[i])), a[i], b[i], 1), 2)
		bitsRes[i] = api.(gkr.API).ToSingleVariable(bitsRes[i])
	}
	return bitsRes
}

func not(api frontend.API, a []frontend.Variable) []frontend.Variable {
	bitsRes := make([]frontend.Variable, len(a))
	for i := 0; i < len(a); i++ {
		bitsRes[i] = api.Sub(0, a[i])
	}
	return bitsRes
}

func rotateLeft(bits []frontend.Variable, k int) []frontend.Variable {
	n := uint(len(bits))
	s := uint(k) & (n - 1)
	newBits := bits[n-s:]
	return append(newBits, bits[:n-s]...)
}

type keccak256Circuit struct {
	S   [NHashes][25 * 64]frontend.Variable
	P   [NHashes][136 * 8]frontend.Variable
	Out [NHashes][25 * 64]frontend.Variable
}

func assertIsBoolean(api frontend.API, x frontend.Variable) {
	api.AssertIsEqual(api.Mul(x, x), 1)
}

func checkKeccak(api frontend.API, S, P, Out []frontend.Variable) {
	for i := 0; i < 25*64; i++ {
		assertIsBoolean(api, S[i])
	}
	for i := 0; i < 136*8; i++ {
		assertIsBoolean(api, P[i])
	}
	for i := 0; i < 25*64; i++ {
		assertIsBoolean(api, Out[i])
	}

	ss := make([][]frontend.Variable, 25)
	for i := 0; i < 25; i++ {
		ss[i] = make([]frontend.Variable, 64)
		for j := 0; j < 64; j++ {
			ss[i][j] = S[i*64+j]
		}
	}
	p := make([][]frontend.Variable, 17)
	for i := 0; i < 17; i++ {
		p[i] = make([]frontend.Variable, 64)
		for j := 0; j < 64; j++ {
			p[i][j] = P[i*64+j]
		}
	}
	ss = xorIn(api, ss, p)
	ss = keccakF(api, ss)
	outflat := []frontend.Variable{}
	for i := 0; i < 25; i++ {
		outflat = append(outflat, ss[i]...)
	}
	compareBits(api, outflat, Out)
}

func (t *keccak256Circuit) Define(api frontend.API) error {
	f := builder.MemorizedVoidFunc(checkKeccak)
	for i := 0; i < NHashes; i++ {
		f(api, t.S[i][:], t.P[i][:], t.Out[i][:])
	}
	return nil
}

func main() {
	var circuit keccak256Circuit

	cr, _ := gkr.Compile(m31.ScalarField, &circuit)
	//cr.Print()
	_ = cr

	c := cr.GetLayeredCircuit()
	//c.Print()
	os.WriteFile("circuit.txt", c.Serialize(), 0o644)

	for k := 0; k < NHashes; k++ {
		for i := 0; i < 25*64; i++ {
			circuit.S[k][i] = 1
		}
		for i := 0; i < 136*8; i++ {
			circuit.P[k][i] = 1
		}
		for i := 0; i < 25*64; i++ {
			circuit.Out[k][i] = 1
		}
		for i := 0; i < 10; i++ {
			t := 97 + i%3
			if i == 9 {
				t = 1
			}
			for j := 0; j < 8; j++ {
				circuit.P[k][i*8+j] = toMyBitForm((t >> j) & 1)
			}
		}

		out := make([]uint64, 25)
		out[0] = 17219205011983708464
		out[1] = 17263267039051296837
		out[2] = 4734580643271052073
		out[3] = 14210404430156729305
		out[4] = 14326798916247726117
		out[5] = 9741415084646213999
		out[6] = 7657700635244418443
		out[7] = 17064445256936182899
		out[8] = 9353040293697576515
		out[9] = 4956016039090590093
		out[10] = 14613965790834230574
		out[11] = 6082825785624035690
		out[12] = 14543440552284219774
		out[13] = 3844699381476460584
		out[14] = 1308273130786354139
		out[15] = 7511437080392509869
		out[16] = 2736747253291613912
		out[17] = 4285095010498297369
		out[18] = 5348191640999176495
		out[19] = 2055916821189839315
		out[20] = 9550189129561540020
		out[21] = 18176507538384442338
		out[22] = 10689654814838395219
		out[23] = 6386678527123596175
		out[24] = 10689114996529251139
		for i := 0; i < 25; i++ {
			for j := 0; j < 64; j++ {
				circuit.Out[k][i*64+j] = toMyBitForm(int((out[i] >> j) & 1))
			}
		}
	}

	wit, err := cr.GetInputSolver().SolveInput(&circuit, 8)
	if err != nil {
		panic("gg")
	}

	if !test.CheckCircuit(c, wit) {
		panic("gg")
	}

	for k := 0; k < NHashes; k++ {
		circuit.S[k][0] = -1
	}
	wit, err = cr.GetInputSolver().SolveInput(&circuit, 8)
	if err != nil {
		panic("gg")
	}

	if test.CheckCircuit(c, wit) {
		panic("gg")
	}
}
