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
