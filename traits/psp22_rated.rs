use brush::{
    contracts::traits::psp22::*,
    traits::{AccountId, Balance, Timestamp},
};

#[brush::wrapper]
pub type PSP22RatedRef = dyn PSP22Rated + PSP22 + PSP22RatedView;

#[brush::trait_definition]
pub trait PSP22Rated {
    #[ink(message)]
    fn update_current_denominator_e12(&mut self) -> u128;

    #[ink(message)]
    fn set_is_unrated(&mut self, account: AccountId, set_to: bool) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn set_is_tax_free(&mut self, account: AccountId, set_to: bool) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn set_stable_controller_address(
        &mut self,
        new_stable_controller_address: AccountId,
    ) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn be_controlled(
        &mut self,
        new_interest_rate: i128,
        new_tax_e6: u128,
    ) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn add_account_debt(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn sub_account_debt(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error>;
}

#[brush::trait_definition]
pub trait PSP22RatedView {
    #[ink(message)]
    fn rated_supply(&self) -> Balance;

    #[ink(message)]
    fn unrated_supply(&self) -> Balance;

    #[ink(message)]
    fn current_denominator_e12(&self) -> u128;

    #[ink(message)]
    fn last_current_denominator_update_timestamp(&self) -> Timestamp;

    #[ink(message)]
    fn applied_denominator_e12(&self, account: AccountId) -> Balance;

    #[ink(message)]
    fn is_unrated(&self, account: AccountId) -> bool;

    #[ink(message)]
    fn stable_controller_address(&self) -> AccountId;

    #[ink(message)]
    fn current_interest_rate_e12(&self) -> i128;

    #[ink(message)]
    fn tax_e6(&self) -> u128;

    #[ink(message)]
    fn is_tax_free(&self, account: AccountId) -> bool;

    #[ink(message)]
    fn account_debt(&self, account: AccountId) -> Balance;
}

#[brush::trait_definition]
pub trait PSP22RatedInternals {
    fn _unupdated_balance_of(&self, account: &AccountId) -> Balance;
    fn _is_unrated(&self, account: &AccountId) -> bool;
    fn _applied_denominator_e12(&self, account: &AccountId) -> u128;
    fn _is_tax_free(&self, account: &AccountId) -> bool;
    fn _account_debt(&self, account: &AccountId) -> Balance;
    fn _update_current_denominator_e12(&mut self) -> u128;
    fn _switch_is_unrated(&mut self, account: AccountId) -> Result<(), PSP22Error>;
    fn _increase_balance(
        &mut self,
        account: AccountId,
        amount: Balance,
        current_denominator_e12: u128,
    );
    fn _decrease_balance(
        &mut self,
        account: AccountId,
        amount: Balance,
        current_denominator_e12: u128,
    ) -> Result<(), PSP22Error>;
    fn _calculate_tax(&self, account: AccountId, amount: Balance, tax_e6: u128) -> Balance;
}
