**This example faulty since `Commit` returns a compile-time random number.**

Also, this example requires a patching on gnark `frontend/variable.go` to recognize our expression type:

```go
func IsCanonical(v Variable) bool {
	switch v.(type) {
	case expr.LinearExpression, *expr.LinearExpression, expr.Term, *expr.Term:
		return true
	}
	if reflect.TypeOf(v).String() == "expr.Expression" {
		return true
	}
	return false
}
```