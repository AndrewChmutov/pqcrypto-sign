#![allow(dead_code)]

use core::f64;
use core::f64::consts::LN_2;
use rand::{CryptoRng, Rng};

const ILN_2: f64 = 1f64 / f64::consts::LN_2;

fn base_sampler(random_bytes: [u8; 9]) -> u16 {
    const RCDT: [u128; 18] = [
        3024686241123004913666,
        1564742784480091954050,
        636254429462080897535,
        199560484645026482916,
        47667343854657281903,
        8595902006365044063,
        1163297957344668388,
        117656387352093658,
        8867391802663976,
        496969357462633,
        20680885154299,
        638331848991,
        14602316184,
        247426747,
        3104126,
        28824,
        198,
        1,
    ];
    let mut buffer = [0u8; 16];
    buffer[7..].copy_from_slice(&random_bytes);
    // rust implementation uses big endinan. Why?
    let u = u128::from_le_bytes(buffer);
    RCDT.iter().filter(|elt| u < **elt).count() as u16
}

fn compute_z(x: f64) -> u64 {
    f64::floor(x * (1u64 << 63) as f64) as u64
}

fn mul_shift(a: u64, b: u64) -> u64 {
    ((a as u128 * b as u128) >> 63) as u64
}

/// 2^63 * ccs * exp(-x)
fn approx_exp(x: f64, ccs: f64) -> u64 {
    const C: [u64; 13] = [
        0x00000004741183A3,
        0x00000036548CFC06,
        0x0000024FDCBF140A,
        0x0000171D939DE045,
        0x0000D00CF58F6F84,
        0x000680681CF796E3,
        0x002D82D8305B0FEA,
        0x011111110E066FD0,
        0x0555555555070F00,
        0x155555555581FF00,
        0x400000000002B400,
        0x7FFFFFFFFFFF4800,
        0x8000000000000000,
    ];

    let [mut y, o @ ..] = C;
    let mut z = compute_z(x);
    // wrapping_sub?
    y = o.iter().fold(y, |acc, elt| elt - mul_shift(z, acc));
    // rust implementation does not perform shift << 1. Why?
    z = compute_z(ccs) << 1;
    mul_shift(z, y)
}

/// Bernoulli distribution
fn ber_exp(x: f64, ccs: f64, random_bytes: [u8; 7]) -> bool {
    let mut s = f64::floor(x * ILN_2) as u64;
    let r = x - s as f64 * LN_2;
    s = s.min(63);
    // rust implementation performs additional shift << 1 after approx_exp. Why?
    let z = (approx_exp(r, ccs) - 1) >> s;

    let mut w = None;
    for (i, p) in (0u16..64u16).rev().step_by(8).zip(random_bytes) {
        match p as u64 - ((z >> i) & 0xFF) {
            0 => (),
            non_zero => {
                w = Some(non_zero);
                break;
            }
        }
    }

    w.unwrap() > 0
}

fn sampler_z<R>(mu: f64, sigma: f64, sigmin: f64, rng: &mut R) -> u16
where
    R: Rng + CryptoRng,
{
    const SIGMAX: f64 = 1.8205;
    const INV_2SIGMA2: f64 = 1f64 / ((SIGMAX * SIGMAX) * 2f64);
    let s = f64::floor(mu);
    let r = mu - s;
    let dss = 1f64 / (2f64 * sigma * sigma);
    let ccs = sigmin / sigma;

    loop {
        // Sampler z0 from a Half-Gaussian
        let z0 = base_sampler(rng.random());
        // Convert z0 into a pseudo-Gaussian sample z
        let b = (rng.random::<u8>() & 1) as u16;
        let z = b + (2 * b - 1) * z0;
        // Rejection sampling to obtain a true Gaussian sample
        let zr = z as f64 - r;
        let z0_f = z0 as f64;
        let x = zr * zr * dss - z0_f * z0_f * INV_2SIGMA2;
        if ber_exp(x, ccs, rng.random()) {
            return z + s as u16;
        }
    }
}
