#![allow(dead_code)]

use std::hash::Hasher;

use crate::falcon::SALT_LEN;

use rs_shake256::{HasherContext, Shake256Hasher};

pub const Q: u32 = 12 * 1024 + 1;

type Coefficients<T> = Vec<T>;

pub(crate) struct Polynomial<T: Copy> {
    pub coefficients: Coefficients<T>,
}

impl<T: Copy> Polynomial<T> {
    fn new(coefficients: Coefficients<T>) -> Self {
        Self { coefficients }
    }

    pub fn len(&self) -> usize {
        self.coefficients.len()
    }

    pub fn split(self) -> (Self, Self) {
        let (ipairs_odd, ipairs_even): (Vec<_>, Vec<_>) = self
            .coefficients
            .into_iter()
            .enumerate()
            .partition(|(i, _)| i % 2 == 0);

        let extract_value = |(_, val)| val;
        let vec_odd = ipairs_odd.into_iter().map(extract_value).collect();
        let vec_even = ipairs_even.into_iter().map(extract_value).collect();

        (Self::new(vec_odd), Self::new(vec_even))
    }

    pub fn merge(self, other: Self) -> Self {
        let mut result = Vec::with_capacity(self.len() + other.len());
        let mut iter_odd = self.coefficients.into_iter();
        let mut iter_even = other.coefficients.into_iter();
        while let (Some(val_odd), Some(val_even)) = (iter_odd.next(), iter_even.next()) {
            result.push(val_odd);
            result.push(val_even);
        }
        result.extend(iter_odd);
        result.extend(iter_even);
        Self::new(result)
    }

    pub fn hash_to_point(message: &[u8], salt: &[u8; SALT_LEN], n: usize) -> Polynomial<u32> {
        const OUTPUT_SIZE: usize = 2;
        const K: u32 = (1u32 << 16) / Q;
        let mut shake = Shake256Hasher::<OUTPUT_SIZE>::default();
        shake.write(salt);
        shake.write(message);

        (0..n)
            .map(|_| {
                loop {
                    let byte_array_wrapper = <Shake256Hasher<OUTPUT_SIZE> as HasherContext<OUTPUT_SIZE>>::finish(&mut shake);
                    let byte_array = byte_array_wrapper.as_ref();
                    let elt = ((byte_array[0] as u32) << 8) | byte_array[1] as u32;
                    if elt < K * Q {
                        return elt % Q;
                    }
                }
            }).collect()
    }
}

impl<T: Copy> Default for Polynomial<T> {
    fn default() -> Self {
        Self {
            coefficients: Vec::default(),
        }
    }
}

impl<T: Copy> Extend<T> for Polynomial<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.coefficients.extend(iter);
    }
}

impl<T: Copy> FromIterator<T> for Polynomial<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            coefficients: Vec::from_iter(iter),
        }
    }
}

