//! Unit tests for the vesting module.

#![cfg(test)]

use super::*;
use frame_support::{assert_err, assert_noop, assert_ok, traits::WithdrawReason};
use mock::{ExtBuilder, Origin, PalletBalances, Runtime, System, TestEvent, Vesting, ALICE, BOB};
use pallet_balances::{BalanceLock, Reasons};

#[test]
fn add_vesting_schedule_works() {
	ExtBuilder::default().one_hundred_for_alice().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 1u32,
			per_period: 100u64,
		};
		assert_ok!(Vesting::add_vesting_schedule(
			Origin::signed(ALICE),
			BOB,
			schedule.clone()
		));
		assert_eq!(Vesting::vesting_schedules(&BOB), vec![schedule.clone()]);

		let vested_event = TestEvent::vesting(RawEvent::VestingScheduleAdded(ALICE, BOB, schedule));
		assert!(System::events().iter().any(|record| record.event == vested_event));
	});
}

#[test]
fn add_new_vesting_schedule_merges_with_current_locked_balance_and_until() {
	ExtBuilder::default().one_hundred_for_alice().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 2u32,
			per_period: 10u64,
		};
		assert_ok!(Vesting::add_vesting_schedule(Origin::signed(ALICE), BOB, schedule));

		System::set_block_number(12);

		let another_schedule = VestingSchedule {
			start: 10u64,
			period: 13u64,
			period_count: 1u32,
			per_period: 7u64,
		};
		assert_ok!(Vesting::add_vesting_schedule(
			Origin::signed(ALICE),
			BOB,
			another_schedule
		));

		assert_eq!(
			PalletBalances::locks(&BOB).pop(),
			Some(BalanceLock {
				id: VESTING_LOCK_ID,
				amount: 17u64,
				reasons: Reasons::All,
			})
		);
	});
}

#[test]
fn cannot_use_fund_if_not_claimed() {
	ExtBuilder::default().one_hundred_for_alice().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 10u64,
			period: 10u64,
			period_count: 1u32,
			per_period: 50u64,
		};
		assert_ok!(Vesting::add_vesting_schedule(
			Origin::signed(ALICE),
			BOB,
			schedule.clone()
		));
		assert!(PalletBalances::ensure_can_withdraw(&BOB, 1, WithdrawReason::Transfer.into(), 49).is_err());
	});
}

#[test]
fn add_vesting_schedule_fails_if_zero_period_or_count() {
	ExtBuilder::default().one_hundred_for_alice().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 1u64,
			period: 0u64,
			period_count: 1u32,
			per_period: 100u64,
		};
		assert_err!(
			Vesting::add_vesting_schedule(Origin::signed(ALICE), BOB, schedule.clone()),
			Error::<Runtime>::ZeroVestingPeriod
		);

		let schedule = VestingSchedule {
			start: 1u64,
			period: 1u64,
			period_count: 0u32,
			per_period: 100u64,
		};
		assert_err!(
			Vesting::add_vesting_schedule(Origin::signed(ALICE), BOB, schedule.clone()),
			Error::<Runtime>::ZeroVestingPeriodCount
		);
	});
}

#[test]
fn add_vesting_schedule_fails_if_transfer_err() {
	ExtBuilder::default().one_hundred_for_alice().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 1u64,
			period: 1u64,
			period_count: 1u32,
			per_period: 100u64,
		};
		assert_err!(
			Vesting::add_vesting_schedule(Origin::signed(BOB), ALICE, schedule.clone()),
			pallet_balances::Error::<Runtime, _>::InsufficientBalance,
		);
	});
}

#[test]
fn add_vesting_schedule_fails_if_overflow() {
	ExtBuilder::default().one_hundred_for_alice().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 1u64,
			period: 1u64,
			period_count: 2u32,
			per_period: u64::max_value(),
		};
		assert_err!(
			Vesting::add_vesting_schedule(Origin::signed(ALICE), BOB, schedule),
			Error::<Runtime>::NumOverflow
		);

		let another_schedule = VestingSchedule {
			start: u64::max_value(),
			period: 1u64,
			period_count: 2u32,
			per_period: 1u64,
		};
		assert_err!(
			Vesting::add_vesting_schedule(Origin::signed(ALICE), BOB, another_schedule),
			Error::<Runtime>::NumOverflow
		);
	});
}

#[test]
fn claim_works() {
	ExtBuilder::default().one_hundred_for_alice().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 2u32,
			per_period: 10u64,
		};
		assert_ok!(Vesting::add_vesting_schedule(
			Origin::signed(ALICE),
			BOB,
			schedule.clone()
		));

		System::set_block_number(11);
		// remain locked if not claimed
		assert!(PalletBalances::transfer(Origin::signed(BOB), ALICE, 10).is_err());
		// unlocked after claiming
		assert_ok!(Vesting::claim(Origin::signed(BOB)));
		assert_ok!(PalletBalances::transfer(Origin::signed(BOB), ALICE, 10));
		// more are still locked
		assert!(PalletBalances::transfer(Origin::signed(BOB), ALICE, 1).is_err());

		System::set_block_number(21);
		// claim more
		assert_ok!(Vesting::claim(Origin::signed(BOB)));
		assert_ok!(PalletBalances::transfer(Origin::signed(BOB), ALICE, 10));
		// all used up
		assert_eq!(PalletBalances::free_balance(BOB), 0);

		// no locks anymore
		assert_eq!(PalletBalances::locks(&BOB), vec![]);
	});
}

#[test]
fn update_vesting_schedules_works() {
	ExtBuilder::default().one_hundred_for_alice().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 2u32,
			per_period: 10u64,
		};
		assert_ok!(Vesting::add_vesting_schedule(
			Origin::signed(ALICE),
			BOB,
			schedule.clone()
		));

		let updated_schedule = VestingSchedule {
			start: 0u64,
			period: 20u64,
			period_count: 2u32,
			per_period: 10u64,
		};
		assert_ok!(Vesting::update_vesting_schedules(
			Origin::ROOT,
			BOB,
			vec![updated_schedule]
		));

		System::set_block_number(11);
		assert_ok!(Vesting::claim(Origin::signed(BOB)));
		assert!(PalletBalances::transfer(Origin::signed(BOB), ALICE, 1).is_err());

		System::set_block_number(21);
		assert_ok!(Vesting::claim(Origin::signed(BOB)));
		assert_ok!(PalletBalances::transfer(Origin::signed(BOB), ALICE, 10));
	});
}

#[test]
fn update_vesting_schedules_fails_if_unexpected_existing_locks() {
	ExtBuilder::default().one_hundred_for_alice().build().execute_with(|| {
		assert_ok!(PalletBalances::transfer(Origin::signed(ALICE), BOB, 1));
		PalletBalances::set_lock(*b"prelocks", &BOB, 0u64, WithdrawReasons::all());
	});
}
