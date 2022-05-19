pub use crate::traits::managing::*;

use brush::{
    contracts::{access_control::*, ownable::*},
    modifiers,
    traits::AccountId,
};

impl<T: OwnableStorage + AccessControlStorage> Managing for T {
    #[modifiers(only_owner)]
    default fn set_role_admin(
        &mut self,
        role: RoleType,
        new_admin: RoleType,
    ) -> Result<(), OwnableError> {
        self._set_role_admin(role, new_admin);
        Ok(())
    }

    #[modifiers(only_owner)]
    default fn setup_role(
        &mut self,
        role: RoleType,
        new_member: AccountId,
    ) -> Result<(), OwnableError> {
        self._setup_role(role, new_member);
        Ok(())
    }
}
