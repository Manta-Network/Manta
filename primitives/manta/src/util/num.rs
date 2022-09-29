// Copyright 2020-2022 Manta Network.
// This file is part of Manta.
//
// Manta is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Manta is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Manta.  If not, see <http://www.gnu.org/licenses/>.

//! Numeric Traits

/// Checked Addition
pub trait CheckedAdd<Rhs = Self> {
    /// Output Type
    type Output;

    /// Checked integer addition. Computes `self + rhs`, returning `None` if overflow occurred.
    fn checked_add(self, rhs: Rhs) -> Option<Self::Output>;
}

/// Checked Subtraction
pub trait CheckedSub<Rhs = Self> {
    /// Output Type
    type Output;

    /// Checked integer subtraction. Computes `self - rhs`, returning `None` if overflow occurred.
    fn checked_sub(self, rhs: Rhs) -> Option<Self::Output>;
}

/// Checked Increment
pub trait CheckedIncrement {
    /// Increments `self` returning `None` if it would overflow.
    fn checked_increment(&mut self) -> Option<&mut Self>;
}

/// Checked Decrement
pub trait CheckedDecrement {
    /// Decrements `self` returning `None` if it would underflow.
    fn checked_decrement(&mut self) -> Option<&mut Self>;
}

/// Implements checked operations for the native integer `$type`.
macro_rules! impl_checked {
    ($($type:tt),* $(,)?) => {
        $(

            impl CheckedAdd for $type {
                type Output = Self;

                #[inline]
                fn checked_add(self, rhs: Self) -> Option<Self::Output> {
                    self.checked_add(rhs)
                }
            }

            impl CheckedSub for $type {
                type Output = Self;

                #[inline]
                fn checked_sub(self, rhs: Self) -> Option<Self::Output> {
                    self.checked_sub(rhs)
                }
            }

            impl CheckedIncrement for $type {
                #[inline]
                fn checked_increment(&mut self) -> Option<&mut Self> {
                    *self = self.checked_add(1)?;
                    Some(self)
                }
            }

            impl CheckedDecrement for $type {
                #[inline]
                fn checked_decrement(&mut self) -> Option<&mut Self> {
                    *self = self.checked_sub(1)?;
                    Some(self)
                }
            }
        )*
    };
}

impl_checked!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
