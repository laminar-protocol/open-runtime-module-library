#[frame_support::pallet]
pub mod test_module {
	use frame_support::weights::PostDispatchInfo;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use orml_weight_meter;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(50_000)]
		#[orml_weight_meter::start_with(0)]
		pub fn expect_100(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			Self::put_100();

			Ok(PostDispatchInfo {
				actual_weight: Some(orml_weight_meter::used_weight()),
				pays_fee: Pays::Yes,
			})
		}

		#[pallet::weight(50_000)]
		#[orml_weight_meter::start_with(0)]
		pub fn expect_500(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			Self::put_100();
			Self::put_100();
			Self::put_100();
			Self::put_100();
			Self::put_100();

			Ok(PostDispatchInfo {
				actual_weight: Some(orml_weight_meter::used_weight()),
				pays_fee: Pays::Yes,
			})
		}

		#[pallet::weight(50_000)]
		#[orml_weight_meter::start_with(0)]
		pub fn expect_max_weight(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			Self::max_weight();
			Self::put_100();

			Ok(PostDispatchInfo {
				actual_weight: Some(orml_weight_meter::used_weight()),
				pays_fee: Pays::Yes,
			})
		}

		#[pallet::weight(50_000)]
		#[orml_weight_meter::start_with(0)]
		pub fn expect_100_or_200(origin: OriginFor<T>, branch: bool) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			if branch {
				Self::put_200();
			} else {
				Self::put_100();
			}

			Ok(PostDispatchInfo {
				actual_weight: Some(orml_weight_meter::used_weight()),
				pays_fee: Pays::Yes,
			})
		}

		#[pallet::weight(50_000)]
		#[orml_weight_meter::start_with(0)]
		pub fn nested_inner_methods(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			Self::put_300_nested();

			Ok(PostDispatchInfo {
				actual_weight: Some(orml_weight_meter::used_weight()),
				pays_fee: Pays::Yes,
			})
		}

		#[pallet::weight(50_000)]
		#[orml_weight_meter::start_with(200)]
		pub fn start_with_200(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			Ok(PostDispatchInfo {
				actual_weight: Some(orml_weight_meter::used_weight()),
				pays_fee: Pays::Yes,
			})
		}

		#[pallet::weight(50_000)]
		#[orml_weight_meter::start_with(200)]
		pub fn start_with_200_add_100(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			Self::put_100();

			Ok(PostDispatchInfo {
				actual_weight: Some(orml_weight_meter::used_weight()),
				pays_fee: Pays::Yes,
			})
		}

		#[pallet::weight(50_000)]
		#[orml_weight_meter::start_with(200)]
		pub fn start_with_200_branch(origin: OriginFor<T>, branch: bool) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			if branch {
				Self::put_200();
			} else {
				Self::put_100();
			}

			Ok(PostDispatchInfo {
				actual_weight: Some(orml_weight_meter::used_weight()),
				pays_fee: Pays::Yes,
			})
		}

		#[pallet::weight(50_000)]
		#[orml_weight_meter::start_with(u64::MAX)]
		pub fn start_with_max_weight(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			Self::put_100();

			Ok(PostDispatchInfo {
				actual_weight: Some(orml_weight_meter::used_weight()),
				pays_fee: Pays::Yes,
			})
		}
	}

	impl<T: Config> Pallet<T> {
		#[orml_weight_meter::weight(100)]
		fn put_100() {
			let something = Self::something();

			if let Some(v) = something {
				Something::<T>::put(v.checked_add(100).unwrap());
			} else {
				Something::<T>::put(100);
			}
		}

		#[orml_weight_meter::weight(200)]
		fn put_200() {
			let something = Self::something();

			if let Some(v) = something {
				Something::<T>::put(v.checked_add(200).unwrap());
			} else {
				Something::<T>::put(100);
			}
		}

		#[orml_weight_meter::weight(200)]
		fn put_300_nested() {
			Self::put_100();
		}

		#[orml_weight_meter::weight(u64::MAX)]
		fn max_weight() {
			return;
		}
	}
}

use frame_support::sp_runtime::traits::IdentityLookup;
use sp_runtime::testing::{Header, H256};

pub type BlockNumber = u64;

frame_support::parameter_types! {
		pub const BlockHashCount: u64 = 250;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;
type Balance = u128;

impl frame_system::Config for Runtime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

frame_support::parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Runtime>;
	type MaxLocks = ();
	type WeightInfo = ();
}

impl test_module::Config for Runtime {}

frame_support::construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		TestModule: test_module::{Pallet, Call, Storage},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

pub struct ExtBuilder();

impl Default for ExtBuilder {
	fn default() -> Self {
		Self()
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![(100, 100_000)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
