use expander_compiler::frontend::*;

pub fn simple_select<C: Config, B: RootAPI<C>>(
    native: &mut B,
    selector: Variable,
    a: Variable,
    b: Variable,
) -> Variable {
    let tmp = native.sub(a, b);
    let tmp2 = native.mul(tmp, selector);
    native.add(b, tmp2)
}

//return i0 if selector0 and selector 1 are 0
//return i1 if selector0 is 1 and selector1 is 0
//return i2 if selector0 is 0 and selector1 is 1
//return d if selector0 and selector1 are 1
pub fn simple_lookup2<C: Config, B: RootAPI<C>>(
    native: &mut B,
    selector0: Variable,
    selector1: Variable,
    i0: Variable,
    i1: Variable,
    i2: Variable,
    i3: Variable,
) -> Variable {
    let tmp0 = simple_select(native, selector0, i1, i0);
    let tmp1 = simple_select(native, selector0, i3, i2);
    simple_select(native, selector1, tmp1, tmp0)
}
