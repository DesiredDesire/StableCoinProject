use brush::{
    contracts::traits::{access_control::*, ownable::*},
    traits::AccountId,
};

#[brush::wrapper]
pub type ManagingContractRef = dyn Managing + Ownable + AccessControl;

#[brush::trait_definition]
pub trait ManagingRef: Managing + Ownable + AccessControl {}

#[brush::trait_definition]
pub trait Managing {
    #[ink(message)]
    fn set_role_admin(&mut self, role: RoleType, new_admin: RoleType) -> Result<(), OwnableError>;

    #[ink(message)]
    fn setup_role(&mut self, role: RoleType, new_member: AccountId) -> Result<(), OwnableError>;
}
