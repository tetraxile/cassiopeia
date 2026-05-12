use crate::integer::{Integer, Sign};
use rand::prelude::*;
use std::ops::{Shl, ShlAssign, Shr, ShrAssign};

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

macro_rules! integer_shl {
	($($type:ident),*) => {
    $(
      impl Shl<&$type> for &Integer {
        type Output = Integer;

        fn shl(self, rhs: &$type) -> Self::Output {
          self << *rhs as usize
        }
      }

      impl Shl<&$type> for Integer {
        type Output = Integer;

        fn shl(self, rhs: &$type) -> Self::Output {
          self << *rhs as usize
        }
      }

      impl Shl<$type> for &Integer {
        type Output = Integer;

        fn shl(self, rhs: $type) -> Self::Output {
          self << rhs as usize
        }
      }

      impl Shl<$type> for Integer {
        type Output = Integer;

        fn shl(self, rhs: $type) -> Self::Output {
          self << rhs as usize
        }
      }
    )*
  };
}

macro_rules! integer_shl_assign {
	($($type:ident),*) => {
    $(
      impl ShlAssign<&$type> for Integer {
        fn shl_assign(&mut self, rhs: &$type) {
          *self = &*self << *rhs;
        }
      }

      impl ShlAssign<$type> for Integer {
        fn shl_assign(&mut self, rhs: $type) {
          *self = &*self << rhs;
        }
      }
    )*
  };
}

integer_shl!(u64, u32, u16, u8);
integer_shl_assign!(usize, u64, u32, u16, u8);

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

macro_rules! integer_shr {
	($($type:ident),*) => {
    $(
      impl Shr<&$type> for &Integer {
        type Output = Integer;

        fn shr(self, rhs: &$type) -> Self::Output {
          self >> *rhs as usize
        }
      }

      impl Shr<&$type> for Integer {
        type Output = Integer;

        fn shr(self, rhs: &$type) -> Self::Output {
          self >> *rhs as usize
        }
      }

      impl Shr<$type> for &Integer {
        type Output = Integer;

        fn shr(self, rhs: $type) -> Self::Output {
          self >> rhs as usize
        }
      }

      impl Shr<$type> for Integer {
        type Output = Integer;

        fn shr(self, rhs: $type) -> Self::Output {
          self >> rhs as usize
        }
      }
    )*
  };
}

macro_rules! integer_shr_assign {
	($($type:ident),*) => {
    $(
      impl ShrAssign<&$type> for Integer {
        fn shr_assign(&mut self, rhs: &$type) {
          *self = &*self >> *rhs;
        }
      }

      impl ShrAssign<$type> for Integer {
        fn shr_assign(&mut self, rhs: $type) {
          *self = &*self >> rhs;
        }
      }
    )*
  };
}

integer_shr!(u64, u32, u16, u8);
integer_shr_assign!(usize, u64, u32, u16, u8);

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
}
