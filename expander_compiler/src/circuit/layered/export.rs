use super::*;

impl<C: Config> Circuit<C, NormalInputType> {
    pub fn export_to_expander<
        DestConfig: expander_config::GKRConfig<CircuitField = C::CircuitField>,
    >(
        &self,
    ) -> expander_circuit::RecursiveCircuit<DestConfig> {
        panic!("TODO")
    }
}
