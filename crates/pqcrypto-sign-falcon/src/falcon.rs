use std::marker::PhantomData;

use num_complex::{Complex, Complex64};
use num_traits::identities::Zero;
use rand::{rng, CryptoRng};

use crate::encoding::{Compression, Decompression};
use crate::common::Polynomial;
use crate::ntt::NTT;

pub const HEAD_LEN: usize = 1;
pub const SALT_LEN: usize = 40;
pub const SEED_LEN: usize = 32;

type Seed = [u8; SEED_LEN];

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

    fn sign(&self, message: &[u8]) -> Signature {
        self.sign_rng(message, rng())
    }

    fn sign_rng(&self, message: &[u8], rng: impl CryptoRng) -> Signature {
        todo!()
    }
}

impl<F: Falcon> From<Seed> for SecretKey<F>{
    fn from(value: Seed) -> Self {
        todo!()
    }
}

struct PublicKey<F: Falcon> {
    h: Polynomial<i16>,
    _marker: PhantomData<F>,
}

impl<F: Falcon> PublicKey<F> {
    fn verify<D: Decompression>(&self, message: &[u8], signature: Signature) -> bool {
        let params = F::PARAMETERS;
        let Some(s1) = D::decompress(&signature.content, signature.content.len(), params.n) else {
            return false;
        };

        let hashed: u32 = Polynomial::hash_to_point(message, &signature.salt, params.n);
        let norm_sign: i64 = todo!();

        norm_sign < params.sig_bound
    }
}

impl<F: Falcon> From<&SecretKey<F>> for PublicKey<F>{
    fn from(value: &SecretKey<F>) -> Self {
        todo!()
    }
}

struct Signature {
    head: u8,
    salt: [u8; SALT_LEN],
    content: Vec<u8>,
}

impl TryFrom<&[u8]> for Signature {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let [head, salt_and_content @ ..] = value else {
            return Err("Could not parse head".into());
        };
        if salt_and_content.len() < SALT_LEN {
            return Err("Could not parse salt".into());
        }
        let (salt, content) = salt_and_content.split_at(SALT_LEN);

        Ok(Signature {
            head: *head,
            salt: salt.try_into().unwrap(),
            content: content.to_vec(),
        })
    }
}

fn keygen<F: Falcon>(seed: Seed) -> (SecretKey<F>, PublicKey<F>) {
    let sk = SecretKey::from(seed);
    let pk = PublicKey::from(&sk);
    (sk, pk)
}
