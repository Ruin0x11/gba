//! Module for all things relating to the Video RAM.
//!
//! Note that the GBA has six different display modes available, and the
//! _meaning_ of Video RAM depends on which display mode is active. In all
//! cases, Video RAM is **96kb** from `0x0600_0000` to `0x0601_7FFF`.
//!
//! # Safety
//!
//! Note that all possible bit patterns are technically allowed within Video
//! Memory. If you write the "wrong" thing into video memory you don't crash the
//! GBA, instead you just get graphical glitches (or perhaps nothing at all).
//! Accordingly, the "safe" functions here will check that you're in bounds, but
//! they won't bother to check that you've set the video mode they're designed
//! for.

pub(crate) use super::*;

pub mod affine;
pub mod bitmap;
pub mod text;

/// The start of VRAM.
///
/// Depending on what display mode is currently set there's different ways that
/// your program should interpret the VRAM space. Accordingly, we give the raw
/// value as just being a `usize`. Specific video mode types then wrap this as
/// being the correct thing.
pub const VRAM_BASE_USIZE: usize = 0x600_0000;

/// The character base blocks.
pub const CHAR_BASE_BLOCKS: VolAddressBlock<[u8; 0x4000]> = unsafe { VolAddressBlock::new_unchecked(VolAddress::new_unchecked(VRAM_BASE_USIZE), 6) };

/// The screen entry base blocks.
pub const SCREEN_BASE_BLOCKS: VolAddressBlock<[u8; 0x800]> =
  unsafe { VolAddressBlock::new_unchecked(VolAddress::new_unchecked(VRAM_BASE_USIZE), 32) };

newtype! {
  /// An 8x8 tile with 4bpp, packed as `u32` values for proper alignment.
  #[derive(Debug, Clone, Copy, Default)]
  Tile4bpp, pub [u32; 8], no frills
}

newtype! {
  /// An 8x8 tile with 8bpp, packed as `u32` values for proper alignment.
  #[derive(Debug, Clone, Copy, Default)]
  Tile8bpp, pub [u32; 16], no frills
}

/// Gives the specified charblock in 4bpp view.
pub fn get_4bpp_character_block(slot: usize) -> VolAddressBlock<Tile4bpp> {
  unsafe { VolAddressBlock::new_unchecked(CHAR_BASE_BLOCKS.index(slot).cast::<Tile4bpp>(), 512) }
}

/// Gives the specified charblock in 8bpp view.
pub fn get_8bpp_character_block(slot: usize) -> VolAddressBlock<Tile8bpp> {
  unsafe { VolAddressBlock::new_unchecked(CHAR_BASE_BLOCKS.index(slot).cast::<Tile8bpp>(), 256) }
}
