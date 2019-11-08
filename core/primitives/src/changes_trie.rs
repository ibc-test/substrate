// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Substrate changes trie configuration.

#[cfg(any(feature = "std", test))]
use serde::{Serialize, Deserialize};
use codec::{Encode, Decode};
use num_traits::Zero;

/// Substrate changes trie configuration.
#[cfg_attr(any(feature = "std", test), derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default, Encode, Decode)]
pub struct ChangesTrieConfiguration {
	/// Interval (in blocks) at which level1-digests are created. Digests are not
	/// created when this is less or equal to 1.
	pub digest_interval: u32,
	/// Maximal number of digest levels in hierarchy. 0 means that digests are not
	/// created at all (even level1 digests). 1 means only level1-digests are created.
	/// 2 means that every digest_interval^2 there will be a level2-digest, and so on.
	/// Please ensure that maximum digest interval (i.e. digest_interval^digest_levels)
	/// is within `u32` limits. Otherwise you'll never see digests covering such intervals
	/// && maximal digests interval will be truncated to the last interval that fits
	/// `u32` limits.
	pub digest_levels: u32,
}

impl ChangesTrieConfiguration {
	/// Create new configuration given digest interval and levels.
	pub fn new(digest_interval: u32, digest_levels: u32) -> Self {
		Self { digest_interval, digest_levels }
	}

	/// Is digest build enabled?
	pub fn is_digest_build_enabled(&self) -> bool {
		self.digest_interval > 1 && self.digest_levels > 0
	}

	/// Do we need to build digest at given block?
	pub fn is_digest_build_required_at_block<Number>(
		&self,
		zero: Number,
		block: Number,
	) -> bool
		where
			Number: From<u32> + PartialEq +
			::rstd::ops::Rem<Output=Number> + ::rstd::ops::Sub<Output=Number> +
			::rstd::cmp::PartialOrd + Zero,
	{
		block > zero
			&& self.is_digest_build_enabled()
			&& ((block - zero) % self.digest_interval.into()).is_zero()
	}

	/// Returns max digest interval. One if digests are not created at all.
	pub fn max_digest_interval(&self) -> u32 {
		if !self.is_digest_build_enabled() {
			return 1;
		}

		// we'll get >1 loop iteration only when bad configuration parameters are selected
		let mut current_level = self.digest_levels;
		loop {
			if let Some(max_digest_interval) = self.digest_interval.checked_pow(current_level) {
				return max_digest_interval;
			}

			current_level = current_level - 1;
		}
	}

	/// Returns max level digest block number that has been created at block <= passed block number.
	///
	/// Returns None if digests are not created at all.
	pub fn prev_max_level_digest_block<Number>(
		&self,
		zero: Number,
		block: Number,
	) -> Option<Number>
		where
			Number: Clone + From<u32> + PartialOrd + PartialEq +
			::rstd::ops::Add<Output=Number> + ::rstd::ops::Sub<Output=Number> +
			::rstd::ops::Div<Output=Number> + ::rstd::ops::Mul<Output=Number> + Zero,
	{
		if block <= zero {
			return None;
		}

		let (next_begin, next_end) = self.next_max_level_digest_range(zero.clone(), block.clone())?;

		// if 'next' digest includes our block, then it is a also a previous digest
		if next_end == block {
			return Some(block);
		}

		// if previous digest ends at zero block, then there are no previous digest
		let prev_end = next_begin - 1.into();
		if prev_end == zero {
			None
		} else {
			Some(prev_end)
		}
	}

	/// Returns max level digest blocks range (inclusive) which includes passed block.
	///
	/// Returns None if digests are not created at all.
	/// It will return the first max-level digest if block is <= zero.
	pub fn next_max_level_digest_range<Number>(
		&self,
		zero: Number,
		mut block: Number,
	) -> Option<(Number, Number)>
		where
			Number: Clone + From<u32> + PartialOrd + PartialEq +
			::rstd::ops::Add<Output=Number> + ::rstd::ops::Sub<Output=Number> +
			::rstd::ops::Div<Output=Number> + ::rstd::ops::Mul<Output=Number>,
	{
		if !self.is_digest_build_enabled() {
			return None;
		}

		if block <= zero {
			block = zero.clone() + 1.into();
		}

		let max_digest_interval: Number = self.max_digest_interval().into();
		let max_digests_since_zero = (block.clone() - zero.clone()) / max_digest_interval.clone();
		if max_digests_since_zero == 0.into() {
			return Some((zero.clone() + 1.into(), zero + max_digest_interval));
		}
		let last_max_digest_block = zero + max_digests_since_zero * max_digest_interval.clone();
		Some(if block == last_max_digest_block {
			(block.clone() - max_digest_interval + 1.into(), block)
		} else {
			(last_max_digest_block.clone() + 1.into(), last_max_digest_block + max_digest_interval)
		})
	}

	/// Returns Some if digest must be built at given block number.
	/// The tuple is:
	/// (
	///  digest level
	///  digest interval (in blocks)
	///  step between blocks we're interested in when digest is built
	/// )
	pub fn digest_level_at_block<Number>(&self, zero: Number, block: Number) -> Option<(u32, u32, u32)>
		where
			Number: Clone + From<u32> + PartialEq +
			::rstd::ops::Rem<Output=Number> + ::rstd::ops::Sub<Output=Number> +
			::rstd::cmp::PartialOrd + Zero,
	{
		if !self.is_digest_build_required_at_block(zero.clone(), block.clone()) {
			return None;
		}

		let relative_block = block - zero;
		let mut digest_interval = self.digest_interval;
		let mut current_level = 1u32;
		let mut digest_step = 1u32;
		while current_level < self.digest_levels {
			let new_digest_interval = match digest_interval.checked_mul(self.digest_interval) {
				Some(new_digest_interval) if (relative_block.clone() % new_digest_interval.into()).is_zero()
					=> new_digest_interval,
				_ => break,
			};

			digest_step = digest_interval;
			digest_interval = new_digest_interval;
			current_level = current_level + 1;
		}

		Some((
			current_level,
			digest_interval,
			digest_step,
		))
	}
}

#[cfg(test)]
mod tests {
	use super::ChangesTrieConfiguration;

	fn config(interval: u32, levels: u32) -> ChangesTrieConfiguration {
		ChangesTrieConfiguration {
			digest_interval: interval,
			digest_levels: levels,
		}
	}

	#[test]
	fn is_digest_build_enabled_works() {
		assert!(!config(0, 100).is_digest_build_enabled());
		assert!(!config(1, 100).is_digest_build_enabled());
		assert!(config(2, 100).is_digest_build_enabled());
		assert!(!config(100, 0).is_digest_build_enabled());
		assert!(config(100, 1).is_digest_build_enabled());
	}

	#[test]
	fn is_digest_build_required_at_block_works() {
		fn test_with_zero(zero: u64) {
			assert!(!config(8, 4).is_digest_build_required_at_block(zero, zero + 0u64));
			assert!(!config(8, 4).is_digest_build_required_at_block(zero, zero + 1u64));
			assert!(!config(8, 4).is_digest_build_required_at_block(zero, zero + 2u64));
			assert!(!config(8, 4).is_digest_build_required_at_block(zero, zero + 4u64));
			assert!(config(8, 4).is_digest_build_required_at_block(zero, zero + 8u64));
			assert!(!config(8, 4).is_digest_build_required_at_block(zero, zero + 9u64));
			assert!(config(8, 4).is_digest_build_required_at_block(zero, zero + 64u64));
			assert!(config(8, 4).is_digest_build_required_at_block(zero, zero + 64u64));
			assert!(config(8, 4).is_digest_build_required_at_block(zero, zero + 512u64));
			assert!(config(8, 4).is_digest_build_required_at_block(zero, zero + 4096u64));
			assert!(!config(8, 4).is_digest_build_required_at_block(zero, zero + 4103u64));
			assert!(config(8, 4).is_digest_build_required_at_block(zero, zero + 4104u64));
			assert!(!config(8, 4).is_digest_build_required_at_block(zero, zero + 4108u64));
		}

		test_with_zero(0);
		test_with_zero(8);
		test_with_zero(17);
	}

	#[test]
	fn digest_level_at_block_works() {
		fn test_with_zero(zero: u64) {
			assert_eq!(config(8, 4).digest_level_at_block(zero, zero + 0u64), None);
			assert_eq!(config(8, 4).digest_level_at_block(zero, zero + 7u64), None);
			assert_eq!(config(8, 4).digest_level_at_block(zero, zero + 63u64), None);
			assert_eq!(config(8, 4).digest_level_at_block(zero, zero + 8u64), Some((1, 8, 1)));
			assert_eq!(config(8, 4).digest_level_at_block(zero, zero + 64u64), Some((2, 64, 8)));
			assert_eq!(config(8, 4).digest_level_at_block(zero, zero + 512u64), Some((3, 512, 64)));
			assert_eq!(config(8, 4).digest_level_at_block(zero, zero + 4096u64), Some((4, 4096, 512)));
			assert_eq!(config(8, 4).digest_level_at_block(zero, zero + 4112u64), Some((1, 8, 1)));
		}

		test_with_zero(0);
		test_with_zero(8);
		test_with_zero(17);
	}

	#[test]
	fn max_digest_interval_works() {
		assert_eq!(config(0, 0).max_digest_interval(), 1);
		assert_eq!(config(2, 2).max_digest_interval(), 4);
		assert_eq!(config(8, 4).max_digest_interval(), 4096);
		assert_eq!(config(::std::u32::MAX, 1024).max_digest_interval(), ::std::u32::MAX);
	}

	#[test]
	fn next_max_level_digest_range_works() {
		assert_eq!(config(0, 0).next_max_level_digest_range(0u64, 16), None);
		assert_eq!(config(1, 1).next_max_level_digest_range(0u64, 16), None);
		assert_eq!(config(2, 1).next_max_level_digest_range(0u64, 16), Some((15, 16)));
		assert_eq!(config(4, 1).next_max_level_digest_range(0u64, 16), Some((13, 16)));
		assert_eq!(config(32, 1).next_max_level_digest_range(0u64, 16), Some((1, 32)));
		assert_eq!(config(2, 3).next_max_level_digest_range(0u64, 10), Some((9, 16)));
		assert_eq!(config(2, 3).next_max_level_digest_range(0u64, 8), Some((1, 8)));
		assert_eq!(config(2, 1).next_max_level_digest_range(1u64, 1), Some((2, 3)));
		assert_eq!(config(2, 2).next_max_level_digest_range(7u64, 9), Some((8, 11)));

		assert_eq!(config(2, 2).next_max_level_digest_range(7u64, 5), Some((8, 11)));
	}

	#[test]
	fn prev_max_level_digest_block_works() {
		assert_eq!(config(0, 0).prev_max_level_digest_block(0u64, 16), None);
		assert_eq!(config(1, 1).prev_max_level_digest_block(0u64, 16), None);
		assert_eq!(config(2, 1).prev_max_level_digest_block(0u64, 16), Some(16));
		assert_eq!(config(4, 1).prev_max_level_digest_block(0u64, 16), Some(16));

		assert_eq!(config(4, 2).prev_max_level_digest_block(0u64, 16), Some(16));
		assert_eq!(config(4, 2).prev_max_level_digest_block(0u64, 17), Some(16));
		assert_eq!(config(4, 2).prev_max_level_digest_block(0u64, 33), Some(32));

		assert_eq!(config(32, 1).prev_max_level_digest_block(0u64, 16), None);
		assert_eq!(config(2, 3).prev_max_level_digest_block(0u64, 10), Some(8));
		assert_eq!(config(2, 3).prev_max_level_digest_block(0u64, 8), Some(8));
		assert_eq!(config(2, 2).prev_max_level_digest_block(7u64, 8), None);

		assert_eq!(config(2, 2).prev_max_level_digest_block(7u64, 5), None);
	}
}
