pub use crate::traits::managing::*;

use brush::{
    contracts::{access_control::*, ownable::*, psp22::*},
    modifiers,
    traits::AccountId,
};

impl<T: PSP22Storage + OwnableStorage + AccessControlStorage> Managing for T {
    default fn get_minter(&self) -> u32 {
        ink_lang::selector_id!("MINTER")
    }
    default fn get_setter(&self) -> u32 {
        ink_lang::selector_id!("BURNER")
    }
    default fn get_burner(&self) -> u32 {
        ink_lang::selector_id!("SETTER")
    }

    #[modifiers(only_owner)]
    default fn set_role_admin(
        &mut self,
        role: RoleType,
        new_admin: RoleType,
    ) -> Result<(), PSP22Error> {
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
