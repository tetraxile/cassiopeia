use paste::paste;

macro_rules! integer_binary_op {
	($trait:ident, $func:ident, $op:tt; $($type:ident),*) => {
		$(
			impl $trait<&$type> for &Integer {
				type Output = Integer;

				fn $func(self, rhs: &$type) -> Self::Output {
					self $op Integer::from(rhs)
				}
			}

			impl $trait<&$type> for Integer {
				type Output = Integer;

				fn $func(self, rhs: &$type) -> Self::Output {
					self $op Integer::from(rhs)
				}
			}

			impl $trait<$type> for &Integer {
				type Output = Integer;

				fn $func(self, rhs: $type) -> Self::Output {
					self $op Integer::from(rhs)
				}
			}

			impl $trait<$type> for Integer {
				type Output = Integer;

				fn $func(self, rhs: $type) -> Self::Output {
					self $op Integer::from(rhs)
				}
			}

      paste::paste! {
        impl [<$trait Assign>]<&$type> for Integer {
          fn [<$func _assign>](&mut self, rhs: &$type) {
            *self = &*self $op rhs
          }
        }

        impl [<$trait Assign>]<$type> for Integer {
          fn [<$func _assign>](&mut self, rhs: $type) {
            *self = &*self $op rhs
          }
        }
      }
		)*
	}
}
macro_rules! integer_shift {
	($trait:ident, $func:ident, $op:tt; $($type:ident),*) => {
    $(
      impl $trait<&$type> for &Integer {
        type Output = Integer;

        fn $func(self, rhs: &$type) -> Self::Output {
          self $op *rhs as usize
        }
      }

      impl $trait<&$type> for Integer {
        type Output = Integer;

        fn $func(self, rhs: &$type) -> Self::Output {
          self $op *rhs as usize
        }
      }

      impl $trait<$type> for &Integer {
        type Output = Integer;

        fn $func(self, rhs: $type) -> Self::Output {
          self $op rhs as usize
        }
      }

      impl $trait<$type> for Integer {
        type Output = Integer;

        fn $func(self, rhs: $type) -> Self::Output {
          self $op rhs as usize
        }
      }

			paste::paste! {
				impl [<$trait Assign>]<&$type> for Integer {
					fn [<$func _assign>](&mut self, rhs: &$type) {
						*self = &*self $op *rhs;
					}
				}

				impl [<$trait Assign>]<$type> for Integer {
					fn [<$func _assign>](&mut self, rhs: $type) {
						*self = &*self $op rhs;
					}
				}
			}
    )*
  };
}

pub(crate) use integer_binary_op;
pub(crate) use integer_shift;
