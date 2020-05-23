
mod lookup;

use std::f64;
use std::i16;
// use std::ops::Add;
use std::ops::Mul;

#[derive(Debug, Clone, Copy)]
enum Sign {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy)]
struct DecomposedFloat {
    sign: Sign,
    significand: u64,
    exponent: i16,
}

fn decompose(val: f64) -> DecomposedFloat {
    let bits: u64 = val.to_bits();
    let sign = if bits >> 63 == 0 { Sign::Positive } else { Sign::Negative };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
    let significand = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };

    exponent -= 1023 + 52;

    DecomposedFloat {
        sign, significand, exponent
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct HP {
    left: f64,
    right: f64,
}

// impl Add<f64> for HP {
//     type Output = HP;
//
//     fn add(self, y: f64) -> Self::Output {
//         let zb = self.left + y;
//         let c = (zb - self.left) - y;
//         let zd = self.right - c;
//         HP { left: zb, right: zd }
//     }
// }

fn split(val: f64) -> (f64, f64) {
    let hi = val.to_bits();
    let hi = hi & 0xFFFFFFFFF8000000;
    let hi = f64::from_bits(hi);
    let lo = val - hi;
    (hi, lo)
}

fn fpnext(val: f64) -> f64 {
    let val = val.to_bits() + 1;
    f64::from_bits(val)
}

fn fpprev(val: f64) -> f64 {
    let val = val.to_bits() - 1;
    f64::from_bits(val)
}

impl Mul<f64> for HP {
    type Output = HP;
    
    fn mul(self, rhs: f64) -> Self::Output {
        let (hi, lo) = split(self.left);
        let (hi2, lo2) = split(rhs);
        let p = self.right * rhs;
        let e = ((hi * hi2 - p) + lo * hi2 + hi * lo2) + lo * lo2;
        HP { left: p, right: self.right * rhs + e }
    }
}

fn mul_by_10(hp: &mut HP) {
    let val = hp.left;

    hp.left *= 10.0;
    hp.right *= 10.0;

    let mut off = hp.left;
    off -= val * 8.0;
    off -= val * 2.0;

    hp.right -= off;

    normalize(hp);
}

fn div_by_10(hp: &mut HP) {
    let mut val = hp.left;

    hp.left /= 10.0;
    hp.right /= 10.0;
    
    val -= hp.left * 8.0;
    val -= hp.left * 2.0;

    hp.right += val / 10.0;

    normalize(hp);
}

static ERROL1_EPSILON: f64 = 8.77e-15;
// static ERROL1_EPSILON: f64 = f64::EPSILON;

fn normalize(val: &mut HP) {
    let old = val.left;
    val.left += val.right;
    val.right += old - val.left;
}

// Algorithm based on https://github.com/marcandrysco/Errol/blob/master/lib/errol.c
pub fn dtoa(val: f64) -> (Vec<u8>, i32) {

    if val == 0.0 {
        return (vec![0], 0);
    }

    if val == f64::MAX {
        return (vec![1,7,9,7,6,9,3,1,3,4,8,6,2,3,1,5,7], 309);
    }
    
    let decomp = decompose(val);
    let e = decomp.exponent;

    // let exp = 307.0 + (e as f64) * 0.30103;
    let exp = e as f64;

    let mut exp = if exp < 20.0 {
        20i32
    } else if exp >= 600.0 {
        599i32
    } else {
        exp as i32
    };

    let mid = lookup::LOOKUP_TABLE[exp as usize];
    let mut mid = mid * val;
    let lten = lookup::LOOKUP_TABLE[exp as usize].left;
    let mut ten = 1.0;

    exp -= 307;

    while (mid.left > 10.0) || ((mid.left == 10.0) && (mid.right >= 0.0)) {
        exp += 1;
        div_by_10(&mut mid);
        ten /= 10.0;
    }

    while (mid.left < 1.0) || ((mid.left == 1.0) && (mid.right < 0.0)) {
        exp -= 1;
        mul_by_10(&mut mid);
        ten *= 10.0;
    }
    
    let mut inhi  = HP { left: mid.left, right: mid.right + (fpnext(val) - val) * lten * ten / (2.0 + ERROL1_EPSILON) };
    let mut inlo  = HP { left: mid.left, right: mid.right + (fpprev(val) - val) * lten * ten / (2.0 + ERROL1_EPSILON) };
    let mut outhi = HP { left: mid.left, right: mid.right + (fpnext(val) - val) * lten * ten / (2.0 - ERROL1_EPSILON) };
    let mut outlo = HP { left: mid.left, right: mid.right + (fpprev(val) - val) * lten * ten / (2.0 - ERROL1_EPSILON) };

    normalize(&mut inhi);
    normalize(&mut inlo);
    normalize(&mut outhi);
    normalize(&mut outlo);

    while inhi.left > 10.0 || (inhi.left == 10.0 && inhi.right >= 0.0) {
        exp += 1;
		div_by_10(&mut inhi);
        div_by_10(&mut inlo);
        div_by_10(&mut outhi);
        div_by_10(&mut outlo);
    }

    while inhi.left < 1.0 || (inhi.left == 1.0 && inhi.right < 0.0) {
        exp -= 1;
		mul_by_10(&mut inhi);
        mul_by_10(&mut inlo);
        mul_by_10(&mut outhi);
        mul_by_10(&mut outlo);
    }

    let mut res = vec![];

    while inhi.left != 0.0 && inhi.right != 0.0 {
        let mut hdig = inhi.left as u8;
        if (inhi.left == hdig as f64) && (inhi.right < 0.0) {
            hdig -= 1;
        }
        
        let mut ldig = inlo.left as u8;
        if (inlo.left == ldig as f64) && (inlo.right < 0.0) {
            ldig -= 1;
        }

        if ldig != hdig {
            break;
        }

        res.push(hdig);

        inhi.left -= hdig as f64;
        inlo.left -= ldig as f64;
        mul_by_10(&mut inhi);
        mul_by_10(&mut inlo);
        

        let mut hdig = outhi.left as u8;
        if (outhi.left == hdig as f64) && (outhi.right < 0.0) {
            hdig -= 1;
        }
        
        let mut ldig = outlo.left as u8;
        if (outlo.left == ldig as f64) && (outlo.right < 0.0) {
            ldig -= 1;
        }

        if ldig != hdig {
            // Opt = false
            // println!("Opt = false");
        }

        outhi.left -= hdig as f64;
        outlo.left -= ldig as f64;
        mul_by_10(&mut outhi);
        mul_by_10(&mut outlo);
    }

    let mdig = (inhi.left + inlo.left) / 2.0 + 0.5;
    res.push(mdig as u8);

    (res, exp)
}

