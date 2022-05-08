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
    fn set_role_admin(&mut self, role: RoleType, new_admin: RoleType) -> Result<(), ManagingError>;

    #[ink(message)]
    fn setup_role(&mut self, role: RoleType, new_member: AccountId) -> Result<(), ManagingError>;
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ManagingError {
    OwnableError(OwnableError),
    AccessControlError(AccessControlError),
}

impl From<OwnableError> for ManagingError {
    fn from(error: OwnableError) -> Self {
        ManagingError::OwnableError(error)
    }
}

impl From<AccessControlError> for ManagingError {
    fn from(error: AccessControlError) -> Self {
        ManagingError::AccessControlError(error)
    }
}
