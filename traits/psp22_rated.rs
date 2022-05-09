use brush::{
    contracts::traits::psp22::*,
    traits::{AccountId, Balance},
};

#[brush::wrapper]
pub type PSP22RatedRef = dyn PSP22Rated + PSP22;

#[brush::trait_definition]
pub trait PSP22Rated {
    #[ink(message)]
    fn rated_supply(&self) -> Balance;

    #[ink(message)]
    fn unrated_supply(&self) -> Balance;

    #[ink(message)]
    fn applied_denominator_e12(&self) -> Balance;

    #[ink(message)]
    fn interest_income(&self) -> Balance;

    #[ink(message)]
    fn interest_debt(&self) -> Balance;

    #[ink(message)]
    fn set_is_unrated(&mut self, account: AccountId, set_to: bool) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn change_treassury_address(
        &mut self,
        new_treassury_address: AccountId,
    ) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn collect_interest_income(&mut self) -> Result<(), PSP22Error>;
}

#[brush::trait_definition]
pub trait PSP22RatedInternals {
    fn _unupdated_balance_of(&self, account: &AccountId) -> Balance;
    fn _is_unrated(&self, account: &AccountId) -> bool;
    fn _applied_denominator_e12(&self, account: &AccountId) -> u128;
    fn _get_current_denominator_e12(&self) -> u128;
    fn _update_current_denominator_e12(&mut self) -> Result<u128, PSP22Error>;
    fn _add_collected_interests(&mut self, amount: Balance);
    fn _sub_collected_interests(&mut self, amount: Balance);
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
}
