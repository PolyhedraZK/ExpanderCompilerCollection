package utils

import "github.com/consensys/gnark/frontend"

type API interface {
	ToSingleVariable(frontend.Variable) frontend.Variable
}
