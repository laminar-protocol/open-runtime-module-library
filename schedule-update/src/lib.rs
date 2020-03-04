#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::Weight,
	traits::Get,
	weights::{DispatchClass, GetDispatchInfo},
	Parameter,
};
use frame_system::{self as system, ensure_root, ensure_signed};
use sp_runtime::{
	traits::{Dispatchable, One},
	DispatchError, RuntimeDebug,
};
use sp_std::{prelude::*, result};

mod mock;
mod tests;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum DelayedDispatchTime<BlockNumber> {
	At(BlockNumber),
	After(BlockNumber),
}

//#[derive(Encode, Decode, Clone, Eq, PartialEq)]
//pub struct DelayedDispatch<T: Trait> {
//	origin: T::Origin,
//	call: CallOf<T>,
//	when: T::BlockNumber,
//}

type DispatchId = u32;
type CallOf<T> = <T as Trait>::Call;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Call: Parameter + Default + Dispatchable<Origin = Self::Origin> + GetDispatchInfo;
	type MaxScheduleDispatchWeight: Get<Weight>;
}

decl_event!(
	/// Event for schedule-update module.
	pub enum Event<T> where
		<T as frame_system::Trait>::BlockNumber,
	<T as frame_system::Trait>::AccountId,
	{
		/// Add schedule dispatch success (AccountId, DispatchId, BlockNumber)
		ScheduleDispatch(AccountId, DispatchId, BlockNumber),
		/// Cancel deplayed dispatch success (DispatchId)
		CancelDeplayedDispatch(DispatchId),
		///
		ScheduleDispatchSuccess(AccountId, DispatchId, BlockNumber),
		///
		ScheduleDispatchFail(AccountId, DispatchId, DispatchError),
	}
);

decl_error! {
	/// Error for schedule-update module.
	pub enum Error for Module<T: Trait> {
		CannotGetNextId,
		NoPermission,
		DispatchNotExisted,
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as ScheduleUpdate {
		pub NextId get(fn next_id): DispatchId;
		pub DelayedNormalDispatches get(fn delayed_normal_dispatches): double_map hasher(blake2_256) T::BlockNumber, hasher(blake2_256) DispatchId => (T::AccountId, CallOf<T>, DispatchId);
		pub DelayedOperationalDispatches get(fn delayed_operational_dispatches): double_map hasher(blake2_256) T::BlockNumber, hasher(blake2_256) DispatchId => (T::AccountId, CallOf<T>, DispatchId);
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		const MaxScheduleDispatchWeight: Weight = T::MaxScheduleDispatchWeight::get();

		pub fn schedule_dispatch(origin, call: CallOf<T>, when: DelayedDispatchTime<T::BlockNumber>) {
			let who = ensure_signed(origin)?;

			let id = Self::_get_next_id()?;

			let block_number = match when {
				DelayedDispatchTime::At(block_number) => {
					block_number
				},
				DelayedDispatchTime::After(block_count) => {
					<frame_system::Module<T>>::block_number() + block_count
				},
			};

			match call.get_dispatch_info().class {
				DispatchClass::Normal => {
					<DelayedNormalDispatches<T>>::insert(block_number, id, (who.clone(), call, id));
				},
				DispatchClass::Operational => {
					<DelayedOperationalDispatches<T>>::insert(block_number, id, (who.clone(), call, id));
				},
			}
			Self::deposit_event(RawEvent::ScheduleDispatch(who, id, block_number));
		}

		pub fn cancel_deplayed_dispatch(origin, at: T::BlockNumber, id: DispatchId) {
			let is_root = ensure_root(origin.clone()).is_ok();

			if <DelayedNormalDispatches<T>>::contains_key(at, id) {
				if !is_root {
					let w = ensure_signed(origin)?;
					let (who, _, _) = <DelayedNormalDispatches<T>>::get(at, id);
					if w != who {
						return Err(Error::<T>::NoPermission.into());
					}
				}
				<DelayedNormalDispatches<T>>::remove(at, id);
			} else if <DelayedOperationalDispatches<T>>::contains_key(at, id) {
				if !is_root {
					let w = ensure_signed(origin)?;
					let (who, _, _) = <DelayedOperationalDispatches<T>>::get(at, id);
					if w != who {
						return Err(Error::<T>::NoPermission.into());
					}
				}
				<DelayedOperationalDispatches<T>>::remove(at, id);
			} else {
				return Err(Error::<T>::DispatchNotExisted.into());
			}
			Self::deposit_event(RawEvent::CancelDeplayedDispatch(id));
		}

		fn on_initialize(now: T::BlockNumber) {
			let mut weight: Weight = 0;
			let total_weight = T::MaxScheduleDispatchWeight::get();

			let operational_dispatches = <DelayedOperationalDispatches<T>>::iter_prefix(now);
			operational_dispatches.for_each(|(who, call, id)| {
				weight += call.get_dispatch_info().weight;
				if weight > total_weight {
					return;
				}

				let result = call.dispatch(frame_system::RawOrigin::Signed(who.clone()).into());
				if let Err(e) = result {
					Self::deposit_event(RawEvent::ScheduleDispatchFail(who, id, e));
				} else {
					Self::deposit_event(RawEvent::ScheduleDispatchSuccess(who, id, now));
				}
				<DelayedOperationalDispatches<T>>::remove(now, id);
			});

			let normal_dispatches = <DelayedNormalDispatches<T>>::iter_prefix(now);
			normal_dispatches.for_each(|(who, call, id)| {
				weight += call.get_dispatch_info().weight;
				if weight > total_weight {
					return;
				}

				let result = call.dispatch(frame_system::RawOrigin::Signed(who.clone()).into());
				if let Err(e) = result {
					Self::deposit_event(RawEvent::ScheduleDispatchFail(who, id, e));
				} else {
					Self::deposit_event(RawEvent::ScheduleDispatchSuccess(who, id, now));
				}
				<DelayedNormalDispatches<T>>::remove(now, id);
			});

			let operational_dispatches = <DelayedOperationalDispatches<T>>::iter_prefix(now);
			operational_dispatches.for_each(|(who, call, id)| {
				<DelayedOperationalDispatches<T>>::insert(now + One::one(), id, (who, call, id));
				<DelayedOperationalDispatches<T>>::remove(now, id);
			});

			let normal_dispatches = <DelayedNormalDispatches<T>>::iter_prefix(now);
			normal_dispatches.for_each(|(who, call, id)| {
				<DelayedNormalDispatches<T>>::insert(now + One::one(), id, (who, call, id));
				<DelayedNormalDispatches<T>>::remove(now, id);
			});
		}
	}
}

impl<T: Trait> Module<T> {
	fn _get_next_id() -> result::Result<DispatchId, Error<T>> {
		let id = Self::next_id();
		let next_id = id.checked_add(One::one()).ok_or(Error::<T>::CannotGetNextId)?;
		NextId::put(next_id);
		Ok(id)
	}
}
