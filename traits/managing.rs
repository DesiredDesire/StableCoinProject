use brush::{
    contracts::traits::{
        access_control::*,
        ownable::*,
        psp22::extensions::{burnable::*, metadata::*, mintable::*},
        psp22::*,
    },
    traits::AccountId,
};

#[brush::wrapper]
pub type ManagingContractRef =
    dyn Managing + PSP22 + PSP22Mintable + PSP22Burnable + PSP22Metadata + Ownable + AccessControl;

#[brush::trait_definition]
pub trait ManagingRef:
    Managing + PSP22 + PSP22Mintable + PSP22Burnable + PSP22Metadata + Ownable + AccessControl
{
}

#[brush::trait_definition]
pub trait Managing {
    #[ink(message)]
    fn get_minter(&self) -> u32;
    #[ink(message)]
    fn get_setter(&self) -> u32;
    #[ink(message)]
    fn get_burner(&self) -> u32;

    #[ink(message)]
    fn set_role_admin(&mut self, role: RoleType, new_admin: RoleType) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn setup_role(&mut self, role: RoleType, new_member: AccountId) -> Result<(), OwnableError>;
}
