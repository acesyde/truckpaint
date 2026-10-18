//! CityHash64 as the game uses it (CityHash v1.0.x); newer releases hash some lengths differently.

const K0: u64 = 0xc3a5c85c97cb3127;
const K1: u64 = 0xb492b66fbe98f273;
const K2: u64 = 0x9ae16a3b2f90404f;
const K3: u64 = 0xc949d7c7509e6557;

fn fetch64(s: &[u8]) -> u64 {
    u64::from_le_bytes(s[..8].try_into().unwrap())
}

fn fetch32(s: &[u8]) -> u64 {
    u64::from(u32::from_le_bytes(s[..4].try_into().unwrap()))
}

fn shift_mix(v: u64) -> u64 {
    v ^ (v >> 47)
}

fn hash_len16(u: u64, v: u64) -> u64 {
    const MUL: u64 = 0x9ddfea08eb382d69;
    let a = (u ^ v).wrapping_mul(MUL);
    let a = a ^ (a >> 47);
    let b = (v ^ a).wrapping_mul(MUL);
    (b ^ (b >> 47)).wrapping_mul(MUL)
}

fn hash_len0_to16(s: &[u8]) -> u64 {
    let len = s.len();
    if len > 8 {
        let a = fetch64(s);
        let b = fetch64(&s[len - 8..]);
        return hash_len16(a, b.wrapping_add(len as u64).rotate_right(len as u32)) ^ b;
    }
    if len >= 4 {
        let a = fetch32(s);
        return hash_len16((len as u64).wrapping_add(a << 3), fetch32(&s[len - 4..]));
    }
    if len > 0 {
        let a = u32::from(s[0]);
        let b = u32::from(s[len >> 1]);
        let c = u32::from(s[len - 1]);
        let y = a.wrapping_add(b << 8);
        let z = (len as u32).wrapping_add(c << 2);
        return shift_mix(u64::from(y).wrapping_mul(K2) ^ u64::from(z).wrapping_mul(K3))
            .wrapping_mul(K2);
    }
    K2
}

fn hash_len17_to32(s: &[u8]) -> u64 {
    let len = s.len();
    let a = fetch64(s).wrapping_mul(K1);
    let b = fetch64(&s[8..]);
    let c = fetch64(&s[len - 8..]).wrapping_mul(K2);
    let d = fetch64(&s[len - 16..]).wrapping_mul(K0);
    hash_len16(
        a.wrapping_sub(b)
            .rotate_right(43)
            .wrapping_add(c.rotate_right(30))
            .wrapping_add(d),
        a.wrapping_add((b ^ K3).rotate_right(20))
            .wrapping_sub(c)
            .wrapping_add(len as u64),
    )
}

fn weak_hash32(w: u64, x: u64, y: u64, z: u64, a: u64, b: u64) -> (u64, u64) {
    let a = a.wrapping_add(w);
    let b = b.wrapping_add(a).wrapping_add(z).rotate_right(21);
    let c = a;
    let a = a.wrapping_add(x).wrapping_add(y);
    let b = b.wrapping_add(a.rotate_right(44));
    (a.wrapping_add(z), b.wrapping_add(c))
}

fn weak_hash32_at(s: &[u8], a: u64, b: u64) -> (u64, u64) {
    weak_hash32(
        fetch64(s),
        fetch64(&s[8..]),
        fetch64(&s[16..]),
        fetch64(&s[24..]),
        a,
        b,
    )
}

fn hash_len33_to64(s: &[u8]) -> u64 {
    let len = s.len() as u64;
    let n = s.len();
    let mut z = fetch64(&s[24..]);
    let mut a = fetch64(s).wrapping_add(len.wrapping_add(fetch64(&s[n - 16..])).wrapping_mul(K0));
    let mut b = a.wrapping_add(z).rotate_right(52);
    let mut c = a.rotate_right(37);
    a = a.wrapping_add(fetch64(&s[8..]));
    c = c.wrapping_add(a.rotate_right(7));
    a = a.wrapping_add(fetch64(&s[16..]));
    let vf = a.wrapping_add(z);
    let vs = b.wrapping_add(a.rotate_right(31)).wrapping_add(c);
    a = fetch64(&s[16..]).wrapping_add(fetch64(&s[n - 32..]));
    z = fetch64(&s[n - 8..]);
    b = a.wrapping_add(z).rotate_right(52);
    c = a.rotate_right(37);
    a = a.wrapping_add(fetch64(&s[n - 24..]));
    c = c.wrapping_add(a.rotate_right(7));
    a = a.wrapping_add(fetch64(&s[n - 16..]));
    let wf = a.wrapping_add(z);
    let ws = b.wrapping_add(a.rotate_right(31)).wrapping_add(c);
    let r = shift_mix(
        vf.wrapping_add(ws)
            .wrapping_mul(K2)
            .wrapping_add(wf.wrapping_add(vs).wrapping_mul(K0)),
    );
    shift_mix(r.wrapping_mul(K0).wrapping_add(vs)).wrapping_mul(K2)
}

pub fn city_hash64(s: &[u8]) -> u64 {
    let len = s.len();
    if len <= 16 {
        return hash_len0_to16(s);
    }
    if len <= 32 {
        return hash_len17_to32(s);
    }
    if len <= 64 {
        return hash_len33_to64(s);
    }
    let mut x = fetch64(&s[len - 40..]);
    let mut y = fetch64(&s[len - 16..]).wrapping_add(fetch64(&s[len - 56..]));
    let mut z = hash_len16(
        fetch64(&s[len - 48..]).wrapping_add(len as u64),
        fetch64(&s[len - 24..]),
    );
    let mut v = weak_hash32_at(&s[len - 64..], len as u64, z);
    let mut w = weak_hash32_at(&s[len - 32..], y.wrapping_add(K1), x);
    x = x.wrapping_mul(K1).wrapping_add(fetch64(s));
    // Whole 64-byte chunks, the last one included only if it is complete.
    let mut rest = s;
    for _ in 0..(len - 1) / 64 {
        x = x
            .wrapping_add(y)
            .wrapping_add(v.0)
            .wrapping_add(fetch64(&rest[8..]))
            .rotate_right(37)
            .wrapping_mul(K1);
        y = y
            .wrapping_add(v.1)
            .wrapping_add(fetch64(&rest[48..]))
            .rotate_right(42)
            .wrapping_mul(K1);
        x ^= w.1;
        y = y.wrapping_add(v.0).wrapping_add(fetch64(&rest[40..]));
        z = z.wrapping_add(w.0).rotate_right(33).wrapping_mul(K1);
        v = weak_hash32_at(rest, v.1.wrapping_mul(K1), x.wrapping_add(w.0));
        w = weak_hash32_at(
            &rest[32..],
            z.wrapping_add(w.1),
            y.wrapping_add(fetch64(&rest[16..])),
        );
        std::mem::swap(&mut z, &mut x);
        rest = &rest[64..];
    }
    hash_len16(
        hash_len16(v.0, w.0)
            .wrapping_add(shift_mix(y).wrapping_mul(K1))
            .wrapping_add(z),
        hash_len16(v.1, w.1).wrapping_add(x),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Computed with the CityHash64 of ConverterPIX (`libs/cityhash`) on bytes `i * 7 + 3`.
    const VECTORS: [(usize, u64); 19] = [
        (0, 0x9ae16a3b2f90404f),
        (1, 0x5068a5b3d87a0284),
        (3, 0xf6aa543ca4b8bf14),
        (4, 0x9dd33b80b9fa6393),
        (7, 0x0e1bc1022a1b992c),
        (8, 0x37fed2dba3a300e3),
        (9, 0x29b1ff616f22d6f6),
        (16, 0xbdf5bcbc9b4603a4),
        (17, 0xa75fe55f0d775eda),
        (24, 0x7353f72904323f52),
        (32, 0xd7989f47d40c660d),
        (33, 0xef04fa08e8e5b8fb),
        (48, 0xa7b0eb0c7882e506),
        (64, 0xc1515868c51fc399),
        (65, 0xacf6a69f1f078342),
        (100, 0x90d13909657dcf12),
        (128, 0x899e1a09acbba491),
        (129, 0x511fb0cf56a6e561),
        (200, 0x6099c264b552b496),
    ];

    #[test]
    fn matches_the_game_hash_on_every_length_class() {
        for (len, expected) in VECTORS {
            let bytes = (0..len).map(|i| (i * 7 + 3) as u8).collect::<Vec<_>>();
            assert_eq!(city_hash64(&bytes), expected, "length {len}");
        }
    }
}
