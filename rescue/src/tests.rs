use franklin_crypto::{
    bellman::plonk::better_better_cs::cs::{TrivialAssembly, Width4MainGateWithDNext, PlonkCsWidth4WithNextStepParams},
    bellman::Engine,
    plonk::circuit::{Width4WithCustomGates},
};
use rand::{SeedableRng, XorShiftRng};

pub(crate) fn init_rng() -> XorShiftRng {
    XorShiftRng::from_seed(crate::common::TEST_SEED)
}
pub(crate) fn init_cs<E: Engine>(
) -> TrivialAssembly<E, Width4WithCustomGates, Width4MainGateWithDNext> {
    TrivialAssembly::<E, Width4WithCustomGates, Width4MainGateWithDNext>::new()
}
pub(crate) fn init_cs_no_custom_gate<E: Engine>(
) -> TrivialAssembly<E, PlonkCsWidth4WithNextStepParams, Width4MainGateWithDNext> {
    TrivialAssembly::<E, PlonkCsWidth4WithNextStepParams, Width4MainGateWithDNext>::new()
}
