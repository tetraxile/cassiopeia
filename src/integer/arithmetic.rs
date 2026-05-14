use crate::integer::{Integer, Sign, macros::integer_binary_op};
use itertools::{EitherOrBoth::*, Itertools};
use rand::prelude::*;
use std::cmp::Ordering;
use std::iter::zip;
use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

impl Neg for &Integer {
	type Output = Integer;

	fn neg(self) -> Self::Output {
		Self::Output {
			sign: self.sign.neg(),
			bit_length: self.bit_length,
			bytes: self.bytes.clone(),
		}
	}
}

impl Neg for Integer {
	type Output = Integer;

	fn neg(self) -> Self::Output {
		-&self
	}
}

impl Add<&Integer> for &Integer {
	type Output = Integer;

	fn add(self, rhs: &Integer) -> Self::Output {
		let adder = |lhs: &Integer, rhs: &Integer, out_sign: Sign, carry_in: bool, swap: bool| {
			let mut lhs = lhs;
			let mut rhs = rhs;

			if swap {
				std::mem::swap(&mut lhs, &mut rhs);
			}

			let mut carry = carry_in;
			let mut bytes = Itertools::zip_longest(lhs.bytes.iter(), rhs.bytes.iter())
				.map(|pair| {
					let (a, b) = match pair {
						Both(l, r) => (*l, *r),
						Left(l) => (*l, 0u8),
						Right(r) => (0u8, *r),
					};

					let b = if carry_in { !b } else { b };
					let sum: u8;
					(sum, carry) = a.carrying_add(b, carry);
					sum
				})
				.collect::<Vec<_>>();

			if carry && !carry_in {
				bytes.push(1);
			}

			let byte_length = bytes
				.iter()
				.rposition(|b| *b != 0)
				.expect("sum is never zero, because of the cases when this closure is called")
				+ 1;
			bytes = bytes[..byte_length].to_vec();

			let bit_length =
				bytes.len() * u8::BITS as usize - bytes.last().unwrap().leading_zeros() as usize;

			Integer {
				sign: out_sign,
				bit_length,
				bytes,
			}
		};

		match (self.sign, rhs.sign) {
			(_, Sign::NoSign) => self.clone(),
			(Sign::NoSign, _) => rhs.clone(),
			(Sign::Positive, Sign::Positive) => adder(self, rhs, Sign::Positive, false, false),
			(Sign::Negative, Sign::Negative) => adder(self, rhs, Sign::Negative, false, false),
			(Sign::Positive, Sign::Negative) => match self.cmp(&-rhs.clone()) {
				Ordering::Equal => Integer::zero(),
				Ordering::Less => adder(self, rhs, Sign::Negative, true, true),
				Ordering::Greater => adder(self, rhs, Sign::Positive, true, false),
			},
			(Sign::Negative, Sign::Positive) => match self.cmp(&-rhs.clone()) {
				Ordering::Equal => Integer::zero(),
				Ordering::Less => adder(self, rhs, Sign::Negative, true, false),
				Ordering::Greater => adder(self, rhs, Sign::Positive, true, true),
			},
		}
	}
}

impl Sub<&Integer> for &Integer {
	type Output = Integer;

	fn sub(self, rhs: &Integer) -> Self::Output {
		self + -rhs
	}
}

integer_binary_op!(Add, add, +; isize, i64, i32, i16, i8, usize, u64, u32, u16, u8);
integer_binary_op!(Sub, sub, -; isize, i64, i32, i16, i8, usize, u64, u32, u16, u8);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_add() {
		let mut rng = rand::rng();

		for _ in 0..1000 {
			let a: i64 = rng.random_range(-1024..1024);
			let b: i64 = rng.random_range(-1024..1024);
			let na = Integer::from(a);
			let nb = Integer::from(b);
			assert_eq!(Integer::from(a + b), &na + &nb, "{na:b} + {nb:b}");
			assert_eq!(Integer::from(a + 0), &na + 0, "{na:b} + 0");
			assert_eq!(Integer::from(0 + b), Integer::zero() + &nb, "0 + {nb:b}");
		}
	}

	#[test]
	fn test_sub() {
		let mut rng = rand::rng();

		for _ in 0..1000 {
			let a: i64 = rng.random_range(-1024..1024);
			let b: i64 = rng.random_range(-1024..1024);
			let na = Integer::from(a);
			let nb = Integer::from(b);
			assert_eq!(Integer::from(a - b), &na - &nb, "{na:b} - {nb:b}");
			assert_eq!(Integer::from(a - 0), &na - 0, "{na:b} - 0");
			assert_eq!(Integer::from(0 - b), Integer::zero() - &nb, "0 - {nb:b}");
		}
	}
}
