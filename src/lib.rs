
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
    /// How large log_10(n) can be before using scientific notation
    upper_e_break: i8,
    /// How low log_10(n) can be before using scientific notation
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
        }
    }

    pub const fn exact() -> Self {
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
}

pub fn dtoa(value: f64, config: FmtFloatConfig) -> String {
    let (sign, s, mut e) = raw::dtoa(value);
    let mut stripped_string: Vec<u8> = Vec::new();
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
            if curr > limit {
                if *digit >= 5 && config.round_mode == RoundMode::Round {
                    let l = stripped_string.len() - 1;
                    stripped_string[l] += 1;
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
    let use_e_notation = e > config.upper_e_break as i32 || e < config.lower_e_break as i32 || config.force_e_notation;
    if use_e_notation && !config.force_no_e_notation {
        let tail_as_str: String = stripped_string.drain(1..).map(|val| char::from_digit(val as u32, 10).unwrap()).collect();
        return format!("{}{}.{}{}{}",
                       if sign { "-" } else { "" },
                       stripped_string[0],
                       if tail_as_str.len() == 0 { "0" } else { tail_as_str.as_ref() }, 
                       if config.capitalize_e { "E" } else { "e" },
                       e - 1);
    }
    let mut as_str = String::new();
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

    #[test]
    fn test_dtoa() {
        let config = FmtFloatConfig::exact();
        println!("p: {}", dtoa(2.991, config));
    }
}
