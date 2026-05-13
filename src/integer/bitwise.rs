use crate::integer::{
	Integer, Sign,
	macros::{integer_binary_op, integer_shift},
};
use itertools::{EitherOrBoth::*, Itertools};
use rand::prelude::*;
use std::iter::zip;
use std::ops::{
	BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Shl, ShlAssign, Shr, ShrAssign,
};

impl Shl<&usize> for &Integer {
	type Output = Integer;

	fn shl(self, rhs: &usize) -> Self::Output {
		if self.is_zero() {
			return Integer::zero();
		}

		let byte_shift = rhs >> 3;
		let bit_shift = rhs & 0b111;
		let mut bytes = vec![0; byte_shift];

		let mut shifted_curr = (self.bytes[0] as u16) << bit_shift;
		bytes.push(shifted_curr as u8);

		let mut shifted_next;
		for byte in &self.bytes[1..] {
			shifted_next = (*byte as u16) << bit_shift;
			bytes.push((shifted_curr >> 8) as u8 | shifted_next as u8);
			shifted_curr = shifted_next;
		}

		if shifted_curr >> 8 != 0 {
			bytes.push((shifted_curr >> 8) as u8);
		}

		Integer {
			sign: self.sign,
			bit_length: self.bit_length + rhs,
			bytes,
		}
	}
}

impl Shl<usize> for &Integer {
	type Output = Integer;

	fn shl(self, rhs: usize) -> Self::Output {
		self << &rhs
	}
}

impl Shl<&usize> for Integer {
	type Output = Integer;

	fn shl(self, rhs: &usize) -> Self::Output {
		&self << rhs
	}
}

impl Shl<usize> for Integer {
	type Output = Integer;

	fn shl(self, rhs: usize) -> Self::Output {
		&self << rhs
	}
}

impl ShlAssign<&usize> for Integer {
	fn shl_assign(&mut self, rhs: &usize) {
		*self = &*self << *rhs;
	}
}

impl ShlAssign<usize> for Integer {
	fn shl_assign(&mut self, rhs: usize) {
		*self = &*self << rhs;
	}
}

impl Shr<&usize> for &Integer {
	type Output = Integer;

	fn shr(self, rhs: &usize) -> Self::Output {
		if *rhs >= self.bit_length {
			return Integer::zero();
		} else if *rhs == 0 {
			return self.clone();
		}

		let byte_shift = rhs >> 3;
		let bit_shift = rhs & 0b111;
		let mut bytes = vec![];

		let mut shifted_curr: u8 = self.bytes[byte_shift] >> bit_shift;
		let mut shifted_next: u8;
		for byte in &self.bytes[byte_shift + 1..] {
			shifted_next = (*byte).unbounded_shl(u8::BITS - bit_shift as u32);
			bytes.push(shifted_curr | shifted_next);
			shifted_curr = byte >> bit_shift;
		}
		if shifted_curr != 0 {
			bytes.push(shifted_curr);
		}

		Integer {
			sign: self.sign,
			bit_length: self.bit_length - rhs,
			bytes,
		}
	}
}

impl Shr<usize> for &Integer {
	type Output = Integer;

	fn shr(self, rhs: usize) -> Self::Output {
		self >> &rhs
	}
}

impl Shr<&usize> for Integer {
	type Output = Integer;

	fn shr(self, rhs: &usize) -> Self::Output {
		&self >> rhs
	}
}

impl Shr<usize> for Integer {
	type Output = Integer;

	fn shr(self, rhs: usize) -> Self::Output {
		&self >> rhs
	}
}

impl ShrAssign<&usize> for Integer {
	fn shr_assign(&mut self, rhs: &usize) {
		*self = &*self >> *rhs;
	}
}

impl ShrAssign<usize> for Integer {
	fn shr_assign(&mut self, rhs: usize) {
		*self = &*self >> rhs;
	}
}

impl BitAnd<&Integer> for &Integer {
	type Output = Integer;

	fn bitand(self, rhs: &Integer) -> Self::Output {
		assert!(
			!self.is_negative() && !rhs.is_negative(),
			"inputs to bitwise AND cannot be negative"
		);

		let mut bytes: Vec<u8> = Itertools::zip_longest(self.bytes.iter(), rhs.bytes.iter())
			.map(|pair| if let Both(l, r) = pair { l & r } else { 0 })
			.collect();

		match bytes.iter().rposition(|b| *b != 0) {
			None => Integer::zero(),
			Some(l) => {
				bytes = bytes[..l + 1].to_vec();
				let bit_length =
					bytes.len() * u8::BITS as usize - bytes.last().unwrap().leading_zeros() as usize;
				Integer {
					sign: Sign::Positive,
					bit_length,
					bytes,
				}
			}
		}
	}
}

impl BitAnd<&Integer> for Integer {
	type Output = Integer;

	fn bitand(self, rhs: &Integer) -> Self::Output {
		&self & rhs
	}
}

impl BitAnd<Integer> for &Integer {
	type Output = Integer;

	fn bitand(self, rhs: Integer) -> Self::Output {
		self & &rhs
	}
}

impl BitAnd<Integer> for Integer {
	type Output = Integer;

	fn bitand(self, rhs: Integer) -> Self::Output {
		self & &rhs
	}
}

impl BitAndAssign<&Integer> for Integer {
	fn bitand_assign(&mut self, rhs: &Integer) {
		*self = &*self & rhs
	}
}

impl BitAndAssign<Integer> for Integer {
	fn bitand_assign(&mut self, rhs: Integer) {
		*self = &*self & rhs
	}
}

impl BitOr<&Integer> for &Integer {
	type Output = Integer;

	fn bitor(self, rhs: &Integer) -> Self::Output {
		assert!(
			!self.is_negative() && !rhs.is_negative(),
			"inputs to bitwise OR cannot be negative"
		);

		let mut bytes: Vec<u8> = Itertools::zip_longest(self.bytes.iter(), rhs.bytes.iter())
			.map(|pair| match pair {
				Both(l, r) => l | r,
				Left(l) => *l,
				Right(r) => *r,
			})
			.collect();

		match bytes.iter().rposition(|b| *b != 0) {
			None => Integer::zero(),
			Some(l) => {
				bytes = bytes[..l + 1].to_vec();
				let bit_length =
					bytes.len() * u8::BITS as usize - bytes.last().unwrap().leading_zeros() as usize;
				Integer {
					sign: Sign::Positive,
					bit_length,
					bytes,
				}
			}
		}
	}
}

impl BitOr<&Integer> for Integer {
	type Output = Integer;

	fn bitor(self, rhs: &Integer) -> Self::Output {
		&self | rhs
	}
}

impl BitOr<Integer> for &Integer {
	type Output = Integer;

	fn bitor(self, rhs: Integer) -> Self::Output {
		self | &rhs
	}
}

impl BitOr<Integer> for Integer {
	type Output = Integer;

	fn bitor(self, rhs: Integer) -> Self::Output {
		self | &rhs
	}
}

impl BitOrAssign<&Integer> for Integer {
	fn bitor_assign(&mut self, rhs: &Integer) {
		*self = &*self | rhs
	}
}

impl BitOrAssign<Integer> for Integer {
	fn bitor_assign(&mut self, rhs: Integer) {
		*self = &*self | rhs
	}
}

impl BitXor<&Integer> for &Integer {
	type Output = Integer;

	fn bitxor(self, rhs: &Integer) -> Self::Output {
		assert!(
			!self.is_negative() && !rhs.is_negative(),
			"inputs to bitwise XOR cannot be negative"
		);

		let mut bytes: Vec<u8> = Itertools::zip_longest(self.bytes.iter(), rhs.bytes.iter())
			.map(|pair| match pair {
				Both(l, r) => l ^ r,
				Left(l) => *l,
				Right(r) => *r,
			})
			.collect();

		match bytes.iter().rposition(|b| *b != 0) {
			None => Integer::zero(),
			Some(l) => {
				bytes = bytes[..l + 1].to_vec();
				let bit_length =
					bytes.len() * u8::BITS as usize - bytes.last().unwrap().leading_zeros() as usize;
				Integer {
					sign: Sign::Positive,
					bit_length,
					bytes,
				}
			}
		}
	}
}

impl BitXor<&Integer> for Integer {
	type Output = Integer;

	fn bitxor(self, rhs: &Integer) -> Self::Output {
		&self ^ rhs
	}
}

impl BitXor<Integer> for &Integer {
	type Output = Integer;

	fn bitxor(self, rhs: Integer) -> Self::Output {
		self ^ &rhs
	}
}

impl BitXor<Integer> for Integer {
	type Output = Integer;

	fn bitxor(self, rhs: Integer) -> Self::Output {
		self ^ &rhs
	}
}

impl BitXorAssign<&Integer> for Integer {
	fn bitxor_assign(&mut self, rhs: &Integer) {
		*self = &*self ^ rhs
	}
}

impl BitXorAssign<Integer> for Integer {
	fn bitxor_assign(&mut self, rhs: Integer) {
		*self = &*self ^ rhs
	}
}

integer_shift!(Shr, shr, >>; u64, u32, u16, u8);
integer_shift!(Shl, shl, <<; u64, u32, u16, u8);
integer_binary_op!(BitAnd, bitand, &; isize, i64, i32, i16, i8, usize, u64, u32, u16, u8);
integer_binary_op!(BitOr, bitor, |; isize, i64, i32, i16, i8, usize, u64, u32, u16, u8);
integer_binary_op!(BitXor, bitxor, ^; isize, i64, i32, i16, i8, usize, u64, u32, u16, u8);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_shift() {
		let mut rng = rand::rng();

		for _ in 0..1000 {
			let a: i64 = rng.random_range(-1024..1024);
			let b: usize = rng.random_range(..50);
			let na = Integer::from(a);
			let nz = Integer::zero();
			assert_eq!(Integer::from(a << b), &na << b, "{na:b} << {b}");
			assert_eq!(Integer::from(a << 0), &na << 0usize, "{na:b} << 0");
			assert_eq!(Integer::from(0i64 << b), &nz << b, "{nz:b} << b");
		}

		for _ in 0..1000 {
			let a: i64 = rng.random_range(-1024..1024);
			let b: usize = rng.random_range(..50);
			let na = Integer::from(a);
			let nz = Integer::zero();
			assert_eq!(
				Integer::from(if a < 0 { -(-a >> b) } else { a >> b }),
				&na >> b,
				"{na:b} >> {b}"
			);
			assert_eq!(Integer::from(a >> 0), &na >> 0usize, "{na:b} >> 0");
			assert_eq!(Integer::from(0i64 >> b), &nz >> b, "{nz:b} >> b");
		}
	}

	#[test]
	fn test_bitand() {
		let mut rng = rand::rng();

		for _ in 0..1000 {
			let a: u64 = rng.random_range(..1024);
			let b: u64 = rng.random_range(..1024);
			let na = Integer::from(a);
			let nz = Integer::zero();
			assert_eq!(Integer::from(a & b), &na & b, "{na:b} & {b}");
			assert_eq!(Integer::from(a & 0), &na & 0usize, "{na:b} & 0");
			assert_eq!(Integer::from(0 & b), &nz & b, "{nz:b} & b");
		}
	}

	#[test]
	fn test_bitor() {
		let mut rng = rand::rng();

		for _ in 0..1000 {
			let a: u64 = rng.random_range(..1024);
			let b: u64 = rng.random_range(..1024);
			let na = Integer::from(a);
			let nz = Integer::zero();
			assert_eq!(Integer::from(a | b), &na | b, "{na:b} | {b}");
			assert_eq!(Integer::from(a | 0), &na | 0usize, "{na:b} | 0");
			assert_eq!(Integer::from(0 | b), &nz | b, "{nz:b} | b");
		}
	}

	#[test]
	fn test_bitxor() {
		let mut rng = rand::rng();

		for _ in 0..1000 {
			let a: u64 = rng.random_range(..1024);
			let b: u64 = rng.random_range(..1024);
			let na = Integer::from(a);
			let nz = Integer::zero();
			assert_eq!(Integer::from(a ^ b), &na ^ b, "{na:b} ^ {b}");
			assert_eq!(Integer::from(a ^ 0), &na ^ 0usize, "{na:b} ^ 0");
			assert_eq!(Integer::from(0 ^ b), &nz ^ b, "{nz:b} ^ b");
		}
	}
}
