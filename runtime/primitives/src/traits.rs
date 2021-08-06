use sp_runtime::DispatchResult;

pub trait XCurrency<AccountId> {
	type Balance;
	type CurrencyId;

	/// The free balance of `who` under `currency_id`.
	fn account(currency_id: Self::CurrencyId, who: &AccountId) -> Self::Balance;

	/// Add `amount` to the balance of `who` under `currency_id`
	fn deposit(
		currency_id: Self::CurrencyId,
		who: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	/// Remove `amount` from the balance of `who` under `currency_id`
	fn withdraw(
		currency_id: Self::CurrencyId,
		who: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult;
}
