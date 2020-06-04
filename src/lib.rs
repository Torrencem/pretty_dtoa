//! Print floats with many options:
//!
//! ```
//! use pretty_dtoa::{dtoa, FmtFloatConfig};
//!
//! let config = FmtFloatConfig::default()
//!       .max_decimal_digits(-3)   // cut off at 3 decimals left of the decimal point
//!       .truncate()               // don't round
//!       .force_no_e_notation()    // don't use exponential notation
//!       .add_point_zero(true);    // add a .0 to the end of integer values
//!
//! assert_eq!(dtoa(123123.0, config), "123000.0");
//! assert_eq!(dtoa(99999.0, config), "99000.0");
//! ```

// Testing macros, to make sure edge cases are hit

#[cfg(not(test))]
macro_rules! hit {
    ($ident:ident) => {}
}

// Mark a code condition as hit
#[cfg(test)]
macro_rules! hit {
    ($ident:ident) => {{
        extern "C" {
            #[no_mangle]
            static $ident: $crate::__rt::AtomicUsize;
        }
        unsafe {
            $ident.fetch_add(1, $crate::__rt::Ordering::Relaxed);
        }
    }};
}

#[cfg(test)]
mod __rt {
    pub use std::sync::atomic::{AtomicUsize, Ordering};

    pub struct Guard {
        mark: &'static AtomicUsize,
        name: &'static str,
        value_on_entry: usize,
    }

    impl Guard {
        pub fn new(mark: &'static AtomicUsize, name: &'static str) -> Guard {
            let value_on_entry = mark.load(Ordering::Relaxed);
            Guard { mark, name, value_on_entry }
        }
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            if std::thread::panicking() {
                return;
            }
            let value_on_exit = self.mark.load(Ordering::Relaxed);
            assert!(
                value_on_exit > self.value_on_entry,
                format!("mark was not hit: {}", self.name)
            )
        }
    }
}

use ryu_floating_decimal::{f2d, d2d};
use std::char;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum RoundMode {
    Round,
    Truncate,
}

/// Configuration for formatting floats into strings
///
/// # Example
///
/// ```
/// use pretty_dtoa::{dtoa, FmtFloatConfig};
///
/// let config = FmtFloatConfig::default()
///     .round()
///     .max_significant_digits(5);
///
/// assert_eq!(dtoa(123.4567, config), "123.46");
/// ```
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct FmtFloatConfig {
    /// A max number of significant digits to include
    /// in the formatted string (after the first non-zero digit).
    /// None means include all digits
    pub max_sig_digits: Option<u8>,
    /// A min number of significant digits to include
    /// in the formatted string (after the first non-zero digit).
    /// None means no minimum
    pub min_sig_digits: Option<u8>,
    /// A max number of digits after the decimal point to include.
    /// Overrides any significant digit rules
    pub max_decimal_digits: Option<i8>,
    /// A min number of digits after the decimal point to include.
    /// Overrides any significant digit rules
    pub min_decimal_digits: Option<i8>,
    /// How many digits left of the decimal point there can be
    /// using scientific notation
    pub upper_e_break: i8,
    /// Lower equivelent of upper_e_break
    pub lower_e_break: i8,
    /// Ignore digits after (and including) a certain number of
    /// consecutive 9's or 0's
    pub ignore_extremes: Option<u8>,
    /// Round or truncate
    pub round_mode: RoundMode,
    /// Force scientific e notation
    pub force_e_notation: bool,
    /// Force no scientific e notation. Overrides force_e_notation
    pub force_no_e_notation: bool,
    /// Capitalize the e in scientific notation
    pub capitalize_e: bool,
    /// Add a .0 at the end of integers
    pub add_point_zero: bool,
    /// The maximum number of characters in the string. This
    /// should be greater than or equal to 7 to guarantee all floats
    /// will print correctly, but can be smaller for certain floats
    pub max_width: Option<u8>,
    /// The seperator between the integer and non-integer part
    pub radix_point: char,
}

impl FmtFloatConfig {
    
    /// A default configuration. This will always round-trip, so
    /// using ``str::parse::<f64>`` or ``str::parse:<f32>`` will
    /// give the exact same float.
    pub const fn default() -> Self {
        FmtFloatConfig {
            max_sig_digits: None,
            min_sig_digits: None,
            max_decimal_digits: None,
            min_decimal_digits: None,
            upper_e_break: 4,
            lower_e_break: -4,
            ignore_extremes: None,
            round_mode: RoundMode::Round,
            force_e_notation: false,
            force_no_e_notation: false,
            capitalize_e: false,
            add_point_zero: true,
            max_width: None,
            radix_point: '.',
        }
    }

    /// The maximum number of non-zero digits to include in the string
    pub const fn max_significant_digits(mut self, val: u8) -> Self {
        self.max_sig_digits = Some(val);
        self
    }
    
    /// The minimum number of non-zero digits to include in the string
    pub const fn min_significant_digits(mut self, val: u8) -> Self {
        self.min_sig_digits = Some(val);
        self
    }

    /// The maximum number of digits past the decimal point to include in the string
    pub const fn max_decimal_digits(mut self, val: i8) -> Self {
        self.max_decimal_digits = Some(val);
        self
    }
    
    /// The minimum number of digits past the decimal point to include in the string
    pub const fn min_decimal_digits(mut self, val: i8) -> Self {
        self.min_decimal_digits = Some(val);
        self
    }
    
    /// The upper exponent value that will force using exponent notation
    /// (default: 4)
    pub const fn upper_e_break(mut self, val: i8) -> Self {
        self.upper_e_break = val;
        self
    }

    /// The lower exponent value that will force using exponent notation
    /// (default: -4)
    pub const fn lower_e_break(mut self, val: i8) -> Self {
        self.lower_e_break = val;
        self
    }

    /// Ignore digits after and including a certain number of
    /// consecutive 9's or 0's. This is useful for printing
    /// numbers with floating point errors to humans, even
    /// if the numbers are technically slightly adjusted.
    /// (example: 3.5999951 -> 3.6)
    pub const fn ignore_extremes(mut self, limit: u8) -> Self {
        self.ignore_extremes = Some(limit);
        self
    }
    
    /// When cutting off after a certain number of
    /// significant digits, ignore any further digits.
    /// Opposite of ``round(self)``.
    pub const fn truncate(mut self) -> Self {
        self.round_mode = RoundMode::Truncate;
        self
    }
    
    /// When cutting off after a certain number of
    /// significant digits / decimal digits, read
    /// the next digit and round up / down. This is 
    /// the default, but it doesn't matter in the
    /// default config, since no rounding happens.
    pub const fn round(mut self) -> Self {
        self.round_mode = RoundMode::Round;
        self
    }

    /// Force all floats to be in scientific notation.
    /// (example: 31 -> 3.1e1)
    pub const fn force_e_notation(mut self) -> Self {
        self.force_e_notation = true;
        self.force_no_e_notation = false;
        self
    }
    
    /// Force all floats to not be in scientific notation.
    /// (example: 3e10 -> 30000000000)
    pub const fn force_no_e_notation(mut self) -> Self {
        self.force_no_e_notation = true;
        self.force_e_notation = false;
        self
    }
    
    /// Capitalize the e in e notation.
    /// (example: 3.1e10 -> 3.1E10)
    /// (default: false)
    pub const fn capitalize_e(mut self, val: bool) -> Self {
        self.capitalize_e = val;
        self
    }
    
    /// Add a ".0" at the end of integers.
    /// (example: 31 -> 31.0)
    /// (default: true)
    pub const fn add_point_zero(mut self, val: bool) -> Self {
        self.add_point_zero = val;
        self
    }
    
    /// The maximum width of all the characters in the string. This
    /// should be greater than or equal to 7 to guarantee all floats
    /// will print correctly, but can be smaller for certain floats
    pub const fn max_width(mut self, val: u8) -> Self {
        self.max_width = Some(val);
        self
    }
    
    /// Allows any width of strings. This is set by default
    pub const fn no_max_width(mut self) -> Self {
        self.max_width = None;
        self
    }
    
    /// The seperator between the integer and non-integer part
    /// of the float string
    /// (default: `'.'`)
    pub const fn radix_point(mut self, val: char) -> Self {
        self.radix_point = val;
        self
    }
}

const fn digit_to_u8(val: u8) -> u8 {
    val + '0' as u8
}

fn digits_to_a(sign: bool, mut digits: Vec<u8>, mut e: i32, config: FmtFloatConfig) -> String {
    // The main string formatting function. digits is a vector of the digits
    // found using the ryu backend function. The value of the float is
    // <- if sign>0.<digits> * 10^<e>
    // NOTE: digits is ascii, so the digit "5" would be represented as "digit_to_u8(5)"
    if let Some(limit) = config.max_sig_digits {
        // Remove extra significant digits
        let limit = limit as usize;
        if digits.len() > limit {
            let removed = digits.drain(limit..).next().unwrap();
            if config.round_mode == RoundMode::Round && removed >= digit_to_u8(5) {
                // round up
                let mut l = digits.len() - 1;
                digits[l] += 1;
                while digits[l] == digit_to_u8(10) {
                    if l == 0 {
                        digits[0] = digit_to_u8(1);
                        e += 1;
                        break;
                    }
                    digits.pop();
                    l -= 1;
                    digits[l] += 1;
                }
            }
        }
    }
    if let Some(limit) = config.max_decimal_digits {
        // Remove extra decimal digits
        let adjusted_limit_position = limit as i32 + e;
        if (0 <= adjusted_limit_position) && (adjusted_limit_position < digits.len() as i32) {
            let final_char = digits.drain(adjusted_limit_position as usize ..).nth(0).unwrap();
            if config.round_mode == RoundMode::Round && final_char >= digit_to_u8(5) {
                // round up
                let mut l = digits.len() - 1;
                digits[l] += 1;
                while digits[l] == digit_to_u8(10) {
                    if l == 0 {
                        digits[0] = digit_to_u8(1);
                        e += 1;
                        break;
                    }
                    digits.pop();
                    l -= 1;
                    digits[l] += 1;
                }
            }
        }
    }
    if let Some(limit) = config.ignore_extremes {
        // Ignore <limit> consecutive 9's or 0's. A copy of digits is made
        let mut stripped_string: Vec<u8> = Vec::with_capacity(30);
        let mut nine_counter = 0;
        let mut zero_counter = 0;
        for digit in digits.iter() {
            if *digit != digit_to_u8(9) {
                nine_counter = 0;
            } else {
                nine_counter += 1;
                if nine_counter >= limit {
                    // 14999...
                    stripped_string.drain((stripped_string.len() + 1 - nine_counter as usize)..);
                    // -> 14
                    let l = stripped_string.len();
                    if l == 0 {
                        // for strings like 999
                        // 999e3 -> 1e4
                        stripped_string.push(digit_to_u8(1));
                        e += 1;
                    } else {
                        // Rounding doesn't have to happen here, because what was removed
                        // was exactly limit 9's
                        // 15
                        stripped_string[l - 1] += 1;
                    }
                    break;
                }
            }

            if *digit != digit_to_u8(0) {
                zero_counter = 0;
            } else {
                zero_counter += 1;
                if zero_counter >= limit {
                    // 14000...
                    stripped_string.drain((stripped_string.len() + 1 - zero_counter as usize)..);
                    // 14
                    break;
                }
            }
            stripped_string.push(*digit);
        }
        digits = stripped_string;
    }
    if let Some(limit) = config.min_sig_digits {
        // Pad 0's to get enough significant digits
        let mut curr = digits.len() as u8;
        while curr < limit {
            digits.push(digit_to_u8(0));
            curr += 1;
        }
    }
    if let Some(limit) = config.min_decimal_digits {
        // Pad 0's to get enough decimal digits
        let adjusted_limit_position = limit as i32 + e;
        while (digits.len() as i32) < adjusted_limit_position {
            digits.push(digit_to_u8(0));
        }
    }
    let mut use_e_notation = (e > config.upper_e_break as i32 || e <= config.lower_e_break as i32 || config.force_e_notation) && !config.force_no_e_notation;
    if let Some(max_width) = config.max_width {
        // Check if it is needed to force using e notation for max width
        let max_width = if sign { max_width - 1 } else { max_width };
        // Is it impossible to represent the value without e notation?
        if e > 0 && e + if config.add_point_zero { 2 } else { 0 } > max_width as i32 {
            hit!(e_width_case_a);
            use_e_notation = true;
        } else if -e + 3 > max_width as i32 {
            hit!(e_width_case_b);
            use_e_notation = true;
        } else if !use_e_notation {
            hit!(e_width_case_c);
            // Otherwise, prepare to not use e notation
            let is_integer = e > digits.len() as i32;
            let extra_length = if config.add_point_zero && is_integer { 2 } else { 0 }
                             + if !is_integer && !(e > 0 && e as u8 == max_width) { 1 } else { 0 }
                             + if e > 0 && digits.len() < e as usize { e - digits.len() as i32 } else { 0 }
                             + if e <= 0 { -e + 1 } else { 0 };
            let total_length = digits.len() + extra_length as usize;
            if total_length > max_width as usize {
                let final_char = digits.drain((max_width as usize - extra_length as usize)..).nth(0).unwrap();
                if config.round_mode == RoundMode::Round && final_char >= digit_to_u8(5) {
                    // round up
                    let mut l = digits.len() - 1;
                    digits[l] += 1;
                    while digits[l] == digit_to_u8(10) {
                        if l == 0 {
                            digits[0] = digit_to_u8(1);
                            e += 1;
                            break;
                        }
                        digits.pop();
                        l -= 1;
                        digits[l] += 1;
                    }
                }
            }
        }
    }
    // Final formatting stage
    if use_e_notation {
        if let Some(max_width) = config.max_width {
            let mut tail_as_str: String = digits.drain(1..).map(|val| val as char).collect();
            let e_length = format!("{}", e - 1).len();
            let extra_length = 3 + e_length + if sign { 1 } else { 0 };
            if extra_length >= max_width as usize {
                tail_as_str.drain(..);
            } else { 
                tail_as_str.truncate(max_width as usize - extra_length);
            }
            // Very special case: can't include a decimal point
            // within max_width
            if tail_as_str.len() == 0 && max_width == 7 && sign {
                return format!("-{}{}{}",
                               digits[0] as char,
                               if config.capitalize_e { "E" } else { "e" },
                               e - 1);
            }
            // Defer to the generic e-notation case
            for c in tail_as_str.chars() {
                digits.push(c as u8);
            }
        } 
        // Generic e-notation case
        let mut res = String::with_capacity(digits.len() + 5);
        if sign {
            res.push('-');
        }
        res.push(digits[0] as char);
        res.push(config.radix_point);
        if digits.len() == 1 {
            if !config.max_width.is_some() {
                res.push('0');
            }
        } else {
            for c in &digits[1..] {
                res.push(*c as char);
            }
        }
        if config.capitalize_e {
            res.push('E');
        } else {
            res.push('e');
        }
        res.push_str(format!("{}", e - 1).as_ref());
        return res;
    }
    // Non-e-notation case
    let mut as_str = String::with_capacity(digits.len() + 3);
    if sign {
        as_str.push('-');
    }
    let mut curr = 0;
    if e <= 0 {
        as_str.push('0');
        as_str.push(config.radix_point);
        for _ in 0..-e {
            as_str.push('0');
        }
    }
    for digit in digits {
        if e > 0 && curr == e {
            as_str.push(config.radix_point);
        }
        as_str.push(digit as char);
        curr += 1;
    }
    let is_integer = curr <= e;
    while e > 0 && curr < e {
        as_str.push('0');
        curr += 1;
    }
    if is_integer && config.add_point_zero {
        as_str.push(config.radix_point);
        as_str.push('0');
    }

    as_str
}

/// Convert a double-precision floating point value (``f64``) to a string
/// using a given configuration
///
/// # Example
/// 
/// ```
/// use pretty_dtoa::{dtoa, FmtFloatConfig};
///
/// let config = FmtFloatConfig::default()
///     .force_no_e_notation()      // Don't use scientific notation
///     .add_point_zero(true)       // Add .0 to the end of integers
///     .max_significant_digits(4)  // Stop after the first 4 non-zero digits
///     .radix_point(',')           // Use a ',' instead of a '.'
///     .round();                   // Round after removing non-significant digits
///
/// assert_eq!(dtoa(12459000.0, config), "12460000,0");
/// ```
pub fn dtoa(value: f64, config: FmtFloatConfig) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    } else if value.is_infinite() {
        return "inf".to_string();
    }
    let rad_10 = d2d(value);
    let sign = value.is_sign_negative();
    let s = format!("{}", rad_10.mantissa);
    let exp = rad_10.exponent + s.len()as i32;
    digits_to_a(sign, s.into_bytes(), exp, config)
}

/// Convert a single-precision floating point value (``f32``) to a string
/// using a given configuration
pub fn ftoa(value: f32, config: FmtFloatConfig) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    } else if value.is_infinite() {
        if value.is_sign_positive() {
            return "inf".to_string();
        } else {
            return "-inf".to_string();
        }
    }
    let rad_10 = f2d(value);
    let sign = value.is_sign_negative();
    let s = format!("{}", rad_10.mantissa);
    let exp = rad_10.exponent + s.len()as i32;
    digits_to_a(sign, s.into_bytes(), exp, config)
}

#[cfg(test)]
mod tests {
    // Macro for checking coverage marks
    macro_rules! check {
        ($ident:ident) => {
            #[no_mangle]
            static $ident: $crate::__rt::AtomicUsize =
                $crate::__rt::AtomicUsize::new(0);
            let _guard = $crate::__rt::Guard::new(&$ident, stringify!($ident));
        }
    }

    use super::*;

    use rand;
    use rand::Rng;

    #[test]
    fn test_widths() {
        // Test random floats with several configurations, and
        // make sure that .max_width(_) does its job
        let mut rng = rand::thread_rng();

        let configs = &[
            FmtFloatConfig::default(),
            FmtFloatConfig::default()
                .force_no_e_notation(),
            FmtFloatConfig::default()
                .add_point_zero(true),
            FmtFloatConfig::default()
                .truncate()
                .force_e_notation()
                .min_significant_digits(5),
        ];
        
        for width in 6..=13 {
            for i in 0..20000 {
                let config = configs[i % configs.len()]
                    .max_width(width);
                let val = f64::from_bits(rng.gen::<u64>());
                if val > 0.0 && val <= 9e-99 && width == 6 {
                    // unrepresentable
                    continue;
                }
                if val < 0.0 && val >= -9e-9 && width == 6 {
                    // unrepresentable
                    continue;
                }
                if val <= -1e100 && width == 6 {
                    // unrepresentable
                    continue;
                }
                if val.is_nan() {
                    continue;
                }
                let as_string = dtoa(val, config);
                assert!(as_string.len() <= width as usize, "Found bad example for string width: '{}' at width {} gives {}", val, width, as_string);
            }
        }
    }

    #[test]
    fn test_round_trip_dtoa() {
        // Make sure random floats with several configurations
        // round trip perfectly
        let mut rng = rand::thread_rng();
        
        let configs = &[
            FmtFloatConfig::default(),
            FmtFloatConfig::default()
                .force_no_e_notation()
                .add_point_zero(true),
            FmtFloatConfig::default()
                .force_e_notation(),
        ];
        for _ in 0..20000 {
            for config in configs.iter().cloned() {
                let val = f64::from_bits(rng.gen::<u64>());
                if val.is_nan() {
                    continue;
                }
                let as_string = dtoa(val, config);
                let round = as_string.parse::<f64>().unwrap();
                assert!(round == val, "Found bad example for round trip: value '{}' gives string '{}' which turns into value '{}'", val, as_string, round);
            }
        }
    }
    
    #[test]
    fn test_round_trip_ftoa() {
        // Test round tripping for random f32's
        let mut rng = rand::thread_rng();
        
        let configs = &[
            FmtFloatConfig::default(),
            FmtFloatConfig::default()
                .force_no_e_notation()
                .add_point_zero(true),
            FmtFloatConfig::default()
                .force_e_notation(),
        ];
        for _ in 0..20000 {
            for config in configs.iter().cloned() {
                let val = f32::from_bits(rng.gen::<u32>());
                if val.is_nan() {
                    continue;
                }
                let as_string = ftoa(val, config);
                let round = as_string.parse::<f32>().unwrap();
                assert!(round == val, "Found bad example for round trip: value '{}' gives string '{}' which turns into value '{}'", val, as_string, round);
            }
        }
    }

    #[test]
    fn test_max_sig_digits() {
        let config = FmtFloatConfig::default()
            .round()
            .max_significant_digits(5);
        assert_eq!(dtoa(3.111111, config), "3.1111");
        assert_eq!(dtoa(123.4567, config), "123.46");
        assert_eq!(dtoa(0.0001234567, config), "0.00012346");
        assert_eq!(dtoa(22.29999, config), "22.3");
        assert_eq!(dtoa(4.1, config), "4.1");
        let config = FmtFloatConfig::default()
            .truncate()
            .max_significant_digits(4);
        assert_eq!(dtoa(3.999999, config), "3.999");
        assert_eq!(dtoa(555.5555, config), "555.5");
        assert_eq!(dtoa(923.1, config), "923.1");
    }

    #[test]
    fn test_min_sig_digits() {
        let config = FmtFloatConfig::default()
            .min_significant_digits(5);
        assert_eq!(dtoa(3.111111, config), "3.111111");
        assert_eq!(dtoa(12.0, config), "12.000");
        assert_eq!(dtoa(340.0, config), "340.00");
        assert_eq!(dtoa(123.4567, config), "123.4567");
        assert_eq!(dtoa(0.00123, config), "0.0012300");
    }

    #[test]
    fn test_max_decimal_digits() {
        let config = FmtFloatConfig::default()
            .max_decimal_digits(3)
            .round();
        assert_eq!(dtoa(3.41214, config), "3.412");
        assert_eq!(dtoa(3.4129, config), "3.413");
        assert_eq!(dtoa(3.4999, config), "3.5");
        assert_eq!(dtoa(1293.4129, config), "1293.413");
        assert_eq!(dtoa(203.4999, config), "203.5");
        assert_eq!(dtoa(0.002911, config), "0.003");
        let config = FmtFloatConfig::default()
            .max_decimal_digits(3)
            .truncate();
        assert_eq!(dtoa(3.41214, config), "3.412");
        assert_eq!(dtoa(3.4129, config), "3.412");
        assert_eq!(dtoa(3.4999, config), "3.499");
        assert_eq!(dtoa(393.4129, config), "393.412");
        let config = FmtFloatConfig::default()
            .max_decimal_digits(-3)
            .truncate()
            .force_no_e_notation()
            .add_point_zero(true);
        assert_eq!(dtoa(123123.0, config), "123000.0");
        assert_eq!(dtoa(99999.0, config), "99000.0");
        let config = FmtFloatConfig::default()
            .max_decimal_digits(-3)
            .round()
            .force_no_e_notation()
            .add_point_zero(true);
        assert_eq!(dtoa(123123.0, config), "123000.0");
        assert_eq!(dtoa(123923.0, config), "124000.0");
        assert_eq!(dtoa(99999.0, config), "100000.0");
    }

    #[test]
    fn test_min_decimal_digits() {
        let config = FmtFloatConfig::default()
            .min_decimal_digits(3);
        assert_eq!(dtoa(3.4, config), "3.400");
        assert_eq!(dtoa(10.0, config), "10.000");
        assert_eq!(dtoa(100.0, config), "100.000");
        let config = FmtFloatConfig::default()
            .min_decimal_digits(5);
        assert_eq!(dtoa(0.023, config), "0.02300");
        assert_eq!(dtoa(0.123, config), "0.12300");
        assert_eq!(dtoa(0.12345678, config), "0.12345678");
    }

    #[test]
    fn test_upper_e_break() {
        let config = FmtFloatConfig::default()
            .upper_e_break(3);
        assert_eq!(dtoa(23.4, config), "23.4");
        assert_eq!(dtoa(892.3, config), "892.3");
        assert_eq!(dtoa(1892.3, config), "1.8923e3");
        assert_eq!(dtoa(71892.3, config), "7.18923e4");
    }
    
    #[test]
    fn test_lower_e_break() {
        let config = FmtFloatConfig::default()
            .lower_e_break(-3);
        assert_eq!(dtoa(0.123, config), "0.123");
        assert_eq!(dtoa(0.0123, config), "0.0123");
        assert_eq!(dtoa(0.00123, config), "0.00123");
        assert_eq!(dtoa(0.000123, config), "1.23e-4");
        assert_eq!(dtoa(0.0000123, config), "1.23e-5");
    }

    #[test]
    fn test_ignore_extremes() {
        let config = FmtFloatConfig::default()
            .ignore_extremes(3);
        assert_eq!(dtoa(12.1992, config), "12.1992");
        assert_eq!(dtoa(12.199921, config), "12.2");
        assert_eq!(dtoa(12.1002, config), "12.1002");
        assert_eq!(dtoa(12.10002, config), "12.1");
        let config = FmtFloatConfig::default()
            .ignore_extremes(3)
            .add_point_zero(true);
        assert_eq!(dtoa(99.99, config), "100.0");
    }

    #[test]
    fn test_force_e_notation() {
        let config = FmtFloatConfig::default()
            .force_e_notation();
        assert_eq!(dtoa(1.0, config), "1.0e0");
        assert_eq!(dtoa(15.0, config), "1.5e1");
        assert_eq!(dtoa(0.150, config), "1.5e-1");
    }

    #[test]
    fn test_force_no_e_notation() {
        let config = FmtFloatConfig::default()
            .force_no_e_notation()
            .add_point_zero(false);
        let s = format!("{}", 1.123e20);
        assert_eq!(dtoa(1.123e20, config), s);
        let s = format!("{}", 1.123e-20);
        assert_eq!(dtoa(1.123e-20, config), s);
    }

    #[test]
    fn test_capitalize_e() {
        let config = FmtFloatConfig::default()
            .capitalize_e(true);
        assert_eq!(dtoa(1.2e8, config), "1.2E8");
    }

    #[test]
    fn test_add_point_zero() {
        let config = FmtFloatConfig::default()
            .add_point_zero(true);
        assert_eq!(dtoa(1230.0, config), "1230.0");
        assert_eq!(dtoa(129.0, config), "129.0");
        assert_eq!(dtoa(12.2, config), "12.2");
        let config = FmtFloatConfig::default()
            .add_point_zero(false);
        assert_eq!(dtoa(1230.0, config), "1230");
        assert_eq!(dtoa(129.0, config), "129");
        assert_eq!(dtoa(12.2, config), "12.2");
    }

    #[test]
    fn test_max_width_specifics() {
        check!(e_width_case_a);
        check!(e_width_case_b);
        check!(e_width_case_c);
        let config = FmtFloatConfig::default()
            .max_width(6)
            .force_no_e_notation();
        assert_eq!(dtoa(123.4533, config), "123.45");
        assert_eq!(dtoa(0.00324, config), "0.0032");
        assert_eq!(dtoa(-0.0324, config), "-0.032");
        let config = FmtFloatConfig::default()
            .max_width(8)
            .force_no_e_notation();
        assert_eq!(dtoa(3.24e-10, config), "3.24e-10");
        assert_eq!(dtoa(3.24e10, config), "3.24e10");
    }

    #[test]
    fn test_radix_point() {
        let config = FmtFloatConfig::default()
            .radix_point(',')
            .add_point_zero(true);
        assert_eq!(dtoa(123.4, config), "123,4");
        assert_eq!(dtoa(1.3e10, config), "1,3e10");
        assert_eq!(dtoa(123.0, config), "123,0");
    }
}
