//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Spontra;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn set_payer_key() {
		// TODO: Add benchmarking logic.
	}

	#[benchmark]
	fn sponsor_call() {
		// TODO: Add benchmarking logic.
	}

	#[benchmark]
	fn unsponsor_call() {
		// TODO: Add benchmarking logic.
	}

	impl_benchmark_test_suite!(Spontra, crate::mock::new_test_ext(), crate::mock::Test);
}
