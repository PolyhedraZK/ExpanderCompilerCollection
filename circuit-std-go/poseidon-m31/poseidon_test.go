package poseidonM31

import (
	"testing"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/test"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/field/m31"
	"github.com/consensys/gnark/frontend"
	"github.com/stretchr/testify/require"
)

func TestPoseidonM31x16Params(t *testing.T) {
	require.Equal(t,
		uint(80596940),
		poseidonM31x16RoundConstant[0][0],
		"poseidon round constant m31x16 0.0 not matching ggs",
	)
}

func TestPoseidonM31x16HashToState(t *testing.T) {

	testcases := []struct {
		InputLen   uint
		Assignment PoseidonM31x16Sponge
	}{
		{
			InputLen: 8,
			Assignment: PoseidonM31x16Sponge{
				ToBeHashed: []frontend.Variable{
					114514, 114514, 114514, 114514,
					114514, 114514, 114514, 114514,
				},
				Digest: [16]frontend.Variable{
					1021105124, 1342990709, 1593716396, 2100280498,
					330652568, 1371365483, 586650367, 345482939,
					849034538, 175601510, 1454280121, 1362077584,
					528171622, 187534772, 436020341, 1441052621,
				},
			},
		},
		{
			InputLen: 16,
			Assignment: PoseidonM31x16Sponge{
				ToBeHashed: []frontend.Variable{
					114514, 114514, 114514, 114514,
					114514, 114514, 114514, 114514,
					114514, 114514, 114514, 114514,
					114514, 114514, 114514, 114514,
				},
				Digest: [16]frontend.Variable{
					1510043913, 1840611937, 45881205, 1134797377,
					803058407, 1772167459, 846553905, 2143336151,
					300871060, 545838827, 1603101164, 396293243,
					502075988, 2067011878, 402134378, 535675968,
				},
			},
		},
	}

	for _, testcase := range testcases {
		circuit := PoseidonM31x16Sponge{
			ToBeHashed: make([]frontend.Variable, testcase.InputLen),
		}
		circuitCompileResult, err := ecgo.Compile(
			m31.ScalarField,
			&circuit,
		)
		require.NoError(t, err, "ggs compile circuit error")
		layeredCircuit := circuitCompileResult.GetLayeredCircuit()

		inputSolver := circuitCompileResult.GetInputSolver()
		witness, err := inputSolver.SolveInput(&testcase.Assignment, 0)
		require.NoError(t, err, "ggs solving witness error")

		require.True(
			t,
			test.CheckCircuit(layeredCircuit, witness),
			"ggs check circuit error",
		)
	}
}
