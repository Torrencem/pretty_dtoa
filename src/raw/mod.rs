
use ryu::d2s;

#[inline]
pub fn dtoa(val: f64) -> (bool, Vec<u8>, i32) {
    let bits = val.to_bits();
    let sign = ((bits >> (d2s::DOUBLE_MANTISSA_BITS + d2s::DOUBLE_EXPONENT_BITS)) & 1) != 0;
    let ieee_mantissa = bits & ((1u64 << d2s::DOUBLE_MANTISSA_BITS) - 1);
    let ieee_exponent =
        (bits >> d2s::DOUBLE_MANTISSA_BITS) as u32 & ((1u32 << d2s::DOUBLE_EXPONENT_BITS) - 1);
    let as_decimal = d2s::d2d(ieee_mantissa, ieee_exponent);
    let as_digits: Vec<u8> = as_decimal
            .mantissa
            .to_string()
            .into_bytes()
            .into_iter()
            .map(|val| val - ('0' as u8))
            .collect();
    let exp = as_decimal.exponent + as_digits.len() as i32;
    (sign, as_digits, exp)
}

