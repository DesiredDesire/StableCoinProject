#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)] //false positive - without this attribute contract does not compile

#[brush::contract]
pub mod psp22_emitable {

    use brush::{
        contracts::access_control::*, contracts::ownable::*,
        contracts::psp22::extensions::burnable::*, contracts::psp22::extensions::metadata::*,
        contracts::psp22::extensions::mintable::*, modifiers,
    };
    use stable_coin_project::traits::managing::*;

    use ink_lang::codegen::EmitEvent;
    use ink_lang::codegen::Env;
    use ink_prelude::string::String;
    use ink_storage::traits::SpreadAllocate;

    const MINTER: RoleType = ink_lang::selector_id!("MINTER");
    const BURNER: RoleType = ink_lang::selector_id!("BURNER");

    #[ink(storage)]
    #[derive(
        Default,
        OwnableStorage,
        SpreadAllocate,
        PSP22Storage,
        PSP22MetadataStorage,
        AccessControlStorage,
    )]
    pub struct PSP22EmitableContract {
        #[PSP22StorageField]
        psp22: PSP22Data,
        #[PSP22MetadataStorageField]
        metadata: PSP22MetadataData,
        #[OwnableStorageField]
        ownable: OwnableData,
        #[AccessControlStorageField]
        access: AccessControlData,
    }

    impl Ownable for PSP22EmitableContract {}

    impl OwnableInternal for PSP22EmitableContract {
        fn _emit_ownership_transferred_event(
            &self,
            _previous_owner: Option<AccountId>,
            _new_owner: Option<AccountId>,
        ) {
            self.env().emit_event(OwnershipTransferred {
                previous_owner: _previous_owner,
                new_owner: _new_owner,
            })
        }
    }

    impl AccessControl for PSP22EmitableContract {}

    impl AccessControlInternal for PSP22EmitableContract {
        fn _emit_role_admin_changed(
            &mut self,
            _role: RoleType,
            _previous_admin_role: RoleType,
            _new_admin_role: RoleType,
        ) {
            self.env().emit_event(RoleAdminChanged {
                role: _role,
                previous_admin_role: _previous_admin_role,
                new_admin_role: _new_admin_role,
            })
        }

        fn _emit_role_granted(
            &mut self,
            _role: RoleType,
            _grantee: AccountId,
            _grantor: Option<AccountId>,
        ) {
            self.env().emit_event(RoleGranted {
                role: _role,
                grantee: _grantee,
                grantor: _grantor,
            })
        }

        fn _emit_role_revoked(&mut self, _role: RoleType, _account: AccountId, _admin: AccountId) {
            self.env().emit_event(RoleRevoked {
                role: _role,
                account: _account,
                admin: _admin,
            })
        }
    }

    impl PSP22 for PSP22EmitableContract {}

    impl PSP22Metadata for PSP22EmitableContract {}

    impl PSP22Mintable for PSP22EmitableContract {
        #[ink(message)]
        #[modifiers(only_role(MINTER))]
        fn mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            self._mint(account, amount)
        }
    }

    impl PSP22Burnable for PSP22EmitableContract {
        #[ink(message)]
        #[modifiers(only_role(BURNER))]
        fn burn(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            self._burn_from(account, amount)
        }
    }

    impl Managing for PSP22EmitableContract {}

    impl PSP22EmitableContract {
        #[ink(constructor)]
        pub fn new(
            name: Option<String>,
            symbol: Option<String>,
            decimal: u8,
            owner: AccountId,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                // metadata
                instance.metadata.name = name;
                instance.metadata.symbol = symbol;
                instance.metadata.decimals = decimal;
                // ownable & access_control
                instance._init_with_owner(owner);
                instance._init_with_admin(owner);
            })
        }

        #[ink(message)]
        pub fn mint_any_caller(
            &mut self,
            account: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error> {
            self._mint(account, amount)
        }
    }
    // EVENT DEFINITIONS #[ink(event)]
    #[ink(event)]
    pub struct OwnershipTransferred {
        #[ink(topic)]
        previous_owner: Option<AccountId>,
        #[ink(topic)]
        new_owner: Option<AccountId>,
    }

    #[ink(event)]
    pub struct RoleAdminChanged {
        #[ink(topic)]
        role: RoleType,
        #[ink(topic)]
        previous_admin_role: RoleType,
        #[ink(topic)]
        new_admin_role: RoleType,
    }

    #[ink(event)]
    pub struct RoleGranted {
        #[ink(topic)]
        role: RoleType,
        #[ink(topic)]
        grantee: AccountId,
        #[ink(topic)]
        grantor: Option<AccountId>,
    }

    #[ink(event)]
    pub struct RoleRevoked {
        #[ink(topic)]
        role: RoleType,
        #[ink(topic)]
        account: AccountId,
        #[ink(topic)]
        admin: AccountId,
    }
}
