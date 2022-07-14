mod rescue_prime_params;
use rescue_prime_params::RescuePrimeParams;
mod constants;
extern crate ff;
use ff::*;


#[derive(PrimeField)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "7"]
pub struct Fr(FrRepr);

use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct RescuePrime<S: PrimeField> {
    pub(crate) params: Arc<RescuePrimeParams<S>>,
}

impl<S: PrimeField> RescuePrime<S> {
    pub fn new(params: &Arc<RescuePrimeParams<S>>) -> Self {
        RescuePrime {
            params: Arc::clone(params),
        }
    }

    pub fn get_t(&self) -> usize {
        self.params.t
    }

    pub fn permutation(&self, input: &[S]) -> Vec<S> {
        let t = self.params.t;
        assert_eq!(input.len(), t);

        let mut current_state = input.to_owned();

        for r in 0..self.params.rounds {
            current_state = self.sbox(&current_state);
            current_state = self.affine(&current_state, 2 * r);
            current_state = self.sbox_inverse(&current_state);
            current_state = self.affine(&current_state, 2 * r + 1);
        }

        current_state
    }

    fn sbox(&self, input: &[S]) -> Vec<S> {
        input
            .iter()
            .map(|el| {
                let mut el2 = *el;
                el2.square();

                match self.params.d {
                    3 => {
                        let mut out = el2;
                        out.mul_assign(el);
                        out
                    }
                    5 => {
                        let mut out = el2;
                        out.square();
                        out.mul_assign(el);
                        out
                    }
                    _ => {
                        panic!();
                    }
                }
            })
            .collect()
    }

    fn sbox_inverse(&self, input: &[S]) -> Vec<S> {
        input
            .iter()
            .map(|el| {
                if *el == S::zero() {
                    *el
                } else {
                    el.pow(&self.params.d_inv)
                }
            })
            .collect()
    }

    fn affine(&self, input: &[S], round: usize) -> Vec<S> {
        let mat_result = self.matmul(input, &self.params.mds);
        Self::add_rc(&mat_result, &self.params.round_constants[round])
    }

    fn matmul(&self, input: &[S], mat: &[Vec<S>]) -> Vec<S> {
        let t = mat.len();
        debug_assert!(t == input.len());
        let mut out = vec![S::zero(); t];
        // TODO check if really faster
        for row in 0..t {
            for (col, inp) in input.iter().enumerate().take(t) {
                let mut tmp = mat[row][col];
                tmp.mul_assign(inp);
                out[row].add_assign(&tmp);
            }
        }
        out
    }

    fn add_rc(input: &[S], round_constants: &[S]) -> Vec<S> {
        debug_assert!(input.len() == round_constants.len());
        input
            .iter()
            .zip(round_constants.iter())
            .map(|(a, b)| {
                let mut r = *a;
                r.add_assign(b);
                r
            })
            .collect()
    }
}

#[cfg(test)]
mod rescue_prime_tests {
    use super::*;
    use rand::thread_rng;
    // use constants::RESCUE_PRIME_BN_PARAMS;
    use ff::{from_hex, Field, PrimeField};
    
    use constants::*;
    
    type Scalar = Fr;

    static TESTRUNS: usize = 5;

    fn random_scalar<F: PrimeField>(allow_zero: bool) -> F {
        loop {
            let s = F::rand(&mut thread_rng());
            if allow_zero || s != F::zero() {
                return s;
            }
        }
    }

    fn from_u64<F: PrimeField>(val: u64) -> F {
        F::from_repr(F::Repr::from(val)).unwrap()
    }
    

    #[test]
    fn consistent_perm() {
        let rescue_prime = RescuePrime::new(&RESCUE_PRIME_BN_PARAMS);
        let t = rescue_prime.params.t;
        for _ in 0..TESTRUNS {
            let input1: Vec<Scalar> = (0..t).map(|_| random_scalar(true)).collect();

            let mut input2: Vec<Scalar>;
            loop {
                input2 = (0..t).map(|_| random_scalar(true)).collect();
                if input1 != input2 {
                    break;
                }
            }

            let perm1 = rescue_prime.permutation(&input1);
            let perm2 = rescue_prime.permutation(&input1);
            let perm3 = rescue_prime.permutation(&input2);
            assert_eq!(perm1, perm2);
            assert_ne!(perm1, perm3);
        }
    }

    #[test]
    fn kats() {
        let rescue_prime = RescuePrime::new(&RESCUE_PRIME_BN_PARAMS);
        let input: Vec<Scalar> = vec![Scalar::zero(), Scalar::one(), from_u64::<Scalar>(2)];
        let perm = rescue_prime.permutation(&input);
        assert_eq!(
            perm[0],
            from_hex("0x0dc30ccd5d64e5bea071e99087ef86d433eb156aa0500a823298f9bb05328bd2").unwrap()
        );
        assert_eq!(
            perm[1],
            from_hex("0x189893368d5815608c56e44cc67f7e821e093bb6254a0553f9ff69f4d99debc8").unwrap(),
        );
        assert_eq!(
            perm[2],
            from_hex("0x1acafc768221448ebc51fa2cd1e3c9b2044a0c04f3509d833b0a82c7e3462610").unwrap(),
        );
    }
}
