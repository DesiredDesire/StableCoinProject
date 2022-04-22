use brush::{
    contracts::traits::{
        access_control::*,
        ownable::*,
        psp22::extensions::{burnable::*, metadata::*, mintable::*},
        psp22::*,
    },
    traits::{AccountId, Balance},
};

#[brush::wrapper]
pub type StableCoinContractRef = dyn PSP22Taxed
    + PSP22
    + PSP22Mintable
    + PSP22Burnable
    + PSP22Metadata
    + Ownable
    + AccessControl;

#[brush::trait_definition]
pub trait StableCoinRef:
    PSP22Taxed + PSP22 + PSP22Mintable + PSP22Burnable + PSP22Metadata + Ownable + AccessControl
{
}

#[brush::trait_definition]
pub trait PSP22Taxed {
    #[ink(message)]
    fn get_minter(&self) -> u32;
    #[ink(message)]
    fn get_setter(&self) -> u32;
    #[ink(message)]
    fn get_burner(&self) -> u32;

    #[ink(message)]
    fn tax_denom_e12(&mut self) -> Balance;

    #[ink(message)]
    fn tax_denom_e12_view(&self) -> Balance;

    #[ink(message)]
    fn taxed_supply(&mut self) -> Balance;

    #[ink(message)]
    fn taxed_supply_view(&self) -> Balance;

    #[ink(message)]
    fn untaxed_supply(&self) -> Balance;

    #[ink(message)]
    fn undivided_taxed_supply(&self) -> Balance;

    #[ink(message)]
    fn undivided_taxed_balances(&self, account: AccountId) -> Balance;

    #[ink(message)]
    fn set_is_untaxed(
        &mut self,
        account: AccountId,
        set_to: bool,
    ) -> Result<(), AccessControlError>;

    #[ink(message)]
    fn set_role_admin(&mut self, role: RoleType, new_admin: RoleType) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn setup_role(&mut self, role: RoleType, new_member: AccountId) -> Result<(), OwnableError>;

    #[ink(message)]
    fn change_treassury(&mut self, new_treassury: AccountId) -> Result<(), OwnableError>;

    #[ink(message)]
    fn collect_tax(&mut self) -> Result<(), PSP22Error>;
}
