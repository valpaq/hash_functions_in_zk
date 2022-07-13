use super::sbox::*;
use crate::traits::{HashParams};
use franklin_crypto::bellman::plonk::better_better_cs::cs::ConstraintSystem;
use franklin_crypto::bellman::SynthesisError;
use franklin_crypto::{
    bellman::Engine,
    plonk::circuit::{linear_combination::LinearCombination},
};

pub fn matrix_vector_product<E: Engine, const DIM: usize>(
    matrix: &[[E::Fr; DIM]; DIM],
    vector: &mut [LinearCombination<E>; DIM],
) -> Result<(), SynthesisError> {
    let vec_cloned = vector.clone();

    for (idx, row) in matrix.iter().enumerate() {
        // [fr, fr, fr] * [lc, lc, lc]
        vector[idx] = LinearCombination::zero();
        for (factor, lc) in row.iter().zip(&vec_cloned) {
            vector[idx].add_assign_scaled(lc, *factor)
        }
    }

    Ok(())
}

pub(crate) fn gadget_rescue_prime_round_function<
    E: Engine,
    CS: ConstraintSystem<E>,
    P: HashParams<E, RATE, WIDTH>,
    const RATE: usize,
    const WIDTH: usize,
>(
    cs: &mut CS,
    params: &P,
    state: &mut [LinearCombination<E>; WIDTH],
) -> Result<(), SynthesisError> {

    for round in 0..params.number_of_full_rounds() - 1 {
        // apply sbox
        // each lc will have 3 terms but there will be 1 in first iteration
        // total cost 2 gate per state vars = 6
        sbox(
            cs,
            params.alpha(),
            state,
            None,
            params.custom_gate(),
        )?;

        // mul by mds
        matrix_vector_product(&params.mds_matrix(), state)?;

        // round constants
        let constants = params.constants_of_round(round);
        for (s, c) in state.iter_mut().zip(constants.iter().cloned()) {
            s.add_assign_constant(c);
        }
        // apply inverse sbox
        sbox(
            cs,
            params.alpha_inv(),
            state,
            None,
            params.custom_gate(),
        )?;

        // mul by mds
        matrix_vector_product(&params.mds_matrix(), state)?;

        // round constants
        let constants = params.constants_of_round(round + 1);
        for (s, c) in state.iter_mut().zip(constants.iter().cloned()) {
            s.add_assign_constant(c);
        }
    }
    Ok(())
}
