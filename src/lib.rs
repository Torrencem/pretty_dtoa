
pub mod raw;

use std::char;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum RoundMode {
    Round,
    Truncate,
}

/// A configuration for formatting floats into strings
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct FmtFloatConfig {
    /// A max number of significant digits to include
    /// in the formatted string (after the first non-zero digit)
    /// None means include all digits
    max_sig_digits: Option<u8>,
    /// A min number of significant digits to include
    /// in the formatted string (after the first non-zero digit)
    /// None means no minimum
    min_sig_digits: Option<u8>,
    /// How many digits left of the decimal point there can be
    /// using scientific notation
    upper_e_break: i8,
    /// Lower equivelent of upper_e_break
    lower_e_break: i8,
    /// Ignore digits after (and including) a certain number of
    /// consecutive 9's or 0's
    ignore_extremes: Option<u8>,
    /// Round or truncate
    round_mode: RoundMode,
    /// Force scientific e notation
    force_e_notation: bool,
    /// Force no scientific e notation. Overrides force_e_notation
    force_no_e_notation: bool,
    /// Capitalize the e in scientific notation
    capitalize_e: bool,
    /// Add a .0 at the end of integers
    add_point_zero: bool,
    /// The maximum number of characters in the formatted version of
    /// the float. This will try to respect all other options, unless
    /// they would make the string too long. values less than 5 are not allowed
    max_width: Option<u8>,
}

impl FmtFloatConfig {
    pub const fn human() -> Self {
        FmtFloatConfig {
            max_sig_digits: None,
            min_sig_digits: None,
            upper_e_break: 7,
            lower_e_break: -4,
            ignore_extremes: Some(5),
            round_mode: RoundMode::Round,
            force_e_notation: false,
            force_no_e_notation: false,
            capitalize_e: false,
            add_point_zero: false,
            max_width: None,
        }
    }

    pub const fn precise() -> Self {
        FmtFloatConfig {
            max_sig_digits: None,
            min_sig_digits: None,
            upper_e_break: 4,
            lower_e_break: -4,
            ignore_extremes: None,
            round_mode: RoundMode::Truncate,
            force_e_notation: false,
            force_no_e_notation: false,
            capitalize_e: false,
            add_point_zero: false,
            max_width: None,
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
    
    /// The upper exponent value that will force using exponent notation
    pub const fn upper_e_break(mut self, val: i8) -> Self {
        self.upper_e_break = val;
        self
    }

    /// The lower exponent value that will force using exponent notation
    pub const fn lower_e_break(mut self, val: i8) -> Self {
        self.lower_e_break = val;
        self
    }

    /// Un-does ``ignore_extremes()``, and allows
    /// normal patterns of digits.
    pub const fn allow_extremes(mut self) -> Self {
        self.ignore_extremes = None;
        self
    }

    /// Ignore digits after and including a certain number of
    /// consecutive 9's or 0's. This is useful for printing
    /// numbers with floating point errors to humans, even
    /// if the numbers are technically slightly adjusted.
    /// 3.59999951 -> 3.6
    pub const fn ignore_extremes(mut self, limit: u8) -> Self {
        self.ignore_extremes = Some(limit);
        self
    }
    
    /// When cutting off after a certain number of
    /// significant digits, ignore any further digits.
    /// Opposite of ``round()``.
    pub const fn truncate(mut self) -> Self {
        self.round_mode = RoundMode::Truncate;
        self
    }
    
    /// When cutting off after a certain number of
    /// significant digits, read the next digit and
    /// round up / down.
    pub const fn round(mut self) -> Self {
        self.round_mode = RoundMode::Round;
        self
    }

    /// Force all floats to be in scientific notation
    /// 31 -> 3.1e1
    pub const fn force_e_notation(mut self, val: bool) -> Self {
        self.force_e_notation = val;
        self
    }
    
    /// Force all floats to not be in scientific notation
    /// 3e10 -> 30000000000
    pub const fn force_no_e_notation(mut self, val: bool) -> Self {
        self.force_no_e_notation = val;
        self
    }
    
    /// Capitalize the e in e notation
    /// 3.1e10 -> 3.1E10
    pub const fn capitalize_e(mut self, val: bool) -> Self {
        self.capitalize_e = val;
        self
    }
    
    /// Add a ".0" at the end of integers
    /// 31 -> 31.0
    pub const fn add_point_zero(mut self, val: bool) -> Self {
        self.add_point_zero = val;
        self
    }
    
    /// The maximum number of non-zero digits to include in the string
    pub const fn max_width(mut self, val: u8) -> Self {
        self.max_width = Some(val);
        self
    }
}

pub fn dtoa(value: f64, config: FmtFloatConfig) -> String {
    let (sign, s, mut e) = raw::dtoa(value);
    let mut stripped_string: Vec<u8> = Vec::with_capacity(30);
    let mut nine_counter = 0;
    let mut zero_counter = 0;
    let mut curr = 0;
    for digit in s.iter() {
        if let Some(limit) = config.ignore_extremes {
            if *digit != 9 {
                nine_counter = 0;
            } else {
                nine_counter += 1;
                if nine_counter >= limit {
                    stripped_string.drain((stripped_string.len() + 1 - nine_counter as usize)..);
                    let l = stripped_string.len();
                    if l == 0 {
                        stripped_string.push(1);
                        e += 1;
                    } else {
                        stripped_string[l - 1] += 1;
                    }
                    break;
                }
            }

            if *digit != 0 {
                zero_counter = 0;
            } else {
                zero_counter += 1;
                if zero_counter >= limit {
                    stripped_string.drain((stripped_string.len() + 1 - zero_counter as usize)..);
                    break;
                }
            }
        }
        if let Some(limit) = config.max_sig_digits {
            if curr + 1 > limit {
                if *digit >= 5 && config.round_mode == RoundMode::Round {
                    let mut l = stripped_string.len() - 1;
                    stripped_string[l] += 1;
                    while stripped_string[l] == 10 {
                        if l == 0 {
                            stripped_string[0] = 1;
                            e += 1;
                            break;
                        }
                        stripped_string.pop();
                        l -= 1;
                        stripped_string[l] += 1;
                    }
                }
                break;
            }
        }
        curr += 1;
        stripped_string.push(*digit);
    }
    if let Some(limit) = config.min_sig_digits {
        while curr < limit {
            stripped_string.push(0);
            curr += 1;
        }
    }
    let mut use_e_notation = (e > config.upper_e_break as i32 || e <= config.lower_e_break as i32 || config.force_e_notation) && !config.force_no_e_notation;
    if let Some(max_width) = config.max_width {
        let max_width = if sign { max_width - 1 } else { max_width };
        // Is it impossible to represent the value without e notation?
        if e > 0 && e + if config.add_point_zero { 2 } else { 0 } > max_width as i32 {
            use_e_notation = true;
        } else if -e + 3 > max_width as i32 {
            use_e_notation = true;
        } else if !use_e_notation {
            let is_integer = e > stripped_string.len() as i32;
            let extra_length = if config.add_point_zero && is_integer { 2 } else { 0 }
                             + if !is_integer && !(e > 0 && e as u8 == max_width) { 1 } else { 0 }
                             + if e > 0 && stripped_string.len() < e as usize { e - stripped_string.len() as i32 } else { 0 }
                             + if e <= 0 { -e + 1 } else { 0 };
            let total_length = stripped_string.len() + extra_length as usize;
            if total_length > max_width as usize {
                let final_char = stripped_string.drain((max_width as usize - extra_length as usize)..).nth(0).unwrap();
                if config.round_mode == RoundMode::Round && final_char >= 5 {
                    let mut l = stripped_string.len() - 1;
                    stripped_string[l] += 1;
                    while stripped_string[l] == 10 {
                        if l == 0 {
                            stripped_string[0] = 1;
                            e += 1;
                            break;
                        }
                        stripped_string.pop();
                        l -= 1;
                        stripped_string[l] += 1;
                    }
                }
            }
        }
    }
    if use_e_notation {
        let mut tail_as_str: String = stripped_string.drain(1..).map(|val| char::from_digit(val as u32, 10).unwrap()).collect();
        if let Some(max_width) = config.max_width {
            let e_length = format!("{}", e - 1).len();
            let extra_length = 3 + e_length + if sign { 1 } else { 0 };
            if extra_length >= max_width as usize {
                tail_as_str.drain(..);
            } else { 
                tail_as_str.truncate(max_width as usize - extra_length);
            }
        }
        // Special case
        if tail_as_str.len() == 0 && config.max_width == Some(7) && value < 0.0 {
            return format!("-{}{}{}",
                           stripped_string[0],
                           if config.capitalize_e { "E" } else { "e" },
                           e - 1);
        } else {
            return format!("{}{}.{}{}{}",
                           if sign { "-" } else { "" },
                           stripped_string[0],
                           if tail_as_str.len() == 0 { if config.max_width.is_some() { "" } else { "0" } } else { tail_as_str.as_ref() }, 
                           if config.capitalize_e { "E" } else { "e" },
                           e - 1);
        }
    }
    let mut as_str = String::with_capacity(stripped_string.len() + 3);
    if sign {
        as_str.push('-');
    }
    let mut curr = 0;
    if e <= 0 {
        as_str.push('0');
        as_str.push('.');
        for _ in 0..-e {
            as_str.push('0');
        }
    }
    for digit in stripped_string {
        if e > 0 && curr == e {
            as_str.push('.');
        }
        as_str.push(char::from_digit(digit as u32, 10).unwrap());
        curr += 1;
    }
    let is_integer = curr <= e;
    while e > 0 && curr < e {
        as_str.push('0');
        curr += 1;
    }
    if is_integer && config.add_point_zero {
        as_str.push('.');
        as_str.push('0');
    }

    as_str
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand;
    use rand::Rng;

    #[test]
    fn test_widths() {
        let mut rng = rand::thread_rng();
        
        for width in 6..=13 {
            let config = FmtFloatConfig::human()
                .max_width(width);
            for _ in 0..10000 {
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
    fn test_round_trip() {
        let mut rng = rand::thread_rng();
        
        let config = FmtFloatConfig::precise();
        for _ in 0..20000 {
            let val = f64::from_bits(rng.gen::<u64>());
            if val.is_nan() {
                continue;
            }
            let as_string = dtoa(val, config);
            let round = as_string.parse::<f64>().unwrap();
            assert!(round == val, "Found bad example for round trip: value '{}' gives string '{}' which turns into value '{}'", val, as_string, round);
        }
    }

    #[test]
    fn test_max_sig_digits() {
        let config = FmtFloatConfig::precise()
            .round()
            .max_significant_digits(5);
        assert_eq!(dtoa(3.111111, config), "3.1111");
        assert_eq!(dtoa(123.4567, config), "123.46");
        assert_eq!(dtoa(0.0001234567, config), "0.00012346");
        assert_eq!(dtoa(22.29999, config), "22.3");
        assert_eq!(dtoa(4.1, config), "4.1");
        let config = FmtFloatConfig::precise()
            .truncate()
            .max_significant_digits(4);
        assert_eq!(dtoa(3.999999, config), "3.999");
        assert_eq!(dtoa(555.5555, config), "555.5");
        assert_eq!(dtoa(923.1, config), "923.1");
    }

    #[test]
    fn test_min_sig_digits() {
        let config = FmtFloatConfig::precise()
            .min_significant_digits(5);
        assert_eq!(dtoa(3.111111, config), "3.111111");
        assert_eq!(dtoa(12.0, config), "12.000");
        assert_eq!(dtoa(340.0, config), "340.00");
        assert_eq!(dtoa(123.4567, config), "123.4567");
        assert_eq!(dtoa(0.00123, config), "0.0012300");
    }

    #[test]
    fn test_upper_e_break() {
        let config = FmtFloatConfig::precise()
            .upper_e_break(3);
        assert_eq!(dtoa(23.4, config), "23.4");
        assert_eq!(dtoa(892.3, config), "892.3");
        assert_eq!(dtoa(1892.3, config), "1.8923e3");
        assert_eq!(dtoa(71892.3, config), "7.18923e4");
    }
    
    #[test]
    fn test_lower_e_break() {
        let config = FmtFloatConfig::precise()
            .lower_e_break(-3);
        assert_eq!(dtoa(0.123, config), "0.123");
        assert_eq!(dtoa(0.0123, config), "0.0123");
        assert_eq!(dtoa(0.00123, config), "0.00123");
        assert_eq!(dtoa(0.000123, config), "1.23e-4");
        assert_eq!(dtoa(0.0000123, config), "1.23e-5");
    }

    #[test]
    fn test_ignore_extremes() {
        let config = FmtFloatConfig::precise()
            .ignore_extremes(3);
        assert_eq!(dtoa(12.1992, config), "12.1992");
        assert_eq!(dtoa(12.199921, config), "12.2");
        assert_eq!(dtoa(12.1002, config), "12.1002");
        assert_eq!(dtoa(12.10002, config), "12.1");
    }

    #[test]
    fn test_force_e_notation() {
        let config = FmtFloatConfig::precise()
            .force_e_notation(true);
        assert_eq!(dtoa(1.0, config), "1.0e0");
        assert_eq!(dtoa(15.0, config), "1.5e1");
        assert_eq!(dtoa(0.150, config), "1.5e-1");
    }

    #[test]
    fn test_force_no_e_notation() {
        let config = FmtFloatConfig::precise()
            .force_no_e_notation(true);
        let s = format!("{}", 1.123e20);
        assert_eq!(dtoa(1.123e20, config), s);
        let s = format!("{}", 1.123e-20);
        assert_eq!(dtoa(1.123e-20, config), s);
    }

    #[test]
    fn test_capitalize_e() {
        let config = FmtFloatConfig::precise()
            .capitalize_e(true);
        assert_eq!(dtoa(1.2e8, config), "1.2E8");
    }

    #[test]
    fn test_add_point_zero() {
        let config = FmtFloatConfig::precise()
            .add_point_zero(true);
        assert_eq!(dtoa(1230.0, config), "1230.0");
        assert_eq!(dtoa(129.0, config), "129.0");
        assert_eq!(dtoa(12.2, config), "12.2");
        let config = FmtFloatConfig::precise()
            .add_point_zero(false);
        assert_eq!(dtoa(1230.0, config), "1230");
        assert_eq!(dtoa(129.0, config), "129");
        assert_eq!(dtoa(12.2, config), "12.2");
    }

    #[test]
    fn test_max_width_specifics() {
        let config = FmtFloatConfig::precise()
            .max_width(6);
        assert_eq!(dtoa(123.4533, config), "123.45");
        assert_eq!(dtoa(0.00324, config), "0.0032");
        assert_eq!(dtoa(-0.0324, config), "-0.032");
    }

    // #[test]
    // fn test_specific() {
    //     let config = FmtFloatConfig::human()
    //         .max_width(8);
    //     println!("p: {}", dtoa(0.00024027723047814753, config));
    // }
}
