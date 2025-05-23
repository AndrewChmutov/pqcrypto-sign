#![allow(dead_code)]

pub const Q: u16 = 12 * 1024 + 1;

type Coefficients<T> = Vec<T>;

pub(crate) struct Polynomial<T: Copy> {
    coefficients: Coefficients<T>,
}

impl<T: Copy> Polynomial<T> {
    fn new(coefficients: Coefficients<T>) -> Self {
        Self { coefficients }
    }

    fn len(&self) -> usize {
        self.coefficients.len()
    }

    fn split(self) -> (Self, Self) {
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

    fn merge(self, other: Self) -> Self {
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
}
