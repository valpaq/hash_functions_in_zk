use ff::PrimeField;

#[derive(Clone, Debug)]
pub struct RescuePrimeParams<S: PrimeField> {
    pub(crate) t: usize, // statesize
    pub(crate) d: usize, // sbox degree
    pub(crate) d_inv: [u64; 4],
    pub(crate) rounds: usize,
    pub(crate) mds: Vec<Vec<S>>,
    pub(crate) round_constants: Vec<Vec<S>>,
}

impl<S: PrimeField> RescuePrimeParams<S> {
    pub fn new(
        t: usize,
        d: usize,
        d_inv: [u64; 4],
        rounds: usize,
        mds: &[Vec<S>],
        round_constants: &[Vec<S>],
    ) -> Self {
        assert!(d == 3 || d == 5);
        assert_eq!(mds.len(), t);

        RescuePrimeParams {
            t,
            d,
            d_inv,
            rounds,
            mds: mds.to_owned(),
            round_constants: round_constants.to_owned(),
        }
    }
}
