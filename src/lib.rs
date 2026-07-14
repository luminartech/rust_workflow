//! Fixture crate for smoke-testing the reusable `rust-ci.yml` workflow.
//!
//! This crate exists so the workflow can run against real Rust code in this
//! repository. It is deliberately tiny, `no_std`-compatible, and never
//! published.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

/// Computes the Fletcher-16 checksum of `data`.
#[must_use]
pub fn fletcher16(data: &[u8]) -> u16 {
    let mut sum1: u16 = 0;
    let mut sum2: u16 = 0;
    for &byte in data {
        sum1 = (sum1 + u16::from(byte)) % 255;
        sum2 = (sum2 + sum1) % 255;
    }
    (sum2 << 8) | sum1
}

/// Computes the Fletcher-16 checksum of each `chunk_size`-byte chunk of
/// `data`. The final chunk may be shorter. Returns an empty vector when
/// `chunk_size` is zero.
#[cfg(feature = "alloc")]
#[must_use]
pub fn chunk_checksums(data: &[u8], chunk_size: usize) -> alloc::vec::Vec<u16> {
    if chunk_size == 0 {
        return alloc::vec::Vec::new();
    }
    data.chunks(chunk_size).map(fletcher16).collect()
}

#[cfg(test)]
mod tests {
    use super::{chunk_checksums, fletcher16};

    #[test]
    fn known_vectors() {
        assert_eq!(fletcher16(b""), 0);
        assert_eq!(fletcher16(b"abcde"), 0xC8F0);
        assert_eq!(fletcher16(b"abcdef"), 0x2057);
        assert_eq!(fletcher16(b"abcdefgh"), 0x0627);
    }

    #[test]
    fn zero_chunk_size_yields_no_checksums() {
        assert!(chunk_checksums(b"abc", 0).is_empty());
    }

    #[test]
    fn chunks_cover_all_input() {
        assert_eq!(
            chunk_checksums(b"abcdef", 2),
            vec![fletcher16(b"ab"), fletcher16(b"cd"), fletcher16(b"ef")]
        );
    }

    proptest::proptest! {
        #[test]
        fn prop_chunk_count_matches(data: Vec<u8>, chunk_size in 1_usize..64) {
            let checksums = chunk_checksums(&data, chunk_size);
            proptest::prop_assert_eq!(checksums.len(), data.len().div_ceil(chunk_size));
        }

        #[test]
        fn prop_low_byte_is_byte_sum_mod_255(data: Vec<u8>) {
            let expected = data.iter().map(|&b| u32::from(b)).sum::<u32>() % 255;
            proptest::prop_assert_eq!(u32::from(fletcher16(&data) & 0xFF), expected);
        }
    }
}
