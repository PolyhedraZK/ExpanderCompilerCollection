package test

import (
	"testing"

	"github.com/Zklib/gkr-compiler"
	"github.com/consensys/gnark/frontend"
)

type Assert struct {
	t *testing.T
}

func NewAssert(t *testing.T) *Assert {
	return &Assert{t: t}
}

func (a *Assert) ProveSucceeded(cr *gkr.CompileResult, assignment frontend.Circuit) {
	lc := cr.GetLayeredCircuit()
	witness := cr.GetWitness(assignment)
	if !CheckCircuit(lc, witness) {
		a.t.Fatal("should succeed")
	}
}

func (a *Assert) ProveFailed(cr *gkr.CompileResult, assignment frontend.Circuit) {
	lc := cr.GetLayeredCircuit()
	witness := cr.GetWitness(assignment)
	if CheckCircuit(lc, witness) {
		a.t.Fatal("should fail")
	}
}
