#![cfg_attr(not(feature = "std"), no_std)]

use rstd::{marker, result};
use srml_support::{decl_error, decl_event, decl_module, decl_storage, ensure, traits::Get, Parameter};
// FIXME: `srml-` prefix should be used for all srml modules, but currently `srml_system`
// would cause compiling error in `decl_module!` and `construct_runtime!`
// #3295 https://github.com/paritytech/substrate/issues/3295
use srml_system::{self as system, ensure_signed};

use traits::{BasicCurrency, MultiCurrency, MultiCurrencyExtended};

type BalanceOf<T> = <<T as Trait>::MultiCurrency as MultiCurrency<<T as srml_system::Trait>::AccountId>>::Balance;
type CurrencyIdOf<T> = <<T as Trait>::MultiCurrency as MultiCurrency<<T as srml_system::Trait>::AccountId>>::CurrencyId;
type ErrorOf<T> = <<T as Trait>::MultiCurrency as MultiCurrency<<T as srml_system::Trait>::AccountId>>::Error;

type AmountOf<T> = <<T as Trait>::MultiCurrency as MultiCurrencyExtended<<T as srml_system::Trait>::AccountId>>::Amount;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type MultiCurrency: MultiCurrencyExtended<Self::AccountId>;
	type GetNativeCurrencyId: Get<CurrencyIdOf<Self>>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Currencies {

	}
}

decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId
	{
		Dummy(AccountId),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

	}
}

impl<T: Trait> MultiCurrency<T::AccountId> for Module<T> {
	type Balance = BalanceOf<T>;
	type CurrencyId = CurrencyIdOf<T>;
	type Error = ErrorOf<T>;

	fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
		T::MultiCurrency::total_issuance(currency_id)
	}

	fn balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		T::MultiCurrency::balance(currency_id, who)
	}

	fn transfer(
		currency_id: Self::CurrencyId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: Self::Balance,
	) -> result::Result<(), Self::Error> {
		T::MultiCurrency::transfer(currency_id, from, to, amount)
	}

	fn deposit(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> result::Result<(), Self::Error> {
		T::MultiCurrency::deposit(currency_id, who, amount)
	}

	fn withdraw(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> result::Result<(), Self::Error> {
		T::MultiCurrency::withdraw(currency_id, who, amount)
	}

	fn slash(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
		T::MultiCurrency::slash(currency_id, who, amount)
	}
}

impl<T: Trait> MultiCurrencyExtended<T::AccountId> for Module<T> {
	type Amount = AmountOf<T>;

	fn update_balance(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		by_amount: Self::Amount,
	) -> result::Result<(), Self::Error> {
		T::MultiCurrency::update_balance(currency_id, who, by_amount)
	}
}

pub struct Currency<T, GetCurrencyId>(marker::PhantomData<T>, marker::PhantomData<GetCurrencyId>);

impl<T, GetCurrencyId> BasicCurrency<T::AccountId> for Currency<T, GetCurrencyId>
where
	T: Trait,
	GetCurrencyId: Get<CurrencyIdOf<T>>,
{
	type Balance = BalanceOf<T>;
	type Error = ErrorOf<T>;

	fn total_issuance() -> Self::Balance {
		T::MultiCurrency::total_issuance(GetCurrencyId::get())
	}

	fn balance(who: &T::AccountId) -> Self::Balance {
		T::MultiCurrency::balance(GetCurrencyId::get(), who)
	}

	fn transfer(from: &T::AccountId, to: &T::AccountId, amount: Self::Balance) -> result::Result<(), Self::Error> {
		T::MultiCurrency::transfer(GetCurrencyId::get(), from, to, amount)
	}

	fn deposit(who: &T::AccountId, amount: Self::Balance) -> result::Result<(), Self::Error> {
		T::MultiCurrency::deposit(GetCurrencyId::get(), who, amount)
	}

	fn withdraw(who: &T::AccountId, amount: Self::Balance) -> result::Result<(), Self::Error> {
		T::MultiCurrency::withdraw(GetCurrencyId::get(), who, amount)
	}

	fn slash(who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
		T::MultiCurrency::slash(GetCurrencyId::get(), who, amount)
	}
}

pub type NativeCurrencyOf<T> = Currency<T, <T as Trait>::GetNativeCurrencyId>;

impl<T: Trait> Module<T> {}
