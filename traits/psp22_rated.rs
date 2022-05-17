use brush::{
    contracts::traits::psp22::*,
    traits::{AccountId, Balance},
};

#[brush::wrapper]
pub type PSP22RatedRef = dyn PSP22Rated + PSP22;

#[brush::trait_definition]
pub trait PSP22Rated {
    #[ink(message)]
    fn set_is_unrated(&mut self, account: AccountId, set_to: bool) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn update_current_denominator_e12(&mut self) -> u128;

    #[ink(message)]
    fn be_controlled(&mut self, new_interest_rate: i128) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn set_controller_address(
        &mut self,
        new_controller_address: AccountId,
    ) -> Result<(), PSP22Error>;
}

#[brush::trait_definition]
pub trait PSP22RatedView {
    #[ink(message)]
    fn rated_supply(&self) -> Balance;

    #[ink(message)]
    fn unrated_supply(&self) -> Balance;

    #[ink(message)]
    fn applied_denominator_e12(&self) -> Balance;
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
