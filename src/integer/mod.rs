#![allow(unused_imports)]

use paste::paste;
use rand::prelude::*;
use std::cmp::Ordering;
use std::fmt;
use std::ops::Neg;

mod arithmetic;
mod bitwise;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sign {
	NoSign,
	Positive,
	Negative,
}

impl Neg for Sign {
	type Output = Self;

	fn neg(self) -> Self::Output {
		match self {
			Sign::NoSign => Sign::NoSign,
			Sign::Positive => Sign::Negative,
			Sign::Negative => Sign::Positive,
		}
	}
}

impl Into<i8> for Sign {
	fn into(self) -> i8 {
		match self {
			Sign::NoSign => 0,
			Sign::Positive => 1,
			Sign::Negative => -1,
		}
	}
}

#[derive(Clone, Debug)]
pub struct Integer {
	sign: Sign,
	bit_length: usize,
	bytes: Vec<u8>,
}

impl Integer {
	fn from_i64(raw: i64) -> Self {
		let sign = match raw.cmp(&0) {
			Ordering::Less => Sign::Negative,
			Ordering::Equal => {
				return Integer::zero();
			}
			Ordering::Greater => Sign::Positive,
		};

		let mut bytes: Vec<u8> = vec![];

		let mut acc = raw as u64;
		if sign == Sign::Positive {
			let mut remaining = raw;
			while remaining != 0 {
				bytes.push((remaining & 0xff) as u8);
				remaining >>= 8;
			}
		} else if sign == Sign::Negative {
			let mut carry = true;
			let mut byte: u8 = 0;
			let mut bit_idx = 0;
			for _ in 0..=i64::BITS {
				let bit = acc & 1 == 0;
				byte |= ((bit ^ carry) as u8) << bit_idx;
				bit_idx += 1;
				acc >>= 1;
				carry &= bit;
				if bit_idx == 8 {
					bytes.push(byte);
					byte = 0;
					bit_idx = 0;
				}
			}

			let byte_length = match bytes.iter().rposition(|b| *b != 0) {
				Some(l) => l + 1,
				None => todo!(),
			};
			bytes = bytes[..byte_length].to_vec();
		}

		let bit_length =
			bytes.len() * u8::BITS as usize - bytes.last().unwrap().leading_zeros() as usize;

		Self {
			sign,
			bit_length,
			bytes,
		}
	}

	fn from_u64(raw: u64) -> Self {
		let sign = match raw.cmp(&0) {
			Ordering::Less => unreachable!("unsigned cannot be less than 0"),
			Ordering::Equal => {
				return Integer::zero();
			}
			Ordering::Greater => Sign::Positive,
		};

		let mut bytes: Vec<u8> = vec![];

		let mut remaining = raw;
		while remaining != 0 {
			bytes.push((remaining & 0xff) as u8);
			remaining >>= 8;
		}
		let bit_length =
			bytes.len() * u8::BITS as usize - bytes.last().unwrap().leading_zeros() as usize;

		Self {
			sign,
			bit_length,
			bytes,
		}
	}

	pub fn zero() -> Self {
		Self {
			sign: Sign::NoSign,
			bit_length: 0,
			bytes: vec![],
		}
	}

	pub fn abs(self) -> Self {
		match self.sign {
			Sign::NoSign => self,
			Sign::Positive => self,
			Sign::Negative => -self,
		}
	}

	pub fn iter_bits<'a>(&'a self) -> BitIterator<'a> {
		BitIterator::new(self)
	}

	pub fn is_zero(&self) -> bool {
		return self.bit_length == 0;
	}
}

pub struct BitIterator<'a> {
	integer: &'a Integer,
	current_idx: usize,
}

impl<'a> BitIterator<'a> {
	fn new(integer: &'a Integer) -> Self {
		Self {
			integer,
			current_idx: 0,
		}
	}
}

impl Iterator for BitIterator<'_> {
	type Item = bool;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current_idx < self.integer.bit_length {
			let byte_idx = self.current_idx >> 3;
			let bit_idx = self.current_idx & 0b111;
			let bit = self.integer.bytes[byte_idx] >> bit_idx & 1;
			self.current_idx += 1;
			Some(bit == 1)
		} else {
			None
		}
	}
}

macro_rules! integer_from {
	($from:ident; $($type:ident),*) => {
    $(
      impl From<$type> for Integer {
        fn from(value: $type) -> Self {
          Integer::$from(value as _)
        }
      }

      impl From<&$type> for Integer {
        fn from(value: &$type) -> Self {
          Integer::$from(*value as _)
        }
      }
    )*
  };
}

integer_from!(from_i64; isize, i64, i32, i16, i8);
integer_from!(from_u64; usize, u64, u32, u16, u8);

impl fmt::Binary for Integer {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.sign == Sign::NoSign {
			return write!(f, "0");
		}

		let mut is_first = true;
		let mut bit_string = String::new();
		for byte in self.bytes.iter().rev() {
			if is_first {
				bit_string.push_str(&format!("{:0b}", byte));
				is_first = false;
			} else {
				bit_string.push_str(&format!("{:08b}", byte));
			}
		}

		let sign_string = match self.sign {
			Sign::Positive => "",
			Sign::Negative => "-",
			_ => unreachable!(),
		};
		write!(f, "{}{}", sign_string, bit_string)
	}
}

// impl fmt::Display for Integer {
// 	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
// 		let mut acc = 0;
// 		for bit in self.bits.iter().rev() {
// 			acc <<= 1;
// 			acc |= *bit as i64;
// 			if acc > 10 {

// 			}
// 		}

// 		write!(f, "balls")
// 	}
// }

// impl fmt::Debug for Number {
// 	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
// 		fmt::Display::fmt(self, f)
// 	}
// }

impl Default for Integer {
	fn default() -> Self {
		Integer::zero()
	}
}

impl PartialEq for Integer {
	fn eq(&self, other: &Self) -> bool {
		self.sign == other.sign && self.bytes == other.bytes && self.bit_length == other.bit_length
	}
}

impl Eq for Integer {}

impl PartialOrd for Integer {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Integer {
	fn cmp(&self, rhs: &Self) -> Ordering {
		match (self.sign, rhs.sign) {
			(Sign::Negative, Sign::Negative) => (),
			(Sign::Positive, Sign::Positive) => (),
			(Sign::Positive, _) => return Ordering::Greater,
			(Sign::Negative, _) => return Ordering::Less,
			(Sign::NoSign, Sign::NoSign) => return Ordering::Equal,
			(Sign::NoSign, Sign::Positive) => return Ordering::Less,
			(Sign::NoSign, Sign::Negative) => return Ordering::Greater,
		};

		// at this point the signs are either both negative or both positive

		let mut a = self.bytes.clone();
		let mut b = rhs.bytes.clone();

		// make sure both vectors are the same size (pad with zeroes)
		if b.len() > a.len() {
			a.extend(vec![0; b.len() - a.len()]);
		} else if a.len() > b.len() {
			b.extend(vec![0; a.len() - b.len()]);
		}

		a.reverse();
		b.reverse();

		match (a.cmp(&b), self.sign) {
			(Ordering::Equal, _) => Ordering::Equal,
			(Ordering::Less, Sign::Positive) => Ordering::Less,
			(Ordering::Less, Sign::Negative) => Ordering::Greater,
			(Ordering::Greater, Sign::Positive) => Ordering::Greater,
			(Ordering::Greater, Sign::Negative) => Ordering::Less,
			(_, Sign::NoSign) => unreachable!(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_ord() {
		let mut rng = rand::rng();

		for _ in 0..1000 {
			let a = rng.random_range(-1024..1024);
			let b = rng.random_range(-1024..1024);
			let na = Integer::from(a);
			let nb = Integer::from(b);
			assert_eq!(
				a.cmp(&b),
				Integer::from(a).cmp(&Integer::from(b)),
				"comparing {na:b} and {nb:b}"
			);
		}
	}

	// #[test]
	// fn test_to_string() {
	// 	let nums = [
	// 		(0, "0"),
	// 		(5, "101"),
	// 		(-5, "-101"),
	// 		(0b110101001, "110101001"),
	// 		(
	// 			i64::MAX,
	// 			"111111111111111111111111111111111111111111111111111111111111111",
	// 		),
	// 		(
	// 			i64::MIN,
	// 			"-1000000000000000000000000000000000000000000000000000000000000000",
	// 		),
	// 	];
	// 	for (num, num_str) in nums {
	// 		assert_eq!(String::from(num_str), Number::new(num).to_string());
	// 	}
	// }

	#[test]
	fn test_binary_string() {
		let nums = [
			(0, "0"),
			(5, "101"),
			(-5, "-101"),
			(0b110101001, "110101001"),
			(
				i64::MAX,
				"111111111111111111111111111111111111111111111111111111111111111",
			),
			(
				i64::MIN,
				"-1000000000000000000000000000000000000000000000000000000000000000",
			),
		];
		for (num, num_str) in nums {
			assert_eq!(String::from(num_str), format!("{:b}", Integer::from(num)));
		}
	}
}
