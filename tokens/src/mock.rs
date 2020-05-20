//! Mocks for the tokens module.

#![cfg(test)]

use frame_support::{impl_outer_event, impl_outer_origin, parameter_types};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, Perbill};
use sp_std::{cell::RefCell, marker::PhantomData};
use std::collections::HashMap;

use super::*;

impl_outer_origin! {
	pub enum Origin for Runtime {}
}

mod tokens {
	pub use crate::Event;
}

impl_outer_event! {
	pub enum TestEvent for Runtime {
		frame_system<T>,
		tokens<T>,
	}
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Runtime;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

type AccountId = u64;
impl frame_system::Trait for Runtime {
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = TestEvent;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type ModuleToIndex = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
}
pub type System = system::Module<Runtime>;

type CurrencyId = u32;
pub type Balance = u64;

thread_local! {
	static ACCUMULATED_DUST: RefCell<HashMap<CurrencyId, Balance>> = RefCell::new(HashMap::new());
}

pub struct MockDustRemoval<CurrencyId, Balance>(PhantomData<(CurrencyId, Balance)>);
impl MockDustRemoval<CurrencyId, Balance> {
	#[allow(dead_code)]
	pub fn accumulated_dust(currency_id: CurrencyId) -> Balance {
		ACCUMULATED_DUST.with(|v| *v.borrow().get(&currency_id).unwrap_or(&Zero::zero()))
	}
}
impl OnDustRemoval<CurrencyId, Balance> for MockDustRemoval<CurrencyId, Balance> {
	fn on_dust_removal(currency_id: CurrencyId, balance: Balance) {
		ACCUMULATED_DUST.with(|v| {
			let mut old_map = v.borrow().clone();
			if let Some(before) = old_map.get_mut(&currency_id) {
				*before += balance;
			} else {
				old_map.insert(currency_id, balance);
			};

			*v.borrow_mut() = old_map;
		});
	}
}

impl Trait for Runtime {
	type Event = TestEvent;
	type Balance = Balance;
	type Amount = i64;
	type CurrencyId = CurrencyId;
	type DustRemoval = MockDustRemoval<CurrencyId, Balance>;
}

pub type Tokens = Module<Runtime>;

pub const TEST_TOKEN_ID: CurrencyId = 1;
pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const ID_1: LockIdentifier = *b"1       ";
pub const ID_2: LockIdentifier = *b"2       ";

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn balances(mut self, endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>) -> Self {
		self.endowed_accounts = endowed_accounts;
		self
	}

	pub fn one_hundred_for_alice_n_bob(self) -> Self {
		self.balances(vec![(ALICE, TEST_TOKEN_ID, 100), (BOB, TEST_TOKEN_ID, 100)])
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		GenesisConfig::<Runtime> {
			endowed_accounts: self.endowed_accounts,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
