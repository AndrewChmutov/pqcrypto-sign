use std::marker::PhantomData;

use num_complex::{Complex, Complex64};
use num_traits::identities::Zero;

use crate::common::Polynomial;

struct PublicParameters {
    n: usize,
    sigma: f64,
    sigmin: f64,
    sig_bound: i64,
    sig_bytelen: usize,
}

trait Falcon {
    const PARAMETERS: PublicParameters;
}

struct Falcon512;
impl Falcon for Falcon512 {
    const PARAMETERS: PublicParameters = PublicParameters {
        n: 512,
        sigma: 165.7366171829776,
        sigmin: 1.2778336969128337,
        sig_bound: 34034726,
        sig_bytelen: 666,
    };
}

struct Falcon1024;
impl Falcon for Falcon1024 {
    const PARAMETERS: PublicParameters = PublicParameters {
        n: 1024,
        sigma: 168.38857144654395,
        sigmin: 1.298280334344292,
        sig_bound: 70265242,
        sig_bytelen: 1280,
    };
}

pub(crate) enum LdlTree {
    Branch(Polynomial<Complex64>, [Box<LdlTree>; 2]),
    Leaf([Complex64; 2]),
}

impl LdlTree {
    fn normalize_inplace(&mut self, sigma: f64) {
        match self {
            LdlTree::Branch(_, children) => {
                children.iter_mut().for_each(|c| c.normalize_inplace(sigma))
            }
            LdlTree::Leaf(v) => {
                v[0] = Complex::new(sigma / v[0].re.sqrt(), 0.0);
                v[1] = Complex64::zero();
            }
        }
    }

    fn normalize(mut self, sigma: f64) -> Self {
        self.normalize_inplace(sigma);
        self
    }
}

type Polynomials = [Polynomial<i16>; 4];
struct SecretKey<F: Falcon> {
    polys: Polynomials,
    _marker: PhantomData<F>,
}

impl<F: Falcon> SecretKey<F> {
    fn new(polys: Polynomials) -> Self {
        todo!()
    }
}

struct PublicKey<F: Falcon> {
    h: Polynomial<i16>,
    _marker: PhantomData<F>,
}

struct Signature<F: Falcon> {
    _marker: PhantomData<F>,
}

impl<F: Falcon> Signature<F> {
    fn sign(message: &[u8], sk: &SecretKey<F>) -> Self {
        todo!()
    }

    fn verify(&self, message: &[u8], pk: &PublicKey<F>) -> bool {
        todo!()
    }
}
