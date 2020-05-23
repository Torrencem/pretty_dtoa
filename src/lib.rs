
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
    /// A master number of significant digits to include
    /// in the formatted string (after the first non-zero digit)
    /// None means include all digits
    pub master_sig_figs: Option<u8>,
    /// How large log_10(n) can be before using scientific notation
    pub upper_e_break: i8,
    /// How low log_10(n) can be before using scientific notation
    pub lower_e_break: i8,
    /// Ignore digits after (and including) a certain number of
    /// consecutive 9's or 0's
    pub ignore_extremes: Option<u8>,
    /// Round or truncate
    pub round_mode: RoundMode,
    /// Force scientific e notation
    pub force_e_notation: bool,
    /// Capitalize the e in scientific notation
    pub capitalize_e: bool,
}

impl FmtFloatConfig {
    pub const fn default() -> Self {
        FmtFloatConfig {
            master_sig_figs: None,
            upper_e_break: 7,
            lower_e_break: -4,
            ignore_extremes: Some(4),
            round_mode: RoundMode::Round,
            force_e_notation: false,
            capitalize_e: false,
        }
    }

    pub const fn exact() -> Self {
        FmtFloatConfig {
            master_sig_figs: None,
            upper_e_break: 7,
            lower_e_break: -4,
            ignore_extremes: None,
            round_mode: RoundMode::Truncate,
            force_e_notation: false,
            capitalize_e: false,
        }
    }
}

pub fn dtoa(value: f64, config: FmtFloatConfig) -> String {
    let (s, mut e) = raw::dtoa(value);
    // println!("{:?}", s);
    // println!("{}", e);
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
        if let Some(limit) = config.master_sig_figs {
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
    let use_e_notation = e > config.upper_e_break as i32 || e < config.lower_e_break as i32 || config.force_e_notation;
    if use_e_notation {
        let tail_as_str: String = stripped_string.drain(1..).map(|val| char::from_digit(val as u32, 10).unwrap()).collect();
        return format!("{}.{}{}{}",
                       stripped_string[0],
                       if tail_as_str.len() == 0 { "0" } else { tail_as_str.as_ref() }, 
                       if config.capitalize_e { "E" } else { "e" },
                       e - 1);
    }
    let mut as_str = String::new();
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
    as_str
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dtoa() {
        println!("{}", dtoa(123.45678, FmtFloatConfig::default()));
    }
}
