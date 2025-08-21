use bit_vec::BitVec;

pub trait Compression {
    fn compress(v: &[i16], slen: usize) -> Option<Vec<u8>>;
}

pub trait Decompression {
    fn decompress(x: &[u8], slen: usize, n: usize) -> Option<Vec<i16>>;
}

struct NaiveCompression;
impl Compression for NaiveCompression {
    fn compress(v: &[i16], slen: usize) -> Option<Vec<u8>> {
        let mut u = v
            .iter()
            .copied()
            .flat_map(|coef| {
                let s = coef.abs();
                let sign = coef < 0;
                let low_bits = (0..7).rev().map(move |i| ((s >> i) & 1) != 0);
                let high_bits = std::iter::repeat_n(false, (s as usize) >> 7);
                let terminator = true;
                std::iter::once(sign)
                    .chain(low_bits)
                    .chain(high_bits)
                    .chain(std::iter::once(terminator))
            })
            .collect::<BitVec>();

        if u.len() > 8 * slen {
            return None;
        }

        u.extend(std::iter::repeat_n(false, 8 * slen));

        Some(u.to_bytes())
    }
}

struct NaiveDecompression;
impl Decompression for NaiveDecompression {
    fn decompress(x: &[u8], slen: usize, n: usize) -> Option<Vec<i16>> {
        if x.len() > slen {
            return None;
        }
        let u = BitVec::from_bytes(x);

        let mut index = 0;
        let mut v = Vec::with_capacity(slen);

        for _ in 0..n {
            if index + 8 >= u.len() {
                return None;
            };

            let sign = if u[index] { -1 } else { 0 };
            let low_bits = ((index + 1)..(index + 8))
                .rev()
                .enumerate()
                .map(|(i, index)| (v[index] as i16) << i)
                .reduce(std::ops::BitOr::bitor)
                .unwrap();
            let high_bits = ((index + 8)..).take_while(|i| u[*i]).count() as i16;

            v.push(sign * ((high_bits << 7) | low_bits));
            index += 8;
        }

        if v.len() != n {
            return None;
        }

        Some(v)
    }
}
