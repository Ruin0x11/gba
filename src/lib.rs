#![cfg_attr(not(test), no_std)]
#![feature(asm)]
#![feature(cfg_target_vendor)]
#![allow(clippy::cast_lossless)]
#![deny(clippy::float_arithmetic)]
//#![warn(missing_docs)]

//! This crate helps you write GBA ROMs.
//!
//! ## SAFETY POLICY
//!
//! Some parts of this crate are safe wrappers around unsafe operations. This is
//! good, and what you'd expect from a Rust crate.
//!
//! However, the safe wrappers all assume that you will _only_ attempt to
//! execute this crate on a GBA or in a GBA Emulator.
//!
//! **Do not** use this crate in programs that aren't running on the GBA. If you
//! do, it's a giant bag of Undefined Behavior.

pub(crate) use gba_proc_macro::phantom_fields;

/// Assists in defining a newtype wrapper over some base type.
///
/// Note that rustdoc and derives are all the "meta" stuff, so you can write all
/// of your docs and derives in front of your newtype in the same way you would
/// for a normal struct. Then the inner type to be wrapped it name.
///
/// The macro _assumes_ that you'll be using it to wrap numeric types and that
/// it's safe to have a `0` value, so it automatically provides a `const fn`
/// method for `new` that just wraps `0`. Also, it derives Debug, Clone, Copy,
/// Default, PartialEq, and Eq. If all this is not desired you can add `, no
/// frills` to the invocation.
///
/// Example:
/// ```
/// newtype! {
///   /// Records a particular key press combination.
///   KeyInput, u16
/// }
/// newtype! {
///   /// You can't derive most stuff above array size 32, so we add
///   /// the `, no frills` modifier to this one.
///   BigArray, [u8; 200], no frills
/// }
/// ```
#[macro_export]
macro_rules! newtype {
  ($(#[$attr:meta])* $new_name:ident, $v:vis $old_name:ty) => {
    $(#[$attr])*
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct $new_name($v $old_name);
    impl $new_name {
      /// A `const` "zero value" constructor
      pub const fn new() -> Self {
        $new_name(0)
      }
    }
  };
  ($(#[$attr:meta])* $new_name:ident, $v:vis $old_name:ty, no frills) => {
    $(#[$attr])*
    #[repr(transparent)]
    pub struct $new_name($v $old_name);
  };
}

/// Assists in defining a newtype that's an enum.
///
/// First give `NewType = OldType,`, then define the tags and their explicit
/// values with zero or more entries of `TagName = base_value,`. In both cases
/// you can place doc comments or other attributes directly on to the type
/// declaration or the tag declaration.
///
/// The generated enum will get an appropriate `repr` attribute as well as Debug, Clone, Copy,
///
/// Example:
/// ```
/// newtype_enum! {
///   /// The Foo
///   Foo = u16,
///   /// The Bar
///   Bar = 0,
///   /// The Zap
///   Zap = 1,
/// }
/// ```
#[macro_export]
macro_rules! newtype_enum {
  (
    $(#[$struct_attr:meta])*
    $new_name:ident = $old_name:ident,
    $($(#[$tag_attr:meta])* $tag_name:ident = $base_value:expr,)*
  ) => {
    $(#[$struct_attr])*
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr($old_name)]
    pub enum $new_name {
      $(
        $(#[$tag_attr])*
        $tag_name = $base_value,
      )*
    }
  };
}

pub mod base;
pub(crate) use self::base::*;

pub mod bios;

pub mod iwram;

pub mod ewram;

pub mod io;

pub mod palram;

pub mod vram;

pub mod oam;

pub mod rom;

pub mod sram;

pub mod mgba;

extern "C" {
  /// This marks the end of the `.data` and `.bss` sections in IWRAM.
  ///
  /// Memory in IWRAM _before_ this location is not free to use, you'll trash
  /// your globals and stuff. Memory here or after is freely available for use
  /// (careful that you don't run into your own stack of course).
  static __bss_end: u8;
}

newtype! {
  /// A color on the GBA is an RGB 5.5.5 within a `u16`
  #[derive(PartialOrd, Ord, Hash)]
  Color, u16
}

impl Color {
  /// Constructs a color from the channel values provided (should be 0..=31).
  ///
  /// No actual checks are performed, so illegal channel values can overflow
  /// into each other and produce an unintended color.
  pub const fn from_rgb(r: u16, g: u16, b: u16) -> Color {
    Color(b << 10 | g << 5 | r)
  }

  /// Does a left rotate of the bits.
  ///
  /// This has no particular meaning but is a wild way to cycle colors.
  pub const fn rotate_left(self, n: u32) -> Color {
    Color(self.0.rotate_left(n))
  }
}

//
// After here is totally unsorted nonsense
//

/// Performs unsigned divide and remainder, gives None if dividing by 0.
pub fn divrem_u32(numer: u32, denom: u32) -> Option<(u32, u32)> {
  // TODO: const this? Requires const if
  if denom == 0 {
    None
  } else {
    Some(unsafe { divrem_u32_unchecked(numer, denom) })
  }
}

/// Performs divide and remainder, no check for 0 division.
///
/// # Safety
///
/// If you call this with a denominator of 0 the result is implementation
/// defined (not literal UB) including but not limited to: an infinite loop,
/// panic on overflow, or incorrect output.
pub unsafe fn divrem_u32_unchecked(numer: u32, denom: u32) -> (u32, u32) {
  // TODO: const this? Requires const if
  if (numer >> 5) < denom {
    divrem_u32_simple(numer, denom)
  } else {
    divrem_u32_non_restoring(numer, denom)
  }
}

/// The simplest form of division. If N is too much larger than D this will be
/// extremely slow. If N is close enough to D then it will likely be faster than
/// the non_restoring form.
fn divrem_u32_simple(mut numer: u32, denom: u32) -> (u32, u32) {
  // TODO: const this? Requires const if
  let mut quot = 0;
  while numer >= denom {
    numer -= denom;
    quot += 1;
  }
  (quot, numer)
}

/// Takes a fixed quantity of time based on the bit width of the number (in this
/// case 32).
fn divrem_u32_non_restoring(numer: u32, denom: u32) -> (u32, u32) {
  // TODO: const this? Requires const if
  let mut r: i64 = numer as i64;
  let d: i64 = (denom as i64) << 32;
  let mut q: u32 = 0;
  let mut i = 1 << 31;
  while i > 0 {
    if r >= 0 {
      q |= i;
      r = 2 * r - d;
    } else {
      r = 2 * r + d;
    }
    i >>= 1;
  }
  q -= !q;
  if r < 0 {
    q -= 1;
    r += d;
  }
  r >>= 32;
  // TODO: remove this once we've done more checks here.
  debug_assert!(r >= 0);
  debug_assert!(r <= core::u32::MAX as i64);
  (q, r as u32)
}

/// Performs signed divide and remainder, gives None if dividing by 0 or
/// computing `MIN/-1`
pub fn divrem_i32(numer: i32, denom: i32) -> Option<(i32, i32)> {
  if denom == 0 || (numer == core::i32::MIN && denom == -1) {
    None
  } else {
    Some(unsafe { divrem_i32_unchecked(numer, denom) })
  }
}

/// Performs signed divide and remainder, no check for 0 division or `MIN/-1`.
///
/// # Safety
///
/// * If you call this with a denominator of 0 the result is implementation
///   defined (not literal UB) including but not limited to: an infinite loop,
///   panic on overflow, or incorrect output.
/// * If you call this with `MIN/-1` you'll get a panic in debug or just `MIN`
///   in release (which is incorrect), because of how twos-compliment works.
pub unsafe fn divrem_i32_unchecked(numer: i32, denom: i32) -> (i32, i32) {
  // TODO: const this? Requires const if
  let unsigned_numer = numer.abs() as u32;
  let unsigned_denom = denom.abs() as u32;
  let opposite_sign = (numer ^ denom) < 0;
  let (udiv, urem) = if (numer >> 5) < denom {
    divrem_u32_simple(unsigned_numer, unsigned_denom)
  } else {
    divrem_u32_non_restoring(unsigned_numer, unsigned_denom)
  };
  match (opposite_sign, numer < 0) {
    (true, true) => (-(udiv as i32), -(urem as i32)),
    (true, false) => (-(udiv as i32), urem as i32),
    (false, true) => (udiv as i32, -(urem as i32)),
    (false, false) => (udiv as i32, urem as i32),
  }
}

/*
#[cfg(test)]
mod tests {
  use super::*;
  use quickcheck::quickcheck;

  // We have an explicit property on the non_restoring division
  quickcheck! {
    fn divrem_u32_non_restoring_prop(num: u32, denom: u32) -> bool {
      if denom > 0 {
        divrem_u32_non_restoring(num, denom) == (num / denom, num % denom)
      } else {
        true
      }
    }
  }

  // We have an explicit property on the simple division
  quickcheck! {
    fn divrem_u32_simple_prop(num: u32, denom: u32) -> bool {
      if denom > 0 {
        divrem_u32_simple(num, denom) == (num / denom, num % denom)
      } else {
        true
      }
    }
  }

  // Test the u32 wrapper
  quickcheck! {
    fn divrem_u32_prop(num: u32, denom: u32) -> bool {
      if denom > 0 {
        divrem_u32(num, denom).unwrap() == (num / denom, num % denom)
      } else {
        divrem_u32(num, denom).is_none()
      }
    }
  }

  // test the i32 wrapper
  quickcheck! {
    fn divrem_i32_prop(num: i32, denom: i32) -> bool {
      if denom == 0 || num == core::i32::MIN && denom == -1 {
        divrem_i32(num, denom).is_none()
      } else {
        divrem_i32(num, denom).unwrap() == (num / denom, num % denom)
      }
    }
  }
}
*/
