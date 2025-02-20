use std::cell::RefCell;
use std::rc::Rc;

use expander_compiler::frontend::*;

declare_circuit!(Circuit {
    input: PublicVariable,
});

fn to_binary<C: Config>(api: &mut impl RootAPI<C>, x: Variable, n_bits: usize) -> Vec<Variable> {
    api.new_hint("myhint.tobinary", &[x], n_bits)
}

fn from_binary<C: Config>(api: &mut impl RootAPI<C>, bits: Vec<Variable>) -> Variable {
    let mut res = api.constant(0);
    for i in 0..bits.len() {
        let coef = 1 << i;
        let cur = api.mul(coef, bits[i]);
        res = api.add(res, cur);
    }
    res
}

impl Define<M31Config> for Circuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let bits = to_binary(builder, self.input, 8);
        let x = from_binary(builder, bits);
        builder.assert_is_equal(x, self.input);
    }
}

fn to_binary_hint(x: &[M31], y: &mut [M31]) -> Result<(), Error> {
    let t = x[0].to_u256();
    for (i, k) in y.iter_mut().enumerate() {
        *k = M31::from_u256(t >> i as u32 & 1);
    }
    Ok(())
}

#[test]
fn test_300() {
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.tobinary", to_binary_hint);

    let compile_result = compile(&Circuit::default(), CompileOptions::default()).unwrap();
    for i in 0..300 {
        let assignment = Circuit::<M31> {
            input: M31::from(i as u32),
        };
        let witness = compile_result
            .witness_solver
            .solve_witness_with_hints(&assignment, &mut hint_registry)
            .unwrap();
        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![i < 256]);
    }
}

#[test]
fn test_300_closure() {
    let mut hint_registry = HintRegistry::<M31>::new();
    let call_count = Rc::new(RefCell::new(0));
    let call_count_clone = call_count.clone();
    hint_registry.register(
        "myhint.tobinary",
        move |x: &[M31], y: &mut [M31]| -> Result<(), Error> {
            *call_count_clone.borrow_mut() += 1;
            let t = x[0].to_u256();
            for (i, k) in y.iter_mut().enumerate() {
                *k = M31::from_u256(t >> i as u32 & 1);
            }
            Ok(())
        },
    );

    let compile_result = compile(&Circuit::default(), CompileOptions::default()).unwrap();
    for i in 0..300 {
        let assignment = Circuit::<M31> {
            input: M31::from(i as u32),
        };
        let witness = compile_result
            .witness_solver
            .solve_witness_with_hints(&assignment, &mut hint_registry)
            .unwrap();
        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![i < 256]);
    }
    assert_eq!(*call_count.borrow(), 300);
}
