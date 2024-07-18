package builder

import (
	"math/big"
	"runtime"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/expr"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ir"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/utils"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/schema"
)

// ProfilingRoot wraps a Root to facilitate profiling.
type ProfilingRoot struct {
	root   *Root
	rootPb *profilingBuilder
}

type profilingBuilder struct {
	builder *builder
	proot   *ProfilingRoot

	lastVariableIdx int

	varSourceInfo []utils.SourceInfo
	callDep       int
}

// NewProfilingRoot wraps a Root for profiling purposes.
func NewProfilingRoot(fieldorder *big.Int, config frontend.CompileConfig) *ProfilingRoot {
	root := NewRoot(fieldorder, config)
	pb := &profilingBuilder{
		builder:         root.builder,
		lastVariableIdx: 1,
		varSourceInfo:   []utils.SourceInfo{{File: "one", Line: 0}},
	}
	pr := &ProfilingRoot{
		root:   root,
		rootPb: pb,
	}
	pb.proot = pr
	return pr
}

// GetRootBuilder retrieves the builder used for profiling.
func (r *ProfilingRoot) GetRootBuilder() *profilingBuilder {
	return r.rootPb
}

func (b *profilingBuilder) entry() {
	b.callDep++
}

func (b *profilingBuilder) record() {
	b.callDep--
	if b.callDep != 0 {
		return
	}
	_, file, line, ok := runtime.Caller(2)
	var si utils.SourceInfo
	if ok {
		si = utils.SourceInfo{File: file, Line: line}
	} else {
		si = utils.SourceInfo{File: "unknown", Line: 0}
	}
	for b.lastVariableIdx < len(b.builder.vLayer)-1 {
		b.varSourceInfo = append(b.varSourceInfo, si)
		b.lastVariableIdx++
	}
}

func (b *profilingBuilder) recordOther(typ string) {
	si := utils.SourceInfo{File: typ, Line: 0}
	for b.lastVariableIdx < len(b.builder.vLayer)-1 {
		b.varSourceInfo = append(b.varSourceInfo, si)
		b.lastVariableIdx++
	}
}

func (b *profilingBuilder) MemorizedSimpleCall(f SubCircuitSimpleFunc, input []frontend.Variable) []frontend.Variable {
	panic("sub-circuit calling is currently unsupported in profiling mode")
}

func (b *profilingBuilder) MemorizedCall(fn SubCircuitFunc, inputs ...interface{}) interface{} {
	panic("sub-circuit calling is currently unsupported in profiling mode")
}

// Finalize is similar to Root.Finalize but includes profiling data.
func (pr *ProfilingRoot) Finalize() (*ir.RootCircuit, []utils.SourceInfo) {
	pr.rootPb.callDep++
	res := pr.root.Finalize()
	pr.rootPb.callDep--
	pr.rootPb.recordOther("finalize")

	c := res.Circuits[0]
	for i, e := range c.Constraints {
		c.Constraints[i] = pr.root.ToSingleVariable(e).(expr.Expression)
	}
	c.Output = []expr.Expression{}
	c.Instructions = pr.root.instructions
	pr.rootPb.recordOther("finalize")

	return res, pr.rootPb.varSourceInfo
}

// ------------------ root.go ------------------

func (builder *profilingBuilder) PublicVariable(f schema.LeafInfo) frontend.Variable {
	if builder.builder.root.builder != builder.builder {
		panic("unexpected")
	}
	defer builder.recordOther("input")
	return builder.builder.root.PublicVariable(f)
}

func (builder *profilingBuilder) SecretVariable(f schema.LeafInfo) frontend.Variable {
	if builder.builder.root.builder != builder.builder {
		panic("unexpected")
	}
	defer builder.recordOther("input")
	return builder.builder.root.SecretVariable(f)
}

// ------------------ api.go ------------------

func (builder *profilingBuilder) Add(i1, i2 frontend.Variable, in ...frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.Add(i1, i2, in...)
}
func (builder *profilingBuilder) MulAcc(a, b, c frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.MulAcc(a, b, c)
}
func (builder *profilingBuilder) Sub(i1, i2 frontend.Variable, in ...frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.Sub(i1, i2, in...)
}
func (builder *profilingBuilder) Neg(i frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.Neg(i)
}
func (builder *profilingBuilder) Mul(i1, i2 frontend.Variable, in ...frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.Mul(i1, i2, in...)
}
func (builder *profilingBuilder) DivUnchecked(i1, i2 frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.DivUnchecked(i1, i2)
}
func (builder *profilingBuilder) Div(i1, i2 frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.Div(i1, i2)
}
func (builder *profilingBuilder) Inverse(i1 frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.Inverse(i1)
}
func (builder *profilingBuilder) ToBinary(i1 frontend.Variable, n ...int) []frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.ToBinary(i1, n...)
}
func (builder *profilingBuilder) FromBinary(_b ...frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.FromBinary(_b...)
}
func (builder *profilingBuilder) Xor(_a, _b frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.Xor(_a, _b)
}
func (builder *profilingBuilder) Or(_a, _b frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.Or(_a, _b)
}
func (builder *profilingBuilder) And(_a, _b frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.And(_a, _b)
}
func (builder *profilingBuilder) Select(i0, i1, i2 frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.Select(i0, i1, i2)
}
func (builder *profilingBuilder) Lookup2(b0, b1 frontend.Variable, i0, i1, i2, i3 frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.Lookup2(b0, b1, i0, i1, i2, i3)
}
func (builder *profilingBuilder) IsZero(i1 frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.IsZero(i1)
}
func (builder *profilingBuilder) Cmp(i1, i2 frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.Cmp(i1, i2)
}
func (builder *profilingBuilder) Println(a ...frontend.Variable) {
	builder.entry()
	defer builder.record()
	builder.builder.Println(a...)
}
func (builder *profilingBuilder) Compiler() frontend.Compiler {
	return builder
}
func (builder *profilingBuilder) Commit(v ...frontend.Variable) (frontend.Variable, error) {
	builder.entry()
	defer builder.record()
	return builder.builder.Commit(v...)
}
func (builder *profilingBuilder) SetGkrInfo(info constraint.GkrInfo) error {
	builder.entry()
	defer builder.record()
	return builder.builder.SetGkrInfo(info)
}
func (builder *profilingBuilder) Output(x_ frontend.Variable) {
	builder.entry()
	defer builder.record()
	builder.builder.Output(x_)
}

// ------------------ builder.go ------------------

func (builder *profilingBuilder) ToSingleVariable(ein frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.ToSingleVariable(ein)
}
func (builder *profilingBuilder) Field() *big.Int {
	return builder.builder.Field()
}
func (builder *profilingBuilder) FieldBitLen() int {
	return builder.builder.FieldBitLen()
}
func (builder *profilingBuilder) LayerOf(e frontend.Variable) int {
	return builder.builder.LayerOf(e)
}
func (builder *profilingBuilder) MarkBoolean(v frontend.Variable) {
	builder.builder.MarkBoolean(v)
}
func (builder *profilingBuilder) IsBoolean(v frontend.Variable) bool {
	return builder.builder.IsBoolean(v)
}
func (builder *profilingBuilder) Compile() (constraint.ConstraintSystem, error) {
	return builder.builder.Compile()
}
func (builder *profilingBuilder) ConstantValue(v frontend.Variable) (*big.Int, bool) {
	return builder.builder.ConstantValue(v)
}
func (builder *profilingBuilder) NewHint(f solver.Hint, nbOutputs int, inputs ...frontend.Variable) ([]frontend.Variable, error) {
	builder.entry()
	defer builder.record()
	return builder.builder.NewHint(f, nbOutputs, inputs...)
}
func (builder *profilingBuilder) NewHintForId(id solver.HintID, nbOutputs int, inputs ...frontend.Variable) ([]frontend.Variable, error) {
	builder.entry()
	defer builder.record()
	return builder.builder.NewHintForId(id, nbOutputs, inputs...)
}
func (builder *profilingBuilder) CustomGate(gateType uint64, inputs ...frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.CustomGate(gateType, inputs...)
}
func (builder *profilingBuilder) ToFirstLayer(v frontend.Variable) frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.ToFirstLayer(v)
}
func (builder *profilingBuilder) Defer(cb func(frontend.API) error) {
	builder.builder.Defer(cb)
}
func (builder *profilingBuilder) AddInstruction(bID constraint.BlueprintID, calldata []uint32) []uint32 {
	panic("unimplemented")
}
func (builder *profilingBuilder) AddBlueprint(b constraint.Blueprint) constraint.BlueprintID {
	panic("unimplemented")
}
func (builder *profilingBuilder) InternalVariable(wireID uint32) frontend.Variable {
	return builder.builder.InternalVariable(wireID)
}
func (builder *profilingBuilder) ToCanonicalVariable(in frontend.Variable) frontend.CanonicalVariable {
	return builder.builder.ToCanonicalVariable(in)
}
func (builder *profilingBuilder) SetKeyValue(key, value any) {
	builder.builder.SetKeyValue(key, value)
}
func (builder *profilingBuilder) GetKeyValue(key any) any {
	return builder.builder.GetKeyValue(key)
}
func (builder *profilingBuilder) GetRandomValue() frontend.Variable {
	builder.entry()
	defer builder.record()
	return builder.builder.GetRandomValue()
}

// ------------------ api_assertions.go ------------------

func (builder *profilingBuilder) AssertIsEqual(i1, i2 frontend.Variable) {
	builder.entry()
	defer builder.record()
	builder.builder.AssertIsEqual(i1, i2)
}
func (builder *profilingBuilder) AssertIsDifferent(i1, i2 frontend.Variable) {
	builder.entry()
	defer builder.record()
	builder.builder.AssertIsDifferent(i1, i2)
}
func (builder *profilingBuilder) AssertIsBoolean(i1 frontend.Variable) {
	builder.entry()
	defer builder.record()
	builder.builder.AssertIsBoolean(i1)
}
func (builder *profilingBuilder) AssertIsCrumb(i1 frontend.Variable) {
	builder.entry()
	defer builder.record()
	builder.builder.AssertIsCrumb(i1)
}
func (builder *profilingBuilder) AssertIsLessOrEqual(v frontend.Variable, bound frontend.Variable) {
	builder.entry()
	defer builder.record()
	builder.builder.AssertIsLessOrEqual(v, bound)
}
func (builder *profilingBuilder) MustBeLessOrEqCst(aBits []frontend.Variable, bound *big.Int, aForDebug frontend.Variable) {
	builder.entry()
	defer builder.record()
	builder.builder.MustBeLessOrEqCst(aBits, bound, aForDebug)
}
