package builder

type variable struct {
	id int
}

func newVariable(id int) variable {
	return variable{id: id}
}

func unwrapVariables(vars []variable) []int {
	res := make([]int, len(vars))
	for i, v := range vars {
		res[i] = v.id
	}
	return res
}
