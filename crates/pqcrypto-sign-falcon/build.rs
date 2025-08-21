use std::{env, ffi::OsStr, fs, path::Path};

use modular_math::mod_math::ModMath;
use num_complex::Complex64;
use primitive_types::U256;

const Q: u16 = 12 * 1024 + 1;

fn ffi_expand_roots(roots: &[Complex64]) -> Vec<Complex64> {
    let mut result = Vec::with_capacity(roots.len() * 2);
    for &root in roots {
        let sqrt_root = Complex64::from_polar(root.norm().sqrt(), root.arg() / 2.0);
        result.push(sqrt_root);
        result.push(-sqrt_root);
    }
    result
}

fn fft_constants(n: usize) -> Vec<Vec<Complex64>> {
    let mut roots = Vec::with_capacity(n + 1);
    roots.push(vec![Complex64::new(0.0, 1.0), Complex64::new(0.0, -1.0)]);
    for i in 1..=n {
        roots.push(ffi_expand_roots(&roots[i - 1]));
    }
    roots
}

#[allow(non_snake_case)]
fn ntt_solve<T: Into<U256>>(root: T, Zq: &ModMath) -> Option<[U256; 2]> {
    let sqrt_root = Zq.sqrt(root.into())?;
    Some([sqrt_root, Zq.sub(0.into(), sqrt_root)])
}

#[allow(non_snake_case)]
fn ntt_expander(Zq: &ModMath) -> impl Fn(&[U256]) -> Vec<U256> + '_ {
    |roots: &[U256]| {
        let mut result = Vec::with_capacity(roots.len() * 2);
        for &root in roots {
            result.extend(ntt_solve(root, Zq).unwrap())
        }
        result
    }
}

fn from_primitive<T: TryFrom<U256> + std::fmt::Debug>(values: &[U256]) -> Vec<T> {
    values
        .iter()
        .copied()
        .map(TryFrom::try_from)
        .map(|x| x.unwrap_or_else(|_| unreachable!()))
        .collect()
}

#[allow(non_snake_case)]
fn ntt_constants(n: usize) -> (Vec<Vec<u16>>, Vec<u16>) {
    let Zq = ModMath::new(Q as u32);

    let expand = ntt_expander(&Zq);
    let mut roots = Vec::with_capacity(n + 1);
    let mut temp_root = ntt_solve(Zq.sub(0, 1), &Zq).unwrap().to_vec();
    roots.push(from_primitive(&temp_root));
    for _ in 1..=n {
        temp_root = expand(&temp_root);
        roots.push(from_primitive(&temp_root));
    }

    let inverses = std::iter::once(0u16)
        .chain((1..(Q as u32)).map(|i| TryInto::try_into(Zq.inv(i).unwrap()).unwrap()))
        .collect();

    (roots, inverses)
}

fn save_fft_constants(out_dir: &OsStr) {
    let dest_path = Path::new(&out_dir).join("fft_constants.rs");
    let roots = fft_constants(9);

    let align_out = " ".repeat(8);
    let align_in = " ".repeat(12);
    let arms = roots
        .into_iter()
        .map(|v| {
            format!(
                "{case} => &[\n{align_in}{slice}\n{align_out}],",
                case = v.len(),
                slice = v
                    .into_iter()
                    .map(|c| format!("Complex64 {{ re: {}f64, im: {}f64 }},", c.re, c.im))
                    .collect::<Vec<_>>()
                    .join(&format!("\n{}", align_in))
            )
        })
        .collect::<Vec<_>>()
        .join(&format!("\n{align_out}"));

    let content = format!(
        "\
use num_complex::Complex64;

const pub fn roots(n: u16) -> &'static [Complex64] {{
    match n {{
        {arms}
        _ => unreachable!(),
    }}
}}"
    );
    fs::write(&dest_path, content).unwrap();
}

fn save_ntt_constants(out_dir: &OsStr) {
    let dest_path = Path::new(&out_dir).join("ntt_constants.rs");
    let (roots, inverses) = ntt_constants(9);

    let align = " ".repeat(8);
    let arms = roots
        .into_iter()
        .map(|v| format!("{} => &{v:?},", v.len()))
        .collect::<Vec<_>>()
        .join(&format!("\n{align}"));

    let function = format!(
        "\
pub fn roots_Zq(n: u16) -> &'static [u16] {{
    match n {{
        {arms}
        _ => unreachable!(),
    }}
}}"
    );

    let inverses = format!("const INV_MOD_Q: [u16; {}] = {inverses:?};", inverses.len());

    fs::write(
        &dest_path,
        format!(
            "\
{function}

{inverses}"
        ),
    )
    .unwrap();
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    save_fft_constants(&out_dir);
    save_ntt_constants(&out_dir);
    println!("cargo::rerun-if-changed=build.rs");
}
