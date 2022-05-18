#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)] //false positive - without this attribute contract does not compile

#[brush::contract]
pub mod shares_token {

    use brush::{
        contracts::access_control::*, contracts::ownable::*, contracts::pausable::*,
        contracts::psp22::extensions::burnable::*, contracts::psp22::extensions::metadata::*,
        contracts::psp22::extensions::mintable::*, modifiers,
    };
    use stable_coin_project::impls::pausing::*;
    use stable_coin_project::traits::managing::*;

    use ink_lang::codegen::EmitEvent;
    use ink_lang::codegen::Env;
    use ink_prelude::string::String;
    use ink_storage::traits::SpreadAllocate;

    const MINTER: RoleType = ink_lang::selector_id!("MINTER");
    const BURNER: RoleType = ink_lang::selector_id!("BURNER");

    const SHARES_DECIMALS: u128 = 10_u128.pow(6);
    const STABLE_DECIMALS: u128 = 10_u128.pow(6);
    const INIT_SUP: u128 = 10_u128.pow(7); //10 * 10^6

    #[ink(storage)]
    #[derive(
        Default,
        SpreadAllocate,
        OwnableStorage,
        PausableStorage,
        PSP22Storage,
        PSP22MetadataStorage,
        AccessControlStorage,
    )]
    pub struct SharesContract {
        #[PSP22StorageField]
        psp22: PSP22Data,
        #[PSP22MetadataStorageField]
        metadata: PSP22MetadataData,
        #[OwnableStorageField]
        ownable: OwnableData,
        #[PausableStorageField]
        pausable: PausableData,
        #[AccessControlStorageField]
        access: AccessControlData,

        pub total_minted_amount: Balance,
    }

    impl SharesContract {
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
                instance.total_minted_amount = INIT_SUP * SHARES_DECIMALS;
                instance._mint(owner, INIT_SUP * SHARES_DECIMALS);
            })
        }
    }

    impl Ownable for SharesContract {}

    impl OwnableInternal for SharesContract {
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

    impl AccessControl for SharesContract {}

    impl AccessControlInternal for SharesContract {
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

    impl Pausing for SharesContract {}

    impl Pausable for SharesContract {}

    #[ink(event)]
    pub struct Paused {
        #[ink(topic)]
        by: Option<AccountId>,
    }
    #[ink(event)]
    pub struct Unpaused {
        #[ink(topic)]
        by: Option<AccountId>,
    }
    impl PausableInternal for SharesContract {
        /// User must override this method in their contract.
        fn _emit_paused_event(&self, _account: AccountId) {
            self.env().emit_event(Paused { by: Some(_account) });
        }

        /// User must override this method in their contract.
        fn _emit_unpaused_event(&self, _account: AccountId) {
            self.env().emit_event(Unpaused { by: Some(_account) });
        }
    }

    impl Managing for SharesContract {}

    impl PSP22 for SharesContract {}

    impl PSP22Metadata for SharesContract {}

    impl PSP22Mintable for SharesContract {
        #[ink(message)]
        #[modifiers(only_role(MINTER))]
        #[modifiers(when_not_paused)]
        fn mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            let amount_to_mint = amount * SHARES_DECIMALS / STABLE_DECIMALS * INIT_SUP
                / (2 * self.total_minted_amount);
            self.total_minted_amount += amount_to_mint;
            self._mint(account, amount_to_mint)
        }
    }

    impl PSP22Burnable for SharesContract {
        #[ink(message)]
        #[modifiers(only_role(BURNER))]
        fn burn(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            self._burn_from(account, amount)?;
            Ok(())
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
