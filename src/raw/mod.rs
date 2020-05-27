//! Functions for converting floats and doubles into decimal floats (radix 10)

use ryu::d2s;
use ryu::f2s;

#[inline]
pub fn dtod(val: f64) -> (bool, String, i32) {
    let bits = val.to_bits();
    let sign = ((bits >> (d2s::DOUBLE_MANTISSA_BITS + d2s::DOUBLE_EXPONENT_BITS)) & 1) != 0;
    let ieee_mantissa = bits & ((1u64 << d2s::DOUBLE_MANTISSA_BITS) - 1);
    let ieee_exponent =
        (bits >> d2s::DOUBLE_MANTISSA_BITS) as u32 & ((1u32 << d2s::DOUBLE_EXPONENT_BITS) - 1);
    let as_decimal = d2s::d2d(ieee_mantissa, ieee_exponent);
    let as_digits: String = as_decimal
            .mantissa
            .to_string();
    let exp = as_decimal.exponent + as_digits.len() as i32;
    (sign, as_digits, exp)
}

#[inline]
pub fn ftod(val: f32) -> (bool, String, i32) {
    let bits = val.to_bits();
    let sign = ((bits >> (f2s::FLOAT_MANTISSA_BITS + f2s::FLOAT_EXPONENT_BITS)) & 1) != 0;
    let ieee_mantissa = bits & ((1u32 << f2s::FLOAT_MANTISSA_BITS) - 1);
    let ieee_exponent =
        (bits >> f2s::FLOAT_MANTISSA_BITS) as u32 & ((1u32 << f2s::FLOAT_EXPONENT_BITS) - 1);
    let as_decimal = f2s::f2d(ieee_mantissa, ieee_exponent);
    let as_digits: String = as_decimal
            .mantissa
            .to_string();
    let exp = as_decimal.exponent + as_digits.len() as i32;
    (sign, as_digits, exp)
}
