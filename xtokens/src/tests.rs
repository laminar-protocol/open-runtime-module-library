#![cfg(test)]

use super::*;
use cumulus_primitives_core::ParaId;
use frame_support::{assert_ok, traits::Currency};
use mock::*;
use orml_traits::MultiCurrency;
use polkadot_parachain::primitives::{AccountIdConversion, Sibling};
use sp_runtime::AccountId32;
use xcm::v0::{Junction, NetworkId};
use xcm_simulator::TestExt;

fn relay_chain_para_a_account() -> AccountId32 {
	ParaId::from(1).into_account()
}

fn relay_chain_para_b_account() -> AccountId32 {
	ParaId::from(2).into_account()
}

fn para_a_account() -> AccountId32 {
	use sp_runtime::traits::AccountIdConversion;
	Sibling::from(1).into_account()
}

fn para_b_account() -> AccountId32 {
	use sp_runtime::traits::AccountIdConversion;
	Sibling::from(2).into_account()
}

#[test]
fn send_relay_chain_asset_to_relay_chain() {
	TestNetwork::reset();

	MockRelay::execute_with(|| {
		let _ = RelayBalances::deposit_creating(&relay_chain_para_a_account(), 100);
	});

	ParaA::execute_with(|| {
		assert_ok!(ParaAXtokens::transfer(
			Some(ALICE).into(),
			CurrencyId::R,
			30,
			(
				Junction::Parent,
				Junction::AccountId32 {
					network: NetworkId::Polkadot,
					id: BOB.into(),
				},
			)
				.into(),
		));
		assert_eq!(ParaATokens::free_balance(CurrencyId::R, &ALICE), 70);
	});

	MockRelay::execute_with(|| {
		assert_eq!(RelayBalances::free_balance(&relay_chain_para_a_account()), 70);
		assert_eq!(RelayBalances::free_balance(&BOB), 30);
	});
}

#[test]
fn send_relay_chain_asset_to_sibling() {
	TestNetwork::reset();

	MockRelay::execute_with(|| {
		let _ = RelayBalances::deposit_creating(&relay_chain_para_a_account(), 100);
	});

	ParaA::execute_with(|| {
		assert_ok!(ParaAXtokens::transfer(
			Some(ALICE).into(),
			CurrencyId::R,
			30,
			(
				Junction::Parent,
				Junction::Parachain { id: 2 },
				Junction::AccountId32 {
					network: NetworkId::Any,
					id: BOB.into(),
				},
			)
				.into(),
		));
		assert_eq!(ParaATokens::free_balance(CurrencyId::R, &ALICE), 70);
	});

	use xcm_simulator::relay_chain;

	MockRelay::execute_with(|| {
		assert_eq!(RelayBalances::free_balance(&relay_chain_para_a_account()), 70);
		assert_eq!(RelayBalances::free_balance(&relay_chain_para_b_account()), 30);
	});

	ParaB::execute_with(|| {
		assert_eq!(ParaBTokens::free_balance(CurrencyId::R, &BOB), 30);
	});
}

#[test]
fn send_sibling_asset_to_reserve_sibling() {
	TestNetwork::reset();

	ParaA::execute_with(|| {
		assert_ok!(ParaATokens::deposit(CurrencyId::B, &ALICE, 100));
	});

	ParaB::execute_with(|| {
		assert_ok!(ParaBTokens::deposit(CurrencyId::B, &para_a_account(), 100));
	});

	ParaA::execute_with(|| {
		assert_ok!(ParaAXtokens::transfer(
			Some(ALICE).into(),
			CurrencyId::B,
			30,
			(
				Junction::Parent,
				Junction::Parachain { id: 2 },
				Junction::AccountId32 {
					network: NetworkId::Any,
					id: BOB.into(),
				},
			)
				.into(),
		));

		assert_eq!(ParaATokens::free_balance(CurrencyId::B, &ALICE), 70);
	});

	ParaB::execute_with(|| {
		assert_eq!(ParaBTokens::free_balance(CurrencyId::B, &para_a_account()), 70);
		assert_eq!(ParaBTokens::free_balance(CurrencyId::B, &BOB), 30);
	});
}

#[test]
fn send_sibling_asset_to_non_reserve_sibling() {
	//TODO: add another parachain and test
}

#[test]
fn send_self_parachain_asset_to_sibling() {
	TestNetwork::reset();

	ParaA::execute_with(|| {
		assert_ok!(ParaATokens::deposit(CurrencyId::A, &ALICE, 100));

		assert_ok!(ParaAXtokens::transfer(
			Some(ALICE).into(),
			CurrencyId::A,
			30,
			(
				Junction::Parent,
				Junction::Parachain { id: 2 },
				Junction::AccountId32 {
					network: NetworkId::Any,
					id: BOB.into(),
				},
			)
				.into(),
		));

		assert_eq!(ParaATokens::free_balance(CurrencyId::A, &ALICE), 70);
		assert_eq!(ParaATokens::free_balance(CurrencyId::A, &para_b_account()), 30);
	});

	// fix: untrusted reserve location
	ParaB::execute_with(|| {
		para_b::System::events().iter().for_each(|r| {
			println!(">>> {:?}", r.event);
		});
		assert_eq!(ParaBTokens::free_balance(CurrencyId::A, &BOB), 30);
	});
}
