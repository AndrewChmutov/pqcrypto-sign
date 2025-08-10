use crate::common::{Polynomial, Q};

include!(concat!(env!("OUT_DIR"), "/ntt_constants.rs"));

pub trait NTT: Sized {
    fn split_ntt(self) -> (Self, Self);
    fn merge_ntt(self, other: Self) -> Self;
    fn ntt(self) -> Self;
    fn intt(self) -> Self;
    fn add(self, other: Self) -> Self;
    fn neg(self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn mul(self, other: Self) -> Self;
    fn div(self, other: Self) -> Option<Self>;
    fn add_ntt(self, other: Self) -> Self;
    fn sub_ntt(self, other: Self) -> Self;
    fn mul_ntt(self, other: Self) -> Self;
    fn div_ntt(self, other: Self) -> Option<Self>;
}

const I2: u32 = 6145;
const SQR1: u16 = roots_Zq(2)[0];

impl NTT for Polynomial<u32> {
    fn split_ntt(self) -> (Self, Self) {
        let n = self.len();
        let w = roots_Zq(n as u16);
        self.coefficients
            .chunks_exact(2)
            .enumerate()
            .map(|(i, chunks)| {
                let [even, odd] = chunks else { unreachable!() };
                let f0_ntt = (I2 * (even + odd)) % Q;
                let f1_ntt = (I2 * (even - odd) * (INV_MOD_Q[w[2 * i] as usize] as u32)) % Q;
                (f0_ntt, f1_ntt)
            })
            .unzip()
    }

    fn merge_ntt(self, other: Self) -> Self {
        let n = 2 * self.len();
        let w = roots_Zq(n as u16);
        self.coefficients
            .iter()
            .enumerate()
            .zip(other.coefficients)
            .flat_map(|((i, a), b)| [
                (a + w[2 * i] as u32 * b) % Q,
                (a - w[2 * i] as u32 * b) % Q
            ])
            .collect()
    }

    fn ntt(self) -> Self {
        match self.coefficients.as_slice() {
            [f0, f1] => Polynomial {
                coefficients: vec![
                    (f0 + SQR1 as u32 * f1) % Q,
                    (f0 - SQR1 as u32 * f1) % Q,
                ]
            },
            _ => {
                let (f0, f1) = self.split();
                let f0_ntt = f0.ntt();
                let f1_ntt = f1.ntt();
                f0_ntt.merge_ntt(f1_ntt)
            }
        }
    }

    fn intt(self) -> Self {
        match self.coefficients.as_slice() {
            [f0_ntt, f1_ntt] => Polynomial {
                coefficients: vec![
                    (I2 * (f0_ntt + f1_ntt)) % Q,
                    (I2 * INV_MOD_Q[SQR1 as usize] as u32 * (f0_ntt - f1_ntt)) % Q,
                ]
            },
            _ => {
                let (f0_ntt, f1_ntt) = self.split_ntt();
                let f0 = f0_ntt.intt();
                let f1 = f1_ntt.intt();
                f0.merge(f1)
            }
        }
    }

    fn add(self, other: Self) -> Self {
        debug_assert_eq!(
            self.coefficients.len(),
            other.coefficients.len()
        );
        self.coefficients.into_iter()
            .zip(other.coefficients.into_iter())
            .map(|(a, b)| (a + b) % Q)
            .collect()
    }

    fn neg(self) -> Self {
        self.coefficients.into_iter().map(|a| Q - (a % Q)).collect()
    }

    fn sub(self, other: Self) -> Self {
        self.add(other.neg())
    }

    fn mul(self, other: Self) -> Self {
        self.ntt()
            .mul_ntt(other.ntt())
            .intt()
    }

    fn div(self, other: Self) -> Option<Self> {
        self.ntt()
            .div_ntt(other.ntt())
            .map(|p| p.intt())
    }

    fn add_ntt(self, other: Self) -> Self {
        self.add(other)
    }

    fn sub_ntt(self, other: Self) -> Self {
        self.sub(other)
    }

    fn mul_ntt(self, other: Self) -> Self {
        self.coefficients.into_iter()
            .zip(other.coefficients.into_iter())
            .map(|(a, b)| (a * b) % Q)
            .collect()
    }

    fn div_ntt(self, other: Self) -> Option<Self> {
        if other.coefficients.iter().any(|x| *x == 0) {
            return None;
        }
        let poly = self.coefficients.into_iter()
            .zip(other.coefficients.into_iter())
            .map(|(a, b)| (a * INV_MOD_Q[b as usize] as u32) % Q)
            .collect();

        Some(poly)
    }
}
