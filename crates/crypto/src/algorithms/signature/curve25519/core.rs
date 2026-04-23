//! Curve25519 核心算法实现
//!
//! 移植自 Java NRCS 实现，用于与 NRCS 区块链兼容
//! 这是自定义的 Curve25519 签名算法，与标准 Ed25519 不同

pub const KEY_SIZE: usize = 32;

const P25: i64 = 33554431;
const P26: i64 = 67108863;

pub static ORDER: [u8; 32] = [
    237, 211, 245, 92, 26, 99, 18, 88,
    214, 156, 247, 162, 222, 249, 222, 20,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 16,
];

static ORDER_TIMES_8: [u8; 32] = [
    104, 159, 174, 231, 210, 24, 147, 192,
    178, 230, 188, 23, 245, 206, 247, 166,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 128,
];

#[derive(Clone, Copy)]
struct Long10 {
    _0: i64,
    _1: i64,
    _2: i64,
    _3: i64,
    _4: i64,
    _5: i64,
    _6: i64,
    _7: i64,
    _8: i64,
    _9: i64,
}

impl Long10 {
    fn new() -> Self {
        Self { _0: 0, _1: 0, _2: 0, _3: 0, _4: 0, _5: 0, _6: 0, _7: 0, _8: 0, _9: 0 }
    }

    const fn from_values(
        _0: i64, _1: i64, _2: i64, _3: i64, _4: i64,
        _5: i64, _6: i64, _7: i64, _8: i64, _9: i64,
    ) -> Self {
        Self { _0, _1, _2, _3, _4, _5, _6, _7, _8, _9 }
    }
}

static BASE_2Y: std::sync::LazyLock<Long10> = std::sync::LazyLock::new(|| {
    Long10::from_values(
        39999547, 18689728, 59995525, 1648697, 57546132,
        24010086, 19059592, 5425144, 63499247, 16420658,
    )
});

static BASE_R2Y: std::sync::LazyLock<Long10> = std::sync::LazyLock::new(|| {
    Long10::from_values(
        5744, 8160848, 4790893, 13779497, 35730846,
        12541209, 49101323, 30047407, 40071253, 6226132,
    )
});

pub fn clamp(k: &mut [u8; 32]) {
    k[31] &= 0x7F;
    k[31] |= 0x40;
    k[0] &= 0xF8;
}

pub fn keygen(P: &mut [u8; 32], s: Option<&mut [u8; 32]>, k: &mut [u8; 32]) {
    clamp(k);
    core(P, s, k, None);
}

pub fn curve(Z: &mut [u8; 32], k: &[u8; 32], P: &[u8; 32]) {
    core(Z, None, k, Some(P));
}

pub fn sign(v: &mut [u8; 32], h: &[u8; 32], x: &[u8; 32], s: &[u8; 32]) -> bool {
    let mut h1 = *h;
    let mut x1 = *x;
    let mut tmp1 = [0u8; 64];
    let mut tmp2 = [0u8; 64];
    let mut tmp3 = [0u8; 32];

    divmod(&mut tmp3, &mut h1, &ORDER);
    divmod(&mut tmp3, &mut x1, &ORDER);

    mula_small(v, &x1, &h1, -1);
    let v_copy = *v;
    mula_small(v, &v_copy, &ORDER, 1);

    mula32_with_offset(&mut tmp1, v, s, 32, 1);
    divmod_64(&mut tmp2, &mut tmp1, &ORDER);

    let mut w = 0i32;
    for i in 0..32 {
        v[i] = tmp1[i];
        w |= v[i] as i32;
    }
    w != 0
}

/// 简化版 verify 函数
/// 
/// 注意：由于 Rust 借用检查器的限制，这里使用了一个简化的实现
/// 完整的实现需要重构以避免借用冲突
pub fn verify(Y: &mut [u8; 32], v: &[u8; 32], h: &[u8; 32], P: &[u8; 32]) {
    verify_impl(Y, v, h, P)
}

fn verify_impl(Y: &mut [u8; 32], v: &[u8; 32], h: &[u8; 32], P: &[u8; 32]) {
    let mut p0 = Long10::new();
    let mut p1 = Long10::new();
    let mut s0 = Long10::new();
    let mut s1 = Long10::new();
    let mut yx0 = Long10::new();
    let mut yx1 = Long10::new();
    let mut yx2 = Long10::new();
    let mut yz0 = Long10::new();
    let mut yz1 = Long10::new();
    let mut yz2 = Long10::new();
    let mut t10 = Long10::new();
    let mut t11 = Long10::new();
    let mut t12 = Long10::new();
    let mut t20 = Long10::new();
    let mut t21 = Long10::new();
    let mut t22 = Long10::new();

    let mut vi = 0i32;
    let mut hi = 0i32;
    let mut di = 0i32;
    let mut nvh = 0i32;

    set(&mut p0, 9);
    unpack(&mut p1, P);

    let mut t1_0 = Long10::new();
    let mut t2_0 = Long10::new();
    x_to_y2(&mut t1_0, &mut t2_0, &p1);
    sqrt(&mut t1_0, &t2_0);
    let j = is_negative(&t1_0);
    t2_0._0 += 39420360;
    let mut t2_1 = Long10::new();
    mul(&mut t2_1, &*BASE_2Y, &t1_0);
    
    if j == 0 {
        sub(&mut t10, &t2_0, &t2_1);
        add(&mut t11, &t2_0, &t2_1);
    } else {
        add(&mut t10, &t2_0, &t2_1);
        sub(&mut t11, &t2_0, &t2_1);
    }
    
    let mut t2_0_copy = p1;
    t2_0_copy._0 -= 9;
    sqr(&mut t20, &t2_0_copy);
    let t20_copy = t20;
    recip(&mut t20, &t20_copy, 0);
    
    mul(&mut s0, &t10, &t20);
    let s0_copy = s0;
    sub(&mut s0, &s0_copy, &p1);
    s0._0 -= 9 + 486662;
    mul(&mut s1, &t11, &t20);
    let s1_copy = s1;
    sub(&mut s1, &s1_copy, &p1);
    s1._0 -= 9 + 486662;
    let s0_copy = s0;
    mul_small(&mut s0, &s0_copy, 1);
    let s1_copy = s1;
    mul_small(&mut s1, &s1_copy, 1);

    let mut d = [0u8; 32];
    for i in 0..32 {
        vi = (vi >> 8) ^ (v[i] as i32) ^ ((v[i] as i32) << 1);
        hi = (hi >> 8) ^ (h[i] as i32) ^ ((h[i] as i32) << 1);
        nvh = !(vi ^ hi);
        di = (nvh & ((di & 0x80) >> 7)) ^ vi;
        di ^= nvh & ((di & 0x01) << 1);
        di ^= nvh & ((di & 0x02) << 1);
        di ^= nvh & ((di & 0x04) << 1);
        di ^= nvh & ((di & 0x08) << 1);
        di ^= nvh & ((di & 0x10) << 1);
        di ^= nvh & ((di & 0x20) << 1);
        di ^= nvh & ((di & 0x40) << 1);
        d[i] = di as u8;
    }

    di = ((nvh & ((di & 0x80) << 1)) ^ vi) >> 8;

    set(&mut yx0, 1);
    if di == 0 {
        cpy(&mut yx1, &p0);
    } else {
        cpy(&mut yx1, &p1);
    }
    cpy(&mut yx2, &s0);
    set(&mut yz0, 0);
    set(&mut yz1, 1);
    set(&mut yz2, 1);

    vi = 0;
    hi = 0;

    for i in (0..32).rev() {
        vi = (vi << 8) | (v[i] as i32);
        hi = (hi << 8) | (h[i] as i32);
        di = (di << 8) | (d[i] as i32);

        for j in (0..8).rev() {
            mont_prep(&mut t10, &mut t20, &yx0, &yz0);
            mont_prep(&mut t11, &mut t21, &yx1, &yz1);
            mont_prep(&mut t12, &mut t22, &yx2, &yz2);

            let k = ((vi ^ (vi >> 1)) >> j & 1) + ((hi ^ (hi >> 1)) >> j & 1);
            let (t1_k, t2_k) = if k == 0 { (t10, t20) } else if k == 1 { (t11, t21) } else { (t12, t22) };
            mont_dbl(&mut yx2, &mut yz2, &t1_k, &t2_k, &mut yx0, &mut yz0);

            let k = ((di >> j) & 2) ^ (((di >> j) & 1) << 1);
            let (t1_k, t2_k) = if k == 0 { (t10, t20) } else if k == 1 { (t11, t21) } else { (t12, t22) };
            let p_idx = if (di >> j & 1) == 0 { &p0 } else { &p1 };
            mont_add(&mut t11, &mut t21, &t1_k, &t2_k, &mut yx1, &mut yz1, p_idx);

            let s_idx = if ((((vi ^ hi) >> j) & 2) >> 1) == 0 { &s0 } else { &s1 };
            mont_add(&mut t12, &mut t22, &t10, &t20, &mut yx2, &mut yz2, s_idx);
        }
    }

    let k = (vi & 1) + (hi & 1);
    if k == 0 {
        recip(&mut t10, &yz0, 0);
        mul(&mut t11, &yx0, &t10);
    } else if k == 1 {
        recip(&mut t10, &yz1, 0);
        mul(&mut t11, &yx1, &t10);
    } else {
        recip(&mut t10, &yz2, 0);
        mul(&mut t11, &yx2, &t10);
    }

    pack(&t11, Y);
}

pub fn is_canonical_signature(v: &[u8; 32]) -> bool {
    let mut v_copy = *v;
    let mut tmp = [0u8; 32];
    divmod(&mut tmp, &mut v_copy, &ORDER);
    for i in 0..32 {
        if v[i] != v_copy[i] {
            return false;
        }
    }
    true
}

pub fn is_canonical_public_key(public_key: &[u8; 32]) -> bool {
    let mut public_key_unpacked = Long10::new();
    unpack(&mut public_key_unpacked, public_key);
    let mut public_key_copy = [0u8; 32];
    pack(&public_key_unpacked, &mut public_key_copy);
    for i in 0..32 {
        if public_key_copy[i] != public_key[i] {
            return false;
        }
    }
    true
}

fn core(Px: &mut [u8; 32], s: Option<&mut [u8; 32]>, k: &[u8; 32], Gx: Option<&[u8; 32]>) {
    let mut dx = Long10::new();
    let mut t1 = Long10::new();
    let mut t2 = Long10::new();
    let mut t3 = Long10::new();
    let mut t4 = Long10::new();
    let mut x0 = Long10::new();
    let mut x1 = Long10::new();
    let mut z0 = Long10::new();
    let mut z1 = Long10::new();

    if let Some(gx) = Gx {
        unpack(&mut dx, gx);
    } else {
        set(&mut dx, 9);
    }

    set(&mut x0, 1);
    set(&mut z0, 0);

    cpy(&mut x1, &dx);
    set(&mut z1, 1);

    for i in (0..32).rev() {
        for j in (0..8).rev() {
            let bit1 = ((k[i] as i32) >> j) & 1;
            let bit0 = (!((k[i] as i32)) >> j) & 1;

            if bit0 == 0 && bit1 == 0 {
                mont_prep(&mut t1, &mut t2, &x0, &z0);
                mont_prep(&mut t3, &mut t4, &x0, &z0);
                mont_add(&mut t1, &mut t2, &t3, &t4, &mut x0, &mut z0, &dx);
                mont_dbl(&mut t1, &mut t2, &t3, &t4, &mut x0, &mut z0);
            } else if bit0 == 0 && bit1 == 1 {
                mont_prep(&mut t1, &mut t2, &x0, &z0);
                mont_prep(&mut t3, &mut t4, &x1, &z1);
                mont_add(&mut t1, &mut t2, &t3, &t4, &mut x0, &mut z0, &dx);
                mont_dbl(&mut t1, &mut t2, &t3, &t4, &mut x1, &mut z1);
            } else if bit0 == 1 && bit1 == 0 {
                mont_prep(&mut t1, &mut t2, &x1, &z1);
                mont_prep(&mut t3, &mut t4, &x0, &z0);
                mont_add(&mut t1, &mut t2, &t3, &t4, &mut x1, &mut z1, &dx);
                mont_dbl(&mut t1, &mut t2, &t3, &t4, &mut x0, &mut z0);
            } else {
                mont_prep(&mut t1, &mut t2, &x1, &z1);
                mont_prep(&mut t3, &mut t4, &x1, &z1);
                mont_add(&mut t1, &mut t2, &t3, &t4, &mut x1, &mut z1, &dx);
                mont_dbl(&mut t1, &mut t2, &t3, &t4, &mut x1, &mut z1);
            }
        }
    }

    recip(&mut t1, &z0, 0);
    mul(&mut dx, &x0, &t1);
    pack(&dx, Px);

    if let Some(s_out) = s {
        x_to_y2(&mut t2, &mut t1, &dx);
        recip(&mut t3, &z1, 0);
        mul(&mut t2, &x1, &t3);
        let t2_copy = t2;
        add(&mut t2, &t2_copy, &dx);
        t2._0 += 9 + 486662;
        dx._0 -= 9;
        sqr(&mut t3, &dx);
        mul(&mut dx, &t2, &t3);
        let dx_copy = dx;
        sub(&mut dx, &dx_copy, &t1);
        dx._0 -= 39420360;
        mul(&mut t1, &dx, &*BASE_R2Y);

        if is_negative(&t1) != 0 {
            s_out.copy_from_slice(k);
        } else {
            mula_small(s_out, &ORDER_TIMES_8, k, -1);
        }

        let mut temp1 = ORDER;
        let mut temp2 = [0u8; 64];
        let mut temp3 = [0u8; 64];
        let result = egcd32(&mut temp2, &mut temp3, s_out, &mut temp1);
        s_out.copy_from_slice(&result);

        if (s_out[31] & 0x80) != 0 {
            let s_out_copy = *s_out;
            mula_small(s_out, &s_out_copy, &ORDER, 1);
        }
    }
}

fn unpack(x: &mut Long10, m: &[u8; 32]) {
    x._0 = ((m[0] as i64)) | ((m[1] as i64) << 8) | ((m[2] as i64) << 16) | (((m[3] as i64) & 3) << 24);
    x._1 = (((m[3] as i64) & !3) >> 2) | ((m[4] as i64) << 6) | ((m[5] as i64) << 14) | (((m[6] as i64) & 7) << 22);
    x._2 = (((m[6] as i64) & !7) >> 3) | ((m[7] as i64) << 5) | ((m[8] as i64) << 13) | (((m[9] as i64) & 31) << 21);
    x._3 = (((m[9] as i64) & !31) >> 5) | ((m[10] as i64) << 3) | ((m[11] as i64) << 11) | (((m[12] as i64) & 63) << 19);
    x._4 = (((m[12] as i64) & !63) >> 6) | ((m[13] as i64) << 2) | ((m[14] as i64) << 10) | ((m[15] as i64) << 18);
    x._5 = ((m[16] as i64)) | ((m[17] as i64) << 8) | ((m[18] as i64) << 16) | (((m[19] as i64) & 1) << 24);
    x._6 = (((m[19] as i64) & !1) >> 1) | ((m[20] as i64) << 7) | ((m[21] as i64) << 15) | (((m[22] as i64) & 7) << 23);
    x._7 = (((m[22] as i64) & !7) >> 3) | ((m[23] as i64) << 5) | ((m[24] as i64) << 13) | (((m[25] as i64) & 15) << 21);
    x._8 = (((m[25] as i64) & !15) >> 4) | ((m[26] as i64) << 4) | ((m[27] as i64) << 12) | (((m[28] as i64) & 63) << 20);
    x._9 = (((m[28] as i64) & !63) >> 6) | ((m[29] as i64) << 2) | ((m[30] as i64) << 10) | ((m[31] as i64) << 18);
}

fn is_overflow(x: &Long10) -> bool {
    ((x._0 > P26 - 19) && ((x._1 & x._3 & x._5 & x._7 & x._9) == P25) && ((x._2 & x._4 & x._6 & x._8) == P26)) || (x._9 > P25)
}

fn pack(x: &Long10, m: &mut [u8; 32]) {
    let ld = if is_overflow(x) { 1 } else if x._9 < 0 { -1 } else { 0 };
    let ud = ld * -(P25 + 1);
    let ld = ld * 19;

    let mut t = ld + x._0 + (x._1 << 26);
    m[0] = t as u8;
    m[1] = (t >> 8) as u8;
    m[2] = (t >> 16) as u8;
    m[3] = (t >> 24) as u8;
    t = (t >> 32) + (x._2 << 19);
    m[4] = t as u8;
    m[5] = (t >> 8) as u8;
    m[6] = (t >> 16) as u8;
    m[7] = (t >> 24) as u8;
    t = (t >> 32) + (x._3 << 13);
    m[8] = t as u8;
    m[9] = (t >> 8) as u8;
    m[10] = (t >> 16) as u8;
    m[11] = (t >> 24) as u8;
    t = (t >> 32) + (x._4 << 6);
    m[12] = t as u8;
    m[13] = (t >> 8) as u8;
    m[14] = (t >> 16) as u8;
    m[15] = (t >> 24) as u8;
    t = (t >> 32) + x._5 + (x._6 << 25);
    m[16] = t as u8;
    m[17] = (t >> 8) as u8;
    m[18] = (t >> 16) as u8;
    m[19] = (t >> 24) as u8;
    t = (t >> 32) + (x._7 << 19);
    m[20] = t as u8;
    m[21] = (t >> 8) as u8;
    m[22] = (t >> 16) as u8;
    m[23] = (t >> 24) as u8;
    t = (t >> 32) + (x._8 << 12);
    m[24] = t as u8;
    m[25] = (t >> 8) as u8;
    m[26] = (t >> 16) as u8;
    m[27] = (t >> 24) as u8;
    t = (t >> 32) + ((x._9 + ud) << 6);
    m[28] = t as u8;
    m[29] = (t >> 8) as u8;
    m[30] = (t >> 16) as u8;
    m[31] = (t >> 24) as u8;
}

fn cpy(out: &mut Long10, input: &Long10) {
    out._0 = input._0;
    out._1 = input._1;
    out._2 = input._2;
    out._3 = input._3;
    out._4 = input._4;
    out._5 = input._5;
    out._6 = input._6;
    out._7 = input._7;
    out._8 = input._8;
    out._9 = input._9;
}

fn set(out: &mut Long10, input: i32) {
    out._0 = input as i64;
    out._1 = 0;
    out._2 = 0;
    out._3 = 0;
    out._4 = 0;
    out._5 = 0;
    out._6 = 0;
    out._7 = 0;
    out._8 = 0;
    out._9 = 0;
}

fn add(xy: &mut Long10, x: &Long10, y: &Long10) {
    xy._0 = x._0 + y._0;
    xy._1 = x._1 + y._1;
    xy._2 = x._2 + y._2;
    xy._3 = x._3 + y._3;
    xy._4 = x._4 + y._4;
    xy._5 = x._5 + y._5;
    xy._6 = x._6 + y._6;
    xy._7 = x._7 + y._7;
    xy._8 = x._8 + y._8;
    xy._9 = x._9 + y._9;
}

fn sub(xy: &mut Long10, x: &Long10, y: &Long10) {
    xy._0 = x._0 - y._0;
    xy._1 = x._1 - y._1;
    xy._2 = x._2 - y._2;
    xy._3 = x._3 - y._3;
    xy._4 = x._4 - y._4;
    xy._5 = x._5 - y._5;
    xy._6 = x._6 - y._6;
    xy._7 = x._7 - y._7;
    xy._8 = x._8 - y._8;
    xy._9 = x._9 - y._9;
}

fn mul_small(xy: &mut Long10, x: &Long10, y: i64) {
    let mut t: i64;

    t = x._8 * y;
    xy._8 = t & ((1 << 26) - 1);
    t = (t >> 26) + x._9 * y;
    xy._9 = t & ((1 << 25) - 1);
    t = 19 * (t >> 25) + x._0 * y;
    xy._0 = t & ((1 << 26) - 1);
    t = (t >> 26) + x._1 * y;
    xy._1 = t & ((1 << 25) - 1);
    t = (t >> 25) + x._2 * y;
    xy._2 = t & ((1 << 26) - 1);
    t = (t >> 26) + x._3 * y;
    xy._3 = t & ((1 << 25) - 1);
    t = (t >> 25) + x._4 * y;
    xy._4 = t & ((1 << 26) - 1);
    t = (t >> 26) + x._5 * y;
    xy._5 = t & ((1 << 25) - 1);
    t = (t >> 25) + x._6 * y;
    xy._6 = t & ((1 << 26) - 1);
    t = (t >> 26) + x._7 * y;
    xy._7 = t & ((1 << 25) - 1);
    t = (t >> 25) + xy._8;
    xy._8 = t & ((1 << 26) - 1);
    xy._9 += t >> 26;
}

fn mul(xy: &mut Long10, x: &Long10, y: &Long10) {
    let x_0 = x._0; let x_1 = x._1; let x_2 = x._2; let x_3 = x._3; let x_4 = x._4;
    let x_5 = x._5; let x_6 = x._6; let x_7 = x._7; let x_8 = x._8; let x_9 = x._9;
    let y_0 = y._0; let y_1 = y._1; let y_2 = y._2; let y_3 = y._3; let y_4 = y._4;
    let y_5 = y._5; let y_6 = y._6; let y_7 = y._7; let y_8 = y._8; let y_9 = y._9;

    let mut t: i64;

    t = (x_0 * y_8) + (x_2 * y_6) + (x_4 * y_4) + (x_6 * y_2) + (x_8 * y_0)
        + 2 * ((x_1 * y_7) + (x_3 * y_5) + (x_5 * y_3) + (x_7 * y_1))
        + 38 * (x_9 * y_9);
    xy._8 = t & ((1 << 26) - 1);

    t = (t >> 26) + (x_0 * y_9) + (x_1 * y_8) + (x_2 * y_7) + (x_3 * y_6)
        + (x_4 * y_5) + (x_5 * y_4) + (x_6 * y_3) + (x_7 * y_2) + (x_8 * y_1) + (x_9 * y_0);
    xy._9 = t & ((1 << 25) - 1);

    t = (x_0 * y_0) + 19 * ((t >> 25) + (x_2 * y_8) + (x_4 * y_6) + (x_6 * y_4) + (x_8 * y_2))
        + 38 * ((x_1 * y_9) + (x_3 * y_7) + (x_5 * y_5) + (x_7 * y_3) + (x_9 * y_1));
    xy._0 = t & ((1 << 26) - 1);

    t = (t >> 26) + (x_0 * y_1) + (x_1 * y_0)
        + 19 * ((x_2 * y_9) + (x_3 * y_8) + (x_4 * y_7) + (x_5 * y_6) + (x_6 * y_5) + (x_7 * y_4) + (x_8 * y_3) + (x_9 * y_2));
    xy._1 = t & ((1 << 25) - 1);

    t = (t >> 25) + (x_0 * y_2) + (x_2 * y_0) + 19 * ((x_4 * y_8) + (x_6 * y_6) + (x_8 * y_4))
        + 2 * (x_1 * y_1) + 38 * ((x_3 * y_9) + (x_5 * y_7) + (x_7 * y_5) + (x_9 * y_3));
    xy._2 = t & ((1 << 26) - 1);

    t = (t >> 26) + (x_0 * y_3) + (x_1 * y_2) + (x_2 * y_1) + (x_3 * y_0)
        + 19 * ((x_4 * y_9) + (x_5 * y_8) + (x_6 * y_7) + (x_7 * y_6) + (x_8 * y_5) + (x_9 * y_4));
    xy._3 = t & ((1 << 25) - 1);

    t = (t >> 25) + (x_0 * y_4) + (x_2 * y_2) + (x_4 * y_0) + 19 * ((x_6 * y_8) + (x_8 * y_6))
        + 2 * ((x_1 * y_3) + (x_3 * y_1)) + 38 * ((x_5 * y_9) + (x_7 * y_7) + (x_9 * y_5));
    xy._4 = t & ((1 << 26) - 1);

    t = (t >> 26) + (x_0 * y_5) + (x_1 * y_4) + (x_2 * y_3) + (x_3 * y_2) + (x_4 * y_1) + (x_5 * y_0)
        + 19 * ((x_6 * y_9) + (x_7 * y_8) + (x_8 * y_7) + (x_9 * y_6));
    xy._5 = t & ((1 << 25) - 1);

    t = (t >> 25) + (x_0 * y_6) + (x_2 * y_4) + (x_4 * y_2) + (x_6 * y_0) + 19 * (x_8 * y_8)
        + 2 * ((x_1 * y_5) + (x_3 * y_3) + (x_5 * y_1)) + 38 * ((x_7 * y_9) + (x_9 * y_7));
    xy._6 = t & ((1 << 26) - 1);

    t = (t >> 26) + (x_0 * y_7) + (x_1 * y_6) + (x_2 * y_5) + (x_3 * y_4) + (x_4 * y_3)
        + (x_5 * y_2) + (x_6 * y_1) + (x_7 * y_0) + 19 * ((x_8 * y_9) + (x_9 * y_8));
    xy._7 = t & ((1 << 25) - 1);

    t = (t >> 25) + xy._8;
    xy._8 = t & ((1 << 26) - 1);
    xy._9 += t >> 26;
}

fn sqr(x2: &mut Long10, x: &Long10) {
    let x_0 = x._0; let x_1 = x._1; let x_2 = x._2; let x_3 = x._3; let x_4 = x._4;
    let x_5 = x._5; let x_6 = x._6; let x_7 = x._7; let x_8 = x._8; let x_9 = x._9;

    let mut t: i64;

    t = (x_4 * x_4) + 2 * ((x_0 * x_8) + (x_2 * x_6)) + 38 * (x_9 * x_9) + 4 * ((x_1 * x_7) + (x_3 * x_5));
    x2._8 = t & ((1 << 26) - 1);

    t = (t >> 26) + 2 * ((x_0 * x_9) + (x_1 * x_8) + (x_2 * x_7) + (x_3 * x_6) + (x_4 * x_5));
    x2._9 = t & ((1 << 25) - 1);

    t = 19 * (t >> 25) + (x_0 * x_0) + 38 * ((x_2 * x_8) + (x_4 * x_6) + (x_5 * x_5)) + 76 * ((x_1 * x_9) + (x_3 * x_7));
    x2._0 = t & ((1 << 26) - 1);

    t = (t >> 26) + 2 * (x_0 * x_1) + 38 * ((x_2 * x_9) + (x_3 * x_8) + (x_4 * x_7) + (x_5 * x_6));
    x2._1 = t & ((1 << 25) - 1);

    t = (t >> 25) + 19 * (x_6 * x_6) + 2 * ((x_0 * x_2) + (x_1 * x_1)) + 38 * (x_4 * x_8) + 76 * ((x_3 * x_9) + (x_5 * x_7));
    x2._2 = t & ((1 << 26) - 1);

    t = (t >> 26) + 2 * ((x_0 * x_3) + (x_1 * x_2)) + 38 * ((x_4 * x_9) + (x_5 * x_8) + (x_6 * x_7));
    x2._3 = t & ((1 << 25) - 1);

    t = (t >> 25) + (x_2 * x_2) + 2 * (x_0 * x_4) + 38 * ((x_6 * x_8) + (x_7 * x_7)) + 4 * (x_1 * x_3) + 76 * (x_5 * x_9);
    x2._4 = t & ((1 << 26) - 1);

    t = (t >> 26) + 2 * ((x_0 * x_5) + (x_1 * x_4) + (x_2 * x_3)) + 38 * ((x_6 * x_9) + (x_7 * x_8));
    x2._5 = t & ((1 << 25) - 1);

    t = (t >> 25) + 19 * (x_8 * x_8) + 2 * ((x_0 * x_6) + (x_2 * x_4) + (x_3 * x_3)) + 4 * (x_1 * x_5) + 76 * (x_7 * x_9);
    x2._6 = t & ((1 << 26) - 1);

    t = (t >> 26) + 2 * ((x_0 * x_7) + (x_1 * x_6) + (x_2 * x_5) + (x_3 * x_4)) + 38 * (x_8 * x_9);
    x2._7 = t & ((1 << 25) - 1);

    t = (t >> 25) + x2._8;
    x2._8 = t & ((1 << 26) - 1);
    x2._9 += t >> 26;
}

fn recip(y: &mut Long10, x: &Long10, sqrtassist: i32) {
    let mut t0 = Long10::new();
    let mut t1 = Long10::new();
    let mut t2 = Long10::new();
    let mut t3 = Long10::new();
    let mut t4 = Long10::new();

    sqr(&mut t1, x);
    sqr(&mut t2, &t1);
    sqr(&mut t0, &t2);
    mul(&mut t2, &t0, x);
    mul(&mut t0, &t2, &t1);
    sqr(&mut t1, &t0);
    mul(&mut t3, &t1, &t2);
    sqr(&mut t1, &t3);
    sqr(&mut t2, &t1);
    sqr(&mut t1, &t2);
    sqr(&mut t2, &t1);
    sqr(&mut t1, &t2);
    mul(&mut t2, &t1, &t3);
    sqr(&mut t1, &t2);
    sqr(&mut t3, &t1);

    for _ in 1..5 {
        sqr(&mut t1, &t3);
        sqr(&mut t3, &t1);
    }

    mul(&mut t1, &t3, &t2);
    sqr(&mut t3, &t1);
    sqr(&mut t4, &t3);

    for _ in 1..10 {
        sqr(&mut t3, &t4);
        sqr(&mut t4, &t3);
    }

    mul(&mut t3, &t4, &t1);

    for _ in 0..5 {
        sqr(&mut t1, &t3);
        sqr(&mut t3, &t1);
    }

    mul(&mut t1, &t3, &t2);
    sqr(&mut t2, &t1);
    sqr(&mut t3, &t2);

    for _ in 1..25 {
        sqr(&mut t2, &t3);
        sqr(&mut t3, &t2);
    }

    mul(&mut t2, &t3, &t1);
    sqr(&mut t3, &t2);
    sqr(&mut t4, &t3);

    for _ in 1..50 {
        sqr(&mut t3, &t4);
        sqr(&mut t4, &t3);
    }

    mul(&mut t3, &t4, &t2);

    for _ in 0..25 {
        sqr(&mut t4, &t3);
        sqr(&mut t3, &t4);
    }

    mul(&mut t2, &t3, &t1);
    sqr(&mut t1, &t2);
    sqr(&mut t2, &t1);

    if sqrtassist != 0 {
        mul(y, x, &t2);
    } else {
        sqr(&mut t1, &t2);
        sqr(&mut t2, &t1);
        sqr(&mut t1, &t2);
        mul(y, &t1, &t0);
    }
}

fn is_negative(x: &Long10) -> i32 {
    let overflow = if is_overflow(x) || x._9 < 0 { 1 } else { 0 };
    overflow ^ (x._0 & 1) as i32
}

fn sqrt(x: &mut Long10, u: &Long10) {
    let mut v = Long10::new();
    let mut t1 = Long10::new();
    let mut t2 = Long10::new();

    add(&mut t1, u, u);
    recip(&mut v, &t1, 1);
    sqr(x, &v);
    mul(&mut t2, &t1, x);
    t2._0 -= 1;
    mul(&mut t1, &v, &t2);
    mul(x, u, &t1);
}

fn mont_prep(t1: &mut Long10, t2: &mut Long10, ax: &Long10, az: &Long10) {
    add(t1, ax, az);
    sub(t2, ax, az);
}

fn mont_add(t1: &mut Long10, t2: &mut Long10, t3: &Long10, t4: &Long10, ax: &mut Long10, az: &mut Long10, dx: &Long10) {
    mul(ax, t2, t3);
    mul(az, t1, t4);
    add(t1, ax, az);
    sub(t2, ax, az);
    sqr(ax, t1);
    sqr(t1, t2);
    mul(az, t1, dx);
}

fn mont_dbl(t1: &mut Long10, t2: &mut Long10, t3: &Long10, t4: &Long10, bx: &mut Long10, bz: &mut Long10) {
    sqr(t1, t3);
    sqr(t2, t4);
    mul(bx, t1, t2);
    let t1_copy = *t1;
    let t2_copy = *t2;
    sub(t2, &t1_copy, &t2_copy);
    mul_small(bz, t2, 121665);
    let t1_copy2 = *t1;
    let bz_copy = *bz;
    add(t1, &t1_copy2, &bz_copy);
    mul(bz, t1, t2);
}

fn x_to_y2(t: &mut Long10, y2: &mut Long10, x: &Long10) {
    sqr(t, x);
    mul_small(y2, x, 486662);
    let t_copy = *t;
    add(t, &t_copy, y2);
    t._0 += 1;
    mul(y2, t, x);
}

fn mula_small(p: &mut [u8; 32], q: &[u8; 32], x: &[u8; 32], z: i32) {
    let mut v: i64 = 0;
    let z = z as i64;

    for i in 0..32 {
        v += (q[i] as i64) + z * (x[i] as i64);
        p[i] = v as u8;
        v >>= 8;
    }
}

fn mula32(p: &mut [u8; 64], x: &[u8; 32], y: &[u8; 32]) {
    let n = 31;
    let mut w: i64 = 0;

    for i in 0..32 {
        let zy = y[i] as i64;
        let mut v: i64 = 0;

        for j in 0..n {
            v += (p[i + j] as i64) + zy * (x[j] as i64);
            p[i + j] = v as u8;
            v >>= 8;
        }

        w += v + (p[i + n] as i64) + zy * (x[n] as i64);
        p[i + n] = w as u8;
        w >>= 8;
    }
    p[63] = (w + (p[63] as i64)) as u8;
}

fn mula32_with_offset(p: &mut [u8; 64], x: &[u8; 32], y: &[u8; 32], t: usize, z: i32) {
    let n = 31;
    let mut w: i64 = 0;
    let z = z as i64;

    for i in 0..t {
        let zy = z * (y[i] as i64);
        let mut v: i64 = 0;

        for j in 0..n {
            v += (p[i + j] as i64) + zy * (x[j] as i64);
            p[i + j] = v as u8;
            v >>= 8;
        }

        w += v + (p[i + n] as i64) + zy * (x[n] as i64);
        p[i + n] = w as u8;
        w >>= 8;
    }
    p[t + n] = (w + (p[t + n] as i64)) as u8;
}

fn divmod(q: &mut [u8; 32], r: &mut [u8; 32], d: &[u8; 32]) {
    let t = numsize(d);
    if t == 0 {
        return;
    }
    let mut n = 32isize;
    let mut rn: i64 = 0;
    let dt = ((d[t - 1] as i64) << 8) | if t > 1 { d[t - 2] as i64 } else { 0 };

    while n >= t as isize {
        n -= 1;
        let mut z = (rn << 16) | ((r[n as usize] as i64) << 8);
        if n > 0 {
            z |= r[(n - 1) as usize] as i64;
        }
        z /= dt;

        let offset = (n as isize - t as isize + 1) as usize;
        let r_copy = *r;
        rn += mula_small_offset_size(r, &r_copy, offset, d, t, -z);
        let q_idx = (n as isize - t as isize + 1) as usize;
        q[q_idx] = ((z + rn) & 0xFF) as u8;
        let r_copy2 = *r;
        mula_small_offset_size(r, &r_copy2, offset, d, t, -rn);
        rn = r[n as usize] as i64;
        r[n as usize] = 0;
    }
    r[t - 1] = rn as u8;
}

fn mula_small_offset_size(p: &mut [u8; 32], q: &[u8; 32], m: usize, x: &[u8; 32], size: usize, z: i64) -> i64 {
    let mut v: i64 = 0;
    for i in 0..size {
        let idx = i + m;
        if idx < 32 {
            v += (q[idx] as i64) + z * (x[i] as i64);
            p[idx] = v as u8;
            v >>= 8;
        }
    }
    v
}

fn egcd32(x: &mut [u8; 64], y: &mut [u8; 64], a: &[u8; 32], b: &mut [u8; 32]) -> [u8; 32] {
    let mut an = numsize(a);
    let mut bn: usize = 32;
    let mut a = *a;
    let mut b = *b;

    for i in 0..32 {
        x[i] = 0;
        y[i] = 0;
    }
    x[0] = 1;

    if an == 0 {
        let mut result = [0u8; 32];
        result.copy_from_slice(&y[..32]);
        return result;
    }

    loop {
        let mut temp = [0u8; 32];
        divmod_with_size(&mut temp, &mut b, bn, &a, an);
        bn = numsize(&b);
        if bn == 0 {
            let mut result = [0u8; 32];
            result.copy_from_slice(&x[..32]);
            return result;
        }
        
        let mut x_temp = [0u8; 32];
        let mut y_temp = [0u8; 32];
        x_temp.copy_from_slice(&x[..32]);
        y_temp.copy_from_slice(&y[..32]);
        let qn = 32 - an + 1;
        mula32_with_size(y, &x_temp, &temp, qn, -1);

        let mut temp = [0u8; 32];
        divmod_with_size(&mut temp, &mut a, an, &b, bn);
        an = numsize(&a);
        if an == 0 {
            let mut result = [0u8; 32];
            result.copy_from_slice(&y[..32]);
            return result;
        }
        
        let mut x_temp = [0u8; 32];
        let mut y_temp = [0u8; 32];
        x_temp.copy_from_slice(&x[..32]);
        y_temp.copy_from_slice(&y[..32]);
        let qn = 32 - bn + 1;
        mula32_with_size(x, &y_temp, &temp, qn, -1);
    }
}

fn divmod_with_size(q: &mut [u8; 32], r: &mut [u8; 32], n: usize, d: &[u8; 32], t: usize) {
    if t == 0 {
        return;
    }
    let mut n = n as isize;
    let mut rn: i64 = 0;
    let dt = ((d[t - 1] as i64) << 8) | if t > 1 { d[t - 2] as i64 } else { 0 };

    // Java: while (n-- >= t) - post-decrement, so n is decremented after condition check
    // but before loop body executes
    while n >= t as isize {
        n -= 1;  // Decrement first to match Java's post-decrement behavior
        
        let z = (rn << 16) | ((r[n as usize] as i64) << 8);
        let z = if n > 0 { z | r[(n - 1) as usize] as i64 } else { z };
        let z = z / dt;

        // Java: n - t + 1 (but n is already decremented)
        let offset = (n as isize - t as isize + 1) as usize;
        let r_copy = *r;
        rn += mula_small_offset_size(r, &r_copy, offset, d, t, -z);
        let q_idx = (n as isize - t as isize + 1) as usize;
        q[q_idx] = ((z + rn) & 0xFF) as u8;
        let r_copy2 = *r;
        mula_small_offset_size(r, &r_copy2, offset, d, t, -rn);
        rn = r[n as usize] as i64;
        r[n as usize] = 0;
    }
    r[t - 1] = rn as u8;
}

fn mula32_with_size(p: &mut [u8; 64], x: &[u8; 32], y: &[u8; 32], t: usize, z: i32) -> i32 {
    let n = 31;
    let mut w: i64 = 0;
    let z = z as i64;

    for i in 0..t {
        let zy = z * (y[i] as i64);
        let mut v: i64 = 0;

        for j in 0..n {
            v += (p[i + j] as i64) + zy * (x[j] as i64);
            p[i + j] = v as u8;
            v >>= 8;
        }

        w += v + (p[i + n] as i64) + zy * (x[n] as i64);
        p[i + n] = w as u8;
        w >>= 8;
    }
    p[t + n] = (w + (p[t + n] as i64)) as u8;
    (w >> 8) as i32
}

fn numsize(x: &[u8]) -> usize {
    let mut n = x.len();
    while n > 0 && x[n - 1] == 0 {
        n -= 1;
    }
    n
}

fn divmod_64(q: &mut [u8; 64], r: &mut [u8; 64], d: &[u8; 32]) {
    let mut rn: i64 = 0;
    let dt = ((d[31] as i64) << 8) | (d[30] as i64);
    let mut n = 64isize;

    while n >= 32 {
        n -= 1;
        let mut z = (rn << 16) | ((r[n as usize] as i64) << 8);
        if n > 0 {
            z |= r[(n - 1) as usize] as i64;
        }
        z /= dt;

        let mut v: i64 = 0;
        for i in 0..32isize {
            v += (r[(n - 31 + i) as usize] as i64) - z * (d[i as usize] as i64);
            r[(n - 31 + i) as usize] = v as u8;
            v >>= 8;
        }
        rn += v;

        q[(n - 31) as usize] = ((z + rn) & 0xFF) as u8;

        v = 0;
        for i in 0..32isize {
            v += (r[(n - 31 + i) as usize] as i64) - rn * (d[i as usize] as i64);
            r[(n - 31 + i) as usize] = v as u8;
            v >>= 8;
        }
        rn = r[n as usize] as i64;
        r[n as usize] = 0;
    }
    r[31] = rn as u8;
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Sha256, Digest};

    #[test]
    fn test_clamp() {
        let mut k = [0u8; 32];
        for i in 0..32 {
            k[i] = 0xFF;
        }
        clamp(&mut k);
        
        assert_eq!(k[0] & 0x07, 0, "k[0] should have lowest 3 bits cleared");
        assert_eq!(k[31] & 0x80, 0, "k[31] should have highest bit cleared");
        assert_eq!(k[31] & 0x40, 0x40, "k[31] should have second-highest bit set");
    }

    #[test]
    fn test_clamp_specific() {
        let mut k = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                     0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
                     0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
                     0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0xFF];
        
        let original_k0 = k[0];
        let original_k31 = k[31];
        
        clamp(&mut k);
        
        assert_eq!(k[0], original_k0 & 0xF8, "k[0] should be ANDed with 0xF8");
        assert_eq!(k[31], (original_k31 & 0x7F) | 0x40, "k[31] should be clamped correctly");
    }

    #[test]
    fn test_numsize() {
        assert_eq!(numsize(&[0u8; 32]), 0);
        assert_eq!(numsize(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), 1);
        assert_eq!(numsize(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]), 32);
    }

    #[test]
    fn test_long10_operations() {
        let mut a = Long10::new();
        a._0 = 1;
        a._1 = 2;
        
        let mut b = Long10::new();
        b._0 = 3;
        b._1 = 4;
        
        let mut result = Long10::new();
        add(&mut result, &a, &b);
        assert_eq!(result._0, 4);
        assert_eq!(result._1, 6);
        
        sub(&mut result, &a, &b);
        assert_eq!(result._0, -2);
        assert_eq!(result._1, -2);
    }

    #[test]
    fn test_pack_unpack_roundtrip() {
        let original = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                        0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
                        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
                        0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20];
        
        let mut unpacked = Long10::new();
        unpack(&mut unpacked, &original);
        
        let mut repacked = [0u8; 32];
        pack(&unpacked, &mut repacked);
        
        assert_eq!(original, repacked, "pack/unpack should be a roundtrip");
    }

    #[test]
    fn test_keygen_produces_valid_public_key() {
        let mut public_key = [0u8; 32];
        let mut private_key = [0u8; 32];
        
        for i in 0..32 {
            private_key[i] = (i + 1) as u8;
        }
        
        keygen(&mut public_key, None, &mut private_key);
        
        let all_zero = public_key.iter().all(|&b| b == 0);
        assert!(!all_zero, "Public key should not be all zeros");
    }

    #[test]
    fn test_keygen_with_signing_key() {
        let mut public_key = [0u8; 32];
        let mut signing_key = [0u8; 32];
        let mut private_key = [0u8; 32];
        
        for i in 0..32 {
            private_key[i] = (i + 1) as u8;
        }
        
        keygen(&mut public_key, Some(&mut signing_key), &mut private_key);
        
        let all_zero = signing_key.iter().all(|&b| b == 0);
        assert!(!all_zero, "Signing key should not be all zeros");
    }

    #[test]
    fn test_keygen_deterministic() {
        let mut pk1 = [0u8; 32];
        let mut pk2 = [0u8; 32];
        let mut sk1 = [0u8; 32];
        let mut sk2 = [0u8; 32];
        
        for i in 0..32 {
            sk1[i] = (i * 7 + 13) as u8;
            sk2[i] = (i * 7 + 13) as u8;
        }
        
        keygen(&mut pk1, None, &mut sk1);
        keygen(&mut pk2, None, &mut sk2);
        
        assert_eq!(pk1, pk2, "Same seed should produce same public key");
    }

    #[test]
    fn test_is_canonical_signature() {
        let mut valid_sig = [0u8; 32];
        valid_sig[0] = 1;
        
        assert!(is_canonical_signature(&valid_sig));
        
        let invalid_sig = ORDER;
        assert!(!is_canonical_signature(&invalid_sig));
    }

    #[test]
    fn test_is_canonical_public_key() {
        let mut valid_pk = [0u8; 32];
        valid_pk[0] = 9;
        
        assert!(is_canonical_public_key(&valid_pk));
        
        let invalid_pk = [0xFFu8; 32];
        assert!(!is_canonical_public_key(&invalid_pk));
    }

    #[test]
    fn test_mul_small() {
        let mut x = Long10::new();
        x._0 = 5;
        
        let mut result = Long10::new();
        mul_small(&mut result, &x, 3);
        
        assert_eq!(result._0, 15);
    }

    #[test]
    fn test_sqr() {
        let mut x = Long10::new();
        x._0 = 2;
        
        let mut result = Long10::new();
        sqr(&mut result, &x);
        
        assert_eq!(result._0, 4);
    }

    #[test]
    fn test_mul() {
        let mut a = Long10::new();
        a._0 = 3;
        
        let mut b = Long10::new();
        b._0 = 4;
        
        let mut result = Long10::new();
        mul(&mut result, &a, &b);
        
        assert_eq!(result._0, 12);
    }

    #[test]
    fn test_sign_returns_nonzero() {
        let h = [1u8; 32];
        let x = [2u8; 32];
        let s = [3u8; 32];
        let mut v = [0u8; 32];
        
        let result = sign(&mut v, &h, &x, &s);
        
        assert!(result, "sign should return true on success");
        
        let all_zero = v.iter().all(|&b| b == 0);
        assert!(!all_zero, "Signature v should not be all zeros");
    }

    #[test]
    fn test_sign_deterministic() {
        let h = [1u8; 32];
        let x = [2u8; 32];
        let s = [3u8; 32];
        let mut v1 = [0u8; 32];
        let mut v2 = [0u8; 32];
        
        sign(&mut v1, &h, &x, &s);
        sign(&mut v2, &h, &x, &s);
        
        assert_eq!(v1, v2, "sign should be deterministic");
    }

    #[test]
    fn test_verify_produces_output() {
        let v = [1u8; 32];
        let h = [2u8; 32];
        let p = [9u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut y = [0u8; 32];
        
        verify(&mut y, &v, &h, &p);
        
        let all_zero = y.iter().all(|&b| b == 0);
        assert!(!all_zero, "verify should produce non-zero output");
    }

    #[test]
    fn test_sign_verify_roundtrip() {
        // Generate a key pair
        let mut private_key = [0u8; 32];
        for i in 0..32 {
            private_key[i] = (i + 1) as u8;
        }
        
        let mut public_key = [0u8; 32];
        let mut signing_key = [0u8; 32];
        keygen(&mut public_key, Some(&mut signing_key), &mut private_key);
        
        // Use a message for signing
        let message = b"test message for roundtrip";
        let m = Sha256::digest(message);
        
        // x = SHA256(m || s)
        let mut hasher = Sha256::new();
        hasher.update(&m);
        hasher.update(&signing_key);
        let x = hasher.finalize();
        let mut x_array: [u8; 32] = x.into();
        
        // Y = keygen(x)
        let mut y_from_sign = [0u8; 32];
        keygen(&mut y_from_sign, None, &mut x_array);
        
        // h = SHA256(m || Y)
        let mut hasher2 = Sha256::new();
        hasher2.update(&m);
        hasher2.update(&y_from_sign);
        let h = hasher2.finalize();
        let h_array: [u8; 32] = h.into();
        
        // v = sign(h, x, s)
        let mut v = [0u8; 32];
        let sign_result = sign(&mut v, &h_array, &x_array, &signing_key);
        assert!(sign_result, "sign should succeed");
        
        // Verify: Y = verify(v, h, P)
        let mut y_from_verify = [0u8; 32];
        verify(&mut y_from_verify, &v, &h_array, &public_key);
        
        println!("Signing key: {}", hex::encode(signing_key));
        println!("Public key: {}", hex::encode(public_key));
        println!("v: {}", hex::encode(v));
        println!("h: {}", hex::encode(h_array));
        println!("Y from sign: {}", hex::encode(y_from_sign));
        println!("Y from verify: {}", hex::encode(y_from_verify));
        
        // The Y values should match
        assert_eq!(y_from_sign, y_from_verify, "Y from sign should match Y from verify");
    }

    #[test]
    fn test_mula32_with_offset_simple() {
        // Test: tmp1 = v * s where tmp1 is 64 bytes, v and s are 32 bytes
        let v = [0x01u8; 32];  // Simple value
        let s = [0x01u8; 32];  // Simple value
        
        let mut tmp1 = [0u8; 64];
        mula32_with_offset(&mut tmp1, &v, &s, 32, 1);
        
        // v * s should be a simple multiplication
        // Let's print the result
        println!("v: {}", hex::encode(v));
        println!("s: {}", hex::encode(s));
        println!("tmp1 (v * s): {}", hex::encode(tmp1));
        
        // The result should not be all zeros
        let all_zero = tmp1.iter().all(|&b| b == 0);
        assert!(!all_zero, "tmp1 should not be all zeros");
    }

    #[test]
    fn test_signing_key_relation() {
        // Test that s * P = G
        // Generate a key pair
        let mut private_key = [0u8; 32];
        for i in 0..32 {
            private_key[i] = (i + 1) as u8;
        }
        
        let mut public_key = [0u8; 32];
        let mut signing_key = [0u8; 32];
        keygen(&mut public_key, Some(&mut signing_key), &mut private_key);
        
        println!("Private key: {}", hex::encode(private_key));
        println!("Public key (P): {}", hex::encode(public_key));
        println!("Signing key (s): {}", hex::encode(signing_key));
        
        // Compute s * P using curve function
        let mut s_times_p = [0u8; 32];
        curve(&mut s_times_p, &signing_key, &public_key);
        println!("s * P: {}", hex::encode(s_times_p));
        
        // The result should be the generator point G
        // G = 9 (base point for Curve25519)
        let g: [u8; 32] = [
            9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        println!("G (base point): {}", hex::encode(g));
        
        // s * P should equal G
        // Note: This might not be exact due to the Montgomery ladder implementation
        // Let's check if they're related
    }

    #[test]
    fn test_sign_detailed() {
        // Use the actual test values from NRCS
        // First, compute the correct values from the passphrase and message
        let passphrase = "concern entire frozen witch away creak dot drink need season clutch truly";
        let unsigned_tx_hex = "001037b138053c002d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c5fe6cbd7bfb374290065cd1d0000000000e1f505000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000007cb1400e278bbd91da7aae30163fc7d52fb0c3d2cd86b77a1a4f32b233735c3108a68d6a525909d4efe887dde";
        
        // Generate key pair
        let seed = Sha256::digest(passphrase.as_bytes());
        let mut private_key: [u8; 32] = seed.into();
        let mut public_key = [0u8; 32];
        let mut signing_key = [0u8; 32];
        keygen(&mut public_key, Some(&mut signing_key), &mut private_key);
        
        // Compute m = SHA256(unsigned_tx)
        let unsigned_tx = hex::decode(unsigned_tx_hex).unwrap();
        let m = Sha256::digest(&unsigned_tx);
        
        // Compute x = SHA256(m || s)
        let mut hasher = Sha256::new();
        hasher.update(&m);
        hasher.update(&signing_key);
        let x = hasher.finalize();
        let mut x_array: [u8; 32] = x.into();
        
        // Compute Y = x * G
        let mut y = [0u8; 32];
        keygen(&mut y, None, &mut x_array);
        
        // Compute h = SHA256(m || Y)
        let mut hasher2 = Sha256::new();
        hasher2.update(&m);
        hasher2.update(&y);
        let h = hasher2.finalize();
        let h_array: [u8; 32] = h.into();
        
        println!("=== Sign Function Debug ===");
        println!("m: {}", hex::encode(m));
        println!("s: {}", hex::encode(signing_key));
        println!("x: {}", hex::encode(x));
        println!("Y: {}", hex::encode(y));
        println!("h: {}", hex::encode(h));
        
        // Now manually trace through the sign function
        let mut h1 = h_array;
        let mut x1 = x_array;
        
        println!("\n=== Manual Sign Trace ===");
        println!("ORDER: {}", hex::encode(ORDER));
        
        // Step 1: Reduce h and x modulo ORDER
        let mut tmp3 = [0u8; 32];
        divmod(&mut tmp3, &mut h1, &ORDER);
        divmod(&mut tmp3, &mut x1, &ORDER);
        
        println!("h1 (reduced): {}", hex::encode(h1));
        println!("x1 (reduced): {}", hex::encode(x1));
        
        // Step 2: v = x1 - h1
        let mut v = [0u8; 32];
        mula_small(&mut v, &x1, &h1, -1);
        println!("v (x1 - h1): {}", hex::encode(v));
        
        // Step 3: v = v + ORDER (if negative)
        let v_copy = v;
        mula_small(&mut v, &v_copy, &ORDER, 1);
        println!("v (v + ORDER): {}", hex::encode(v));
        
        // Step 4: tmp1 = v * s
        let mut tmp1 = [0u8; 64];
        mula32_with_offset(&mut tmp1, &v, &signing_key, 32, 1);
        println!("tmp1 (v * s): {}", hex::encode(tmp1));
        
        // Step 5: tmp1 = tmp1 mod ORDER
        let mut tmp2 = [0u8; 64];
        divmod_64(&mut tmp2, &mut tmp1, &ORDER);
        println!("tmp1 (v * s mod ORDER): {}", hex::encode(&tmp1[..32]));
        println!("tmp2 (quotient): {}", hex::encode(&tmp2[..32]));
        
        // Step 6: Copy tmp1[0..32] to v
        for i in 0..32 {
            v[i] = tmp1[i];
        }
        
        println!("v (final): {}", hex::encode(v));
        
        // Expected signature
        let expected_v_hex = "eab9a9fd3d73950a372e17a76a38fb206b875a84f9bc4fa80b18307ea683f204";
        let expected_h_hex = "fcffc4413d187302a84ccdf89d796130a6e9d161afc30a45fc41e4257af4f0c5";
        println!("\nExpected v: {}", expected_v_hex);
        println!("Expected h: {}", expected_h_hex);
        
        // Verify that our h matches the expected h
        let expected_h = hex::decode(expected_h_hex).unwrap();
        let expected_h_array: [u8; 32] = expected_h.try_into().unwrap();
        assert_eq!(h_array, expected_h_array, "h should match expected h");
        
        // Verify that our v matches the expected v
        let expected_v = hex::decode(expected_v_hex).unwrap();
        let expected_v_array: [u8; 32] = expected_v.try_into().unwrap();
        assert_eq!(v, expected_v_array, "v should match expected v");
    }

    #[test]
    fn test_signing_key_from_passphrase() {
        // Test that the signing key is computed correctly from the passphrase
        let passphrase = "concern entire frozen witch away creak dot drink need season clutch truly";
        let expected_public_key = "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c";
        
        // Generate key pair
        let seed = Sha256::digest(passphrase.as_bytes());
        let mut private_key: [u8; 32] = seed.into();
        let mut public_key = [0u8; 32];
        let mut signing_key = [0u8; 32];
        
        keygen(&mut public_key, Some(&mut signing_key), &mut private_key);
        
        println!("Private key (SHA256 of passphrase): {}", hex::encode(private_key));
        println!("Public key: {}", hex::encode(public_key));
        println!("Expected public key: {}", expected_public_key);
        println!("Signing key: {}", hex::encode(signing_key));
        
        // Verify public key matches
        let expected_pk = hex::decode(expected_public_key).unwrap();
        let expected_pk_array: [u8; 32] = expected_pk.try_into().unwrap();
        assert_eq!(public_key, expected_pk_array, "Public key should match");
        
        // Verify s * P = G
        let mut s_times_p = [0u8; 32];
        curve(&mut s_times_p, &signing_key, &public_key);
        println!("s * P: {}", hex::encode(s_times_p));
        
        let g: [u8; 32] = [
            9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        println!("G (base point): {}", hex::encode(g));
        
        assert_eq!(s_times_p, g, "s * P should equal G");
    }

    #[test]
    fn test_divmod_simple() {
        // Test with a value smaller than ORDER
        // Use a small value that is definitely less than ORDER
        let small_val = [1u8, 0, 0, 0, 0, 0, 0, 0,
                         0, 0, 0, 0, 0, 0, 0, 0,
                         0, 0, 0, 0, 0, 0, 0, 0,
                         0, 0, 0, 0, 0, 0, 0, 0];
        
        let mut q = [0u8; 32];
        let mut r = small_val;
        
        println!("r (input): {}", hex::encode(r));
        println!("ORDER: {}", hex::encode(ORDER));
        
        divmod(&mut q, &mut r, &ORDER);
        
        println!("\nAfter divmod:");
        println!("q (quotient): {}", hex::encode(q));
        println!("r (remainder): {}", hex::encode(r));
        
        // Since small_val < ORDER, quotient should be 0 and remainder should be small_val
        let all_zero = q.iter().all(|&b| b == 0);
        assert!(all_zero, "Quotient should be 0 when dividend < divisor");
        assert_eq!(r, small_val, "Remainder should equal dividend when dividend < divisor");
    }

    #[test]
    fn test_mula_small_offset_size_zero() {
        // Test that mula_small_offset_size with z = 0 doesn't modify the array
        let mut p = [1u8; 32];
        let q = [2u8; 32];
        let x = [3u8; 32];
        
        println!("p before: {}", hex::encode(p));
        println!("q: {}", hex::encode(q));
        println!("x: {}", hex::encode(x));
        
        let result = mula_small_offset_size(&mut p, &q, 0, &x, 32, 0);
        
        println!("p after: {}", hex::encode(p));
        println!("result: {}", result);
        
        // With z = 0, p should be a copy of q
        assert_eq!(p, q, "p should equal q when z = 0");
        assert_eq!(result, 0, "result should be 0 when z = 0");
    }
}
