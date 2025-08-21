use crate::common::Polynomial;

include!(concat!(env!("OUT_DIR"), "/fft_constants.rs"));

pub trait FFT: Sized {
    fn split_fft(self) -> (Self, Self);
    fn merge_fft(self, other: Self) -> Self;
    fn fft(self) -> Self;
    fn ifft(self) -> Self;
    fn add(self, other: Self) -> Self;
    fn neg(self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn mul(self, other: Self) -> Self;
    fn div(self, other: Self) -> Self;
    fn adj(self) -> Self;
    fn add_fft(self, other: Self) -> Self;
    fn sub_fft(self, other: Self) -> Self;
    fn mul_fft(self, other: Self) -> Self;
    fn div_fft(self, other: Self) -> Self;
    fn adj_fft(self) -> Self;
}

impl FFT for Polynomial<Complex64> {
    fn split_fft(self) -> (Self, Self) {
        let n = self.len();
        let w = roots(n as u16);
        self.coefficients
            .chunks_exact(2)
            .enumerate()
            .map(|(i, chunks)| {
                let [even, odd] = chunks else { unreachable!() };
                let f0_fft = (even + odd) * 0.5;
                let f1_fft = (even - odd) * 0.5 * w[2 * i].conj();
                (f0_fft, f1_fft)
            })
            .unzip()
    }

    fn merge_fft(self, other: Self) -> Self {
        let n = 2 * self.len();
        let w = roots(n as u16);
        self.coefficients
            .iter()
            .enumerate()
            .zip(other.coefficients)
            .flat_map(|((i, a), b)| [a + w[2 * i] * b, a - w[2 * i] * b])
            .collect()
    }

    fn fft(self) -> Self {
        match self.coefficients.as_slice() {
            [f0, f1] => Polynomial {
                coefficients: vec![
                    f0 + Complex64::new(0., 1.) * f1,
                    f0 - Complex64::new(0., 1.) * f1,
                ]
            },
            _ => {
                let (f0, f1) = self.split();
                let f0_fft = f0.fft();
                let f1_fft = f1.fft();
                f0_fft.merge_fft(f1_fft)
            }
        }
    }

    fn ifft(self) -> Self {
        match self.coefficients.as_slice() {
            [f0_fft, _] => Polynomial {
                coefficients: vec![Complex64::new(f0_fft.re, 0.), Complex64::new(f0_fft.im, 0.)]
            },
            _ => {
                let (f0_fft, f1_fft) = self.split_fft();
                let f0 = f0_fft.ifft();
                let f1 = f1_fft.ifft();
                f0.merge(f1)
            }
        }
    }

    fn add(self, other: Self) -> Self {
        self.coefficients.into_iter()
            .zip(other.coefficients.into_iter())
            .map(|(a, b)| a + b)
            .collect()
    }

    fn neg(self) -> Self {
        self.coefficients.into_iter().map(|a| -a).collect()
    }

    fn sub(self, other: Self) -> Self {
        self.coefficients.into_iter()
            .zip(other.coefficients.into_iter())
            .map(|(a, b)| a - b)
            .collect()
    }

    fn mul(self, other: Self) -> Self {
        self.fft()
            .mul_fft(other.fft())
            .ifft()
    }

    fn div(self, other: Self) -> Self {
        self.fft()
            .div_fft(other.fft())
            .ifft()
    }

    fn adj(self) -> Self {
        self.fft().adj_fft().ifft()
    }

    fn add_fft(self, other: Self) -> Self {
        self.add(other)
    }

    fn sub_fft(self, other: Self) -> Self {
        self.sub(other)
    }

    fn mul_fft(self, other: Self) -> Self {
        self.coefficients.into_iter()
            .zip(other.coefficients.into_iter())
            .map(|(a, b)| a * b)
            .collect()
    }

    fn div_fft(self, other: Self) -> Self {
        self.coefficients.into_iter()
            .zip(other.coefficients.into_iter())
            .map(|(a, b)| a / b)
            .collect()
    }

    fn adj_fft(self) -> Self {
        self.coefficients.into_iter()
            .map(|x| x.conj())
            .collect()
    }
}
