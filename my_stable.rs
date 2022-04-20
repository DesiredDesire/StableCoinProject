#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod my_stable_coin {

    use brush::test_utils::*;
    use brush::traits::EnvAccess;
    use brush::{
        contracts::access_control::*,
        contracts::ownable::*,
        contracts::psp22::extensions::burnable::*,
        contracts::psp22::extensions::metadata::*,
        contracts::psp22::extensions::mintable::*,
        modifiers,
        traits::{AccountIdExt, Flush, ZERO_ADDRESS},
    };

    use ink_env::{CallFlags, Error as EnvError};
    use ink_lang::codegen::Env;
    use ink_prelude::{string::String, vec::Vec};

    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;

    // for testing
    type Event = <MyStable as ::ink_lang::reflect::ContractEventBase>::Type;

    const E18: u128 = 10 ^ 18;

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: Option<AccountId>,
        #[ink(topic)]
        spender: Option<AccountId>,
        value: Balance,
    }

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
        sender: AccountId,
    }

    #[ink(storage)]
    #[derive(
        Default, OwnableStorage, SpreadAllocate, PSP22MetadataStorage, AccessControlStorage,
    )]
    pub struct MyStable {
        #[OwnableStorageField]
        ownable: OwnableData,
        #[PSP22MetadataStorageField]
        metadata: PSP22MetadataData,
        #[AccessControlStorageField]
        access: AccessControlData,

        pub supply: Balance,
        pub allowances: Mapping<(AccountId, AccountId), Balance>,

        pub is_untaxed: Mapping<AccountId, bool>,
        pub untaxed_balances: Mapping<AccountId, Balance>,
        pub taxed_balances: Mapping<AccountId, Balance>,

        pub untaxed_supply: Balance,
        pub taxed_supply: Balance,

        pub tax_interest_update_period: u128,
        pub tax_interest_applied: u128,
        pub tax_rate_e18: u128,
        pub tax_last_block: u128,
        pub tax_denom_e18: u128,
    }

    impl Ownable for MyStable {
        #[ink(message)]
        #[modifiers(only_owner)]
        fn renounce_ownership(&mut self) -> Result<(), OwnableError> {
            let old_owner = self.owner();
            self.ownable.owner = ZERO_ADDRESS.into();
            self._emit_ownership_transferred_event(Some(old_owner), None);
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<(), OwnableError> {
            if new_owner.is_zero() {
                return Err(OwnableError::NewOwnerIsZero);
            }
            let old_owner = self.owner();
            self.ownable.owner = new_owner;
            self._emit_ownership_transferred_event(Some(old_owner), Some(new_owner));
            Ok(())
        }
    }

    const MINTER: RoleType = ink_lang::selector_id!("MINTER");
    const BURNER: RoleType = ink_lang::selector_id!("BURNER");
    const SETTER: RoleType = ink_lang::selector_id!("SETTER");

    impl AccessControl for MyStable {
        #[ink(message)]
        fn has_role(&self, role: RoleType, address: AccountId) -> bool {
            has_role(self, &role, &address)
        }

        #[ink(message)]
        fn get_role_admin(&self, role: RoleType) -> RoleType {
            get_role_admin(self, &role)
        }

        #[ink(message)]
        #[modifiers(only_role(get_role_admin(self, &role)))]
        fn grant_role(
            &mut self,
            role: RoleType,
            account: AccountId,
        ) -> Result<(), AccessControlError> {
            if has_role(self, &role, &account) {
                return Err(AccessControlError::RoleRedundant);
            }
            self.access.members.insert((&role, &account), &());
            self._emit_role_granted(role, account, Some(self._caller()));
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(get_role_admin(self, &role)))]
        fn revoke_role(
            &mut self,
            role: RoleType,
            account: AccountId,
        ) -> Result<(), AccessControlError> {
            check_role(self, &role, &account)?;
            self._do_revoke_role(role, account);
            Ok(())
        }

        #[ink(message)]
        fn renounce_role(
            &mut self,
            role: RoleType,
            account: AccountId,
        ) -> Result<(), AccessControlError> {
            if self._caller() != account {
                return Err(AccessControlError::InvalidCaller);
            }
            check_role(self, &role, &account)?;
            self._do_revoke_role(role, account);
            Ok(())
        }
    }

    impl AccessControlInternal for MyStable {
        fn _emit_role_admin_changed(
            &mut self,
            _role: RoleType,
            _previous_admin_role: RoleType,
            _new_admin_role: RoleType,
        ) {
        }

        default fn _emit_role_granted(
            &mut self,
            _role: RoleType,
            _grantee: AccountId,
            _grantor: Option<AccountId>,
        ) {
        }

        default fn _emit_role_revoked(
            &mut self,
            _role: RoleType,
            _account: AccountId,
            _sender: AccountId,
        ) {
        }

        fn _default_admin() -> RoleType {
            DEFAULT_ADMIN_ROLE
        }

        fn _init_with_caller(&mut self) {
            let caller = self._caller();
            self._init_with_admin(caller);
        }

        fn _init_with_admin(&mut self, admin: AccountId) {
            self._setup_role(Self::_default_admin(), admin);
        }

        fn _setup_role(&mut self, role: RoleType, admin: AccountId) {
            if !has_role(self, &role, &admin) {
                self.access.members.insert((&role, &admin), &());

                self._emit_role_granted(role, admin, None);
            }
        }

        fn _do_revoke_role(&mut self, role: RoleType, account: AccountId) {
            self.access.members.remove((&role, &account));
            self._emit_role_revoked(role, account, self._caller());
        }

        fn _set_role_admin(&mut self, role: RoleType, new_admin: RoleType) {
            let mut entry = self.access.admin_roles.get(&role);
            if entry.is_none() {
                entry = Some(Self::_default_admin());
            }
            let old_admin = entry.unwrap();
            self.access.admin_roles.insert(&role, &new_admin);
            self._emit_role_admin_changed(role, old_admin, new_admin);
        }
    }

    pub fn check_role<T: AccessControlStorage>(
        instance: &T,
        role: &RoleType,
        account: &AccountId,
    ) -> Result<(), AccessControlError> {
        if !has_role(instance, role, account) {
            return Err(AccessControlError::MissingRole);
        }
        Ok(())
    }

    pub fn has_role<T: AccessControlStorage>(
        instance: &T,
        role: &RoleType,
        account: &AccountId,
    ) -> bool {
        instance.get().members.get((role, account)).is_some()
    }

    pub fn get_role_admin<T: AccessControlStorage>(instance: &T, role: &RoleType) -> RoleType {
        instance
            .get()
            .admin_roles
            .get(role)
            .unwrap_or(T::_default_admin())
    }

    impl PSP22 for MyStable {
        #[ink(message)]
        fn total_supply(&self) -> Balance {
            self.supply.clone()
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance {
            self._balance_of_view(&owner)
        }

        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get((&owner, &spender)).unwrap_or(0)
        }

        #[ink(message)]
        fn transfer(
            &mut self,
            to: AccountId,
            value: Balance,
            data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            //let from = Self::env().caller();
            let from = self._caller();
            self._transfer_from_to(from, to, value, data)?;
            Ok(())
        }

        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
            data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            // let caller = Self::env().caller();
            let caller = self._caller();
            let allowance = self.allowance(from, caller);

            if allowance < value {
                return Err(PSP22Error::InsufficientAllowance);
            }

            self._transfer_from_to(from, to, value, data)?;
            self._approve_from_to(from, caller, allowance - value)?;
            Ok(())
        }

        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error> {
            // let owner = Self::env().caller();
            let owner = self._caller();
            self._approve_from_to(owner, spender, value)?;
            Ok(())
        }

        #[ink(message)]
        fn increase_allowance(
            &mut self,
            spender: AccountId,
            delta_value: Balance,
        ) -> Result<(), PSP22Error> {
            // let owner = Self::env().caller();
            let owner = self._caller();
            self._approve_from_to(owner, spender, self.allowance(owner, spender) + delta_value)?;
            Ok(())
        }

        #[ink(message)]
        fn decrease_allowance(
            &mut self,
            spender: AccountId,
            delta_value: Balance,
        ) -> Result<(), PSP22Error> {
            // let owner = Self::env().caller();
            let owner = self._caller();
            let allowance = self.allowance(owner, spender);

            if allowance < delta_value {
                return Err(PSP22Error::InsufficientAllowance);
            }

            self._approve_from_to(owner, spender, allowance - delta_value)?;
            Ok(())
        }
    }

    impl PSP22Mintable for MyStable {
        #[ink(message)]
        #[modifiers(only_role(MINTER))]
        fn mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            self._mint(account, amount)
        }
    }

    impl PSP22Burnable for MyStable {
        #[ink(message)]
        #[modifiers(only_role(BURNER))]
        fn burn(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            self._mint(account, amount)
        }
    }

    impl PSP22Metadata for MyStable {}

    impl MyStable {
        #[ink(constructor)]
        pub fn new(name: Option<String>, symbol: Option<String>, decimal: u8) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                // metadata
                instance.metadata.name = name;
                instance.metadata.symbol = symbol;
                instance.metadata.decimals = decimal;
                // ownable
                let caller = Self::env().caller();
                instance._init_with_owner(caller);
                instance._init_with_admin(caller);
                // TaxedCoinData
                instance.tax_interest_update_period = 3600;
                instance.tax_interest_applied = 0;
                instance.tax_rate_e18 = 1000001000000000000;
                instance.tax_last_block = Self::env().block_number() as u128;
                instance.tax_denom_e18 = E18;
            })
        }
        fn _init_with_owner(&mut self, owner: AccountId) {
            self.ownable.owner = owner;
            self._emit_ownership_transferred_event(None, Some(owner));
        }

        fn _emit_transfer_event(
            &self,
            _from: Option<AccountId>,
            _to: Option<AccountId>,
            _amount: Balance,
        ) {
            Self::env().emit_event(Transfer {
                from: _from,
                to: _to,
                value: _amount,
            });
        }
        fn _emit_approval_event(
            &self,
            _owner: Option<AccountId>,
            _spender: Option<AccountId>,
            _amount: Balance,
        ) {
            Self::env().emit_event(Approval {
                owner: _owner,
                spender: _spender,
                value: _amount,
            });
        }

        fn _emit_ownership_transferred_event(
            &self,
            _previous_owner: Option<AccountId>,
            _new_owner: Option<AccountId>,
        ) {
            Self::env().emit_event(OwnershipTransferred {
                previous_owner: _previous_owner,
                new_owner: _new_owner,
            })
        }

        fn _emit_role_admin_changed_event(
            &mut self,
            _role: RoleType,
            _previous_admin_role: RoleType,
            _new_admin_role: RoleType,
        ) {
            Self::env().emit_event(RoleAdminChanged {
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
            Self::env().emit_event(RoleGranted {
                role: _role,
                grantee: _grantee,
                grantor: _grantor,
            })
        }

        fn _emit_role_revoked(&mut self, _role: RoleType, _account: AccountId, _sender: AccountId) {
            Self::env().emit_event(RoleRevoked {
                role: _role,
                account: _account,
                sender: _sender,
            })
        }

        #[ink(message)]
        pub fn get_minter(&self) -> u32 {
            MINTER
        }
        #[ink(message)]
        pub fn get_setter(&self) -> u32 {
            SETTER
        }
        #[ink(message)]
        pub fn get_burner(&self) -> u32 {
            BURNER
        }
        // fn new_init(&mut self) {
        //     let caller = Self::env().caller();
        //     self._init_with_owner(caller);
        //     self.tax_interest_update_period = 3600;
        //     self.tax_interest_applied = 0;
        //     self.tax_rate_e18 = 1000001000000000000;
        //     self.tax_last_block = Self::env().block_number() as u128;
        //     self.tax_denom_e18 = E18;
        // }'

        fn _block_number(&self) -> u128 {
            Self::env().block_number() as u128
        }
        fn _caller(&self) -> AccountId {
            Self::env().caller()
        }

        #[ink(message)]
        #[modifiers(only_role(SETTER))]
        pub fn set_is_untaxed(
            &mut self,
            account: AccountId,
            set_to: bool,
        ) -> Result<(), AccessControlError> {
            let is_untaxed: bool = self.is_untaxed.get(account).unwrap_or_default();
            if is_untaxed != set_to {
                self._switch_is_untaxed(account, is_untaxed);
            }
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn set_role_admin(
            &mut self,
            role: RoleType,
            new_admin: RoleType,
        ) -> Result<(), PSP22Error> {
            self._set_role_admin(role, new_admin);
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn setup_role(
            &mut self,
            role: RoleType,
            new_member: AccountId,
        ) -> Result<(), OwnableError> {
            self._setup_role(role, new_member);
            Ok(())
        }
    }

    pub trait MyStableInternal {
        fn _balance_of(&mut self, owner: &AccountId) -> Balance;
        fn _balance_of_view(&self, owner: &AccountId) -> Balance;

        fn _do_safe_transfer_check(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: &Balance,
            data: &Vec<u8>,
        ) -> Result<(), PSP22Error>;

        fn _transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            amount: Balance,
            data: Vec<u8>,
        ) -> Result<(), PSP22Error>;

        fn _approve_from_to(
            &mut self,
            owner: AccountId,
            spender: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error>;

        fn _mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error>;

        fn _burn_from(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error>;
    }

    impl MyStableInternal for MyStable {
        fn _balance_of(&mut self, owner: &AccountId) -> Balance {
            if self.is_untaxed.get(owner).unwrap_or(false) {
                ink_env::debug_println!(
                    "UNTAXED {}",
                    self.untaxed_balances.get(owner).unwrap_or(0)
                );
                return self.untaxed_balances.get(owner).unwrap_or(0);
            } else {
                ink_env::debug_println!(
                    "TAXED {}",
                    self.taxed_balances.get(owner).unwrap_or(0) / self._tax_denom()
                );
                return self.taxed_balances.get(owner).unwrap_or(0) / self._tax_denom();
            }
        }

        fn _balance_of_view(&self, owner: &AccountId) -> Balance {
            if self.is_untaxed.get(owner).unwrap_or(false) {
                ink_env::debug_println!(
                    "UNTAXED {}",
                    self.untaxed_balances.get(owner).unwrap_or(0)
                );
                return self.untaxed_balances.get(owner).unwrap_or(0);
            } else {
                ink_env::debug_println!(
                    "TAXED {}",
                    self.taxed_balances.get(owner).unwrap_or(0) / self._tax_denom_view()
                );
                return self.taxed_balances.get(owner).unwrap_or(0) / self._tax_denom_view();
            }
        }

        fn _do_safe_transfer_check(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: &Balance,
            data: &Vec<u8>,
        ) -> Result<(), PSP22Error> {
            self.flush();
            let builder = PSP22ReceiverRef::before_received_builder(
                to,
                self._caller(),
                from.clone(),
                value.clone(),
                data.clone(),
            )
            .call_flags(CallFlags::default().set_allow_reentry(true));
            let result = match builder.fire() {
                Ok(result) => match result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.into()),
                },
                Err(e) => {
                    match e {
                        // `NotCallable` means that the receiver is not a contract.

                        // `CalleeTrapped` means that the receiver has no method called `before_received` or it failed inside.
                        // First case is expected. Second - not. But we can't tell them apart so it is a positive case for now.
                        // https://github.com/paritytech/ink/issues/1002
                        EnvError::NotCallable | EnvError::CalleeTrapped => Ok(()),
                        _ => Err(PSP22Error::SafeTransferCheckFailed(String::from(
                            "Error during call to receiver",
                        ))),
                    }
                }
            };
            self.load();
            result?;
            Ok(())
        }

        fn _transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            amount: Balance,
            data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            if from.is_zero() {
                return Err(PSP22Error::ZeroSenderAddress);
            }
            if to.is_zero() {
                return Err(PSP22Error::ZeroRecipientAddress);
            }

            let from_balance = self._balance_of(&from);

            if from_balance < amount {
                return Err(PSP22Error::InsufficientBalance);
            }
            self._do_safe_transfer_check(&from, &to, &amount, &data)?;

            let from_untaxed: bool = self.is_untaxed.get(from).unwrap_or_default();
            let to_untaxed: bool = self.is_untaxed.get(to).unwrap_or_default();
            // self._before_token_transfer(Some(&account), None, &amount)?;
            if from_untaxed && to_untaxed {
                self.untaxed_balances
                    .insert(&from, &(from_balance - amount));
                let to_balance = self._balance_of(&to);
                self.untaxed_balances.insert(&to, &(to_balance + amount));
                return Ok(());
            } else if from_untaxed && !to_untaxed {
                let taxed_amount: Balance = amount * self._tax_denom();
                self.untaxed_balances
                    .insert(&from, &(from_balance - amount));
                let to_balance: Balance = self.taxed_balances.get(to).unwrap_or(0);
                self.taxed_balances
                    .insert(&to, &(to_balance + taxed_amount));
                self.untaxed_supply -= amount;
                self.taxed_supply += taxed_amount;
            } else if !from_untaxed && to_untaxed {
                let taxed_amount: Balance = amount * self._tax_denom();
                let from_balance: Balance = self.taxed_balances.get(from).unwrap_or(0);
                self.taxed_balances
                    .insert(&from, &(from_balance - taxed_amount));
                let to_balance = self._balance_of(&to);
                self.untaxed_balances.insert(&to, &(to_balance + amount));
                self.taxed_supply -= taxed_amount;
                self.untaxed_supply += amount;
            } else if !from_untaxed && !to_untaxed {
                let taxed_amount: Balance = amount * self._tax_denom();
                let from_balance: Balance = self.taxed_balances.get(from).unwrap_or(0);
                self.taxed_balances
                    .insert(&from, &(from_balance - taxed_amount));
                let to_balance: Balance = self.taxed_balances.get(to).unwrap_or(0);
                self.taxed_balances
                    .insert(&to, &(to_balance + taxed_amount));
            }
            // self._after_token_transfer(Some(&account), None, &amount)?;
            //self._emit_transfer_event(Some(from), Some(to), amount);
            Ok(())
        }
        fn _approve_from_to(
            &mut self,
            owner: AccountId,
            spender: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error> {
            if owner.is_zero() {
                return Err(PSP22Error::ZeroSenderAddress);
            }
            if spender.is_zero() {
                return Err(PSP22Error::ZeroRecipientAddress);
            }

            self.allowances.insert((&owner, &spender), &amount);
            self._emit_approval_event(Some(owner), Some(spender), amount);
            Ok(())
        }

        fn _mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            ink_env::debug_println!("MINT | START{}", self.supply);
            if account.is_zero() {
                return Err(PSP22Error::ZeroRecipientAddress);
            }
            // self._before_token_transfer(Some(&account), None, &amount)?;

            if self.is_untaxed.get(account).unwrap_or_default() {
                let old_balance = self.untaxed_balances.get(account).unwrap_or_default();
                self.untaxed_balances
                    .insert(&account, &(old_balance + amount));
                self.untaxed_supply += amount;
            } else {
                let taxed_amount = amount * self._tax_denom();
                ink_env::debug_println!("amount: {}", amount);
                ink_env::debug_println!("tax: {}", self._tax_denom());
                ink_env::debug_println!("taxed_amount: {}", taxed_amount);
                let old_balance = self.taxed_balances.get(account).unwrap_or_default();
                ink_env::debug_println!("old balance: {}", old_balance);
                ink_env::debug_println!("amount to insert: {}", old_balance + taxed_amount);
                self.taxed_balances
                    .insert(&account, &(old_balance + taxed_amount));
                ink_env::debug_println!(
                    "taxed_balance: {}",
                    self.taxed_balances.get(account).unwrap_or(0)
                );
                self.taxed_supply += taxed_amount;
            }
            self.supply += amount;
            // self._after_token_transfer(Some(&account), None, &amount)?;
            self._emit_transfer_event(None, Some(account), amount);
            ink_env::debug_println!("MINT | END supply: {}", self.supply);
            Ok(())
        }

        fn _burn_from(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            if account.is_zero() {
                return Err(PSP22Error::ZeroRecipientAddress);
            }

            let mut from_balance = self._balance_of(&account);

            if from_balance < amount {
                return Err(PSP22Error::InsufficientBalance);
            }

            if self.is_untaxed.get(account).unwrap_or_default() {
                from_balance -= amount;
                self.untaxed_balances.insert(&account, &from_balance);
            } else {
                from_balance -= amount * self._tax_denom();
                self.taxed_balances.insert(&account, &from_balance);
            }

            self.supply -= amount;
            // self._after_token_transfer(Some(&account), None, &amount)?;
            // self._emit_transfer_event(Some(account), None, amount);

            Ok(())
        }
    }

    pub trait MyStableMath {}
    impl MyStableMath for MyStable {}

    pub trait MyStableInternals {
        fn _tax_denom(&mut self) -> u128;
        fn _tax_denom_view(&self) -> u128;
        fn _taxed_supply(&mut self) -> Balance;
        fn _taxed_supply_view(&self) -> Balance;
        fn _switch_is_untaxed(&mut self, account: AccountId, is_untaxed: bool);
    }

    impl MyStableInternals for MyStable {
        fn _tax_denom(&mut self) -> u128 {
            //TODO add tests
            let current_block: u128 = self._block_number();
            let mut uncounted_blocks: u128 = current_block - self.tax_last_block;
            if uncounted_blocks > 0 {
                let update_period: u128 = self.tax_interest_update_period;
                let mut tax_denom_e18: u128 = self.tax_denom_e18;
                while uncounted_blocks + self.tax_interest_applied > update_period {
                    let add_e18: u128 = tax_denom_e18 * (self.tax_rate_e18 - E18) / E18;
                    let blocks_with_this_inerest: u128 = update_period - self.tax_interest_applied;
                    tax_denom_e18 += add_e18 * blocks_with_this_inerest;
                    uncounted_blocks -= blocks_with_this_inerest;
                    self.tax_interest_applied = 0;
                }
                let add_e18: u128 = tax_denom_e18 * (self.tax_rate_e18 - E18) / E18;
                tax_denom_e18 += add_e18 * uncounted_blocks;
                self.tax_interest_applied = uncounted_blocks;
                self.tax_last_block = current_block;
                self.tax_denom_e18 = tax_denom_e18;
            }
            return self.tax_denom_e18;
        }

        fn _tax_denom_view(&self) -> u128 {
            let current_block: u128 = self._block_number();
            let mut tax_denom_e18: u128 = self.tax_denom_e18;
            let mut tax_interest_applied = self.tax_interest_applied;
            if current_block > self.tax_last_block {
                let mut uncounted_blocks: u128 = current_block - self.tax_last_block;
                let update_period: u128 = self.tax_interest_update_period;
                while uncounted_blocks + tax_interest_applied > update_period {
                    let add_e18: u128 = tax_denom_e18 * (self.tax_rate_e18 - E18) / E18;
                    let blocks_with_this_inerest: u128 = update_period - self.tax_interest_applied;
                    tax_denom_e18 += add_e18 * blocks_with_this_inerest;
                    uncounted_blocks -= blocks_with_this_inerest;
                    tax_interest_applied = 0;
                }
                let add_e18: u128 = tax_denom_e18 * (self.tax_rate_e18 - E18) / E18;
                tax_denom_e18 += add_e18 * uncounted_blocks;
            }
            return tax_denom_e18;
        }
        fn _taxed_supply(&mut self) -> Balance {
            return self.taxed_supply / self._tax_denom();
        }
        fn _taxed_supply_view(&self) -> Balance {
            return self.taxed_supply / self._tax_denom_view();
        }
        fn _switch_is_untaxed(&mut self, account: AccountId, is_untaxed: bool) {
            if is_untaxed {
                let untaxed_balance: u128 = self.untaxed_balances.get(account).unwrap_or(0) as u128;
                let taxed_balance: u128 = untaxed_balance; //self._tax_denom();
                self.taxed_balances.insert(&account, &taxed_balance);
                self.untaxed_balances.insert(&account, &0);
                self.taxed_supply += taxed_balance;
                self.untaxed_supply -= untaxed_balance;
                self.is_untaxed.insert(&account, &(!is_untaxed))
            } else {
                let taxed_balance = self.taxed_balances.get(account).unwrap_or_default();
                let untaxed_balance = taxed_balance; // / self._tax_denom();
                self.untaxed_balances.insert(&account, &untaxed_balance);
                self.taxed_balances.insert(&account, &0);
                self.taxed_supply -= taxed_balance;
                self.untaxed_supply += untaxed_balance;
                self.is_untaxed.insert(&account, &(!is_untaxed))
            }
        }
    }

    // tests
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        const DECIMALS: u8 = 18;

        // #[ink::test]
        // fn should_emit_transfer_event_after_mint() {
        //     // Constructor works.
        //     let amount_to_mint = 100;
        //     let accounts = accounts();
        //     change_caller(accounts.alice);
        //     let mut psp22 = MyStable::new(None, None, DECIMALS);
        //     assert!(psp22.setup_role(MINTER, accounts.bob).is_ok());

        //     change_caller(accounts.bob);
        //     assert!(psp22.mint(accounts.charlie, amount_to_mint).is_ok());
        //     assert_eq!(psp22.balance_of(accounts.charlie), amount_to_mint);
        // }

        /// OWNABLE TEST

        #[ink::test]
        fn constructor_works() {
            let accounts = accounts();
            change_caller(accounts.alice);
            let instance = MyStable::new(None, None, DECIMALS);

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(2, emitted_events.len());

            assert_ownership_transferred_event(&emitted_events[0], None, Some(instance.owner()))
        }

        fn assert_ownership_transferred_event(
            event: &ink_env::test::EmittedEvent,
            expected_previous_owner: Option<AccountId>,
            expected_new_owner: Option<AccountId>,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::OwnershipTransferred(OwnershipTransferred {
                previous_owner,
                new_owner,
            }) = decoded_event
            {
                assert_eq!(
                    previous_owner, expected_previous_owner,
                    "Previous owner was not equal to expected previous owner."
                );
                assert_eq!(
                    new_owner, expected_new_owner,
                    "New owner was not equal to expected new owner."
                );
            } else {
                panic!("encountered unexpected event kind: expected a Transfer event")
            }
        }

        #[ink::test]
        fn owner_works() {
            let my_ownable = MyStable::new(None, None, DECIMALS);
            let caller = my_ownable.env().caller();
            assert_eq!(my_ownable.owner(), caller)
        }

        #[ink::test]
        fn renounce_ownership_works() {
            let mut my_ownable = MyStable::new(None, None, DECIMALS);
            let caller = my_ownable.env().caller();
            let creator = my_ownable.owner();
            assert_eq!(creator, caller);
            assert!(my_ownable.renounce_ownership().is_ok());
            assert!(my_ownable.owner().is_zero());
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(3, emitted_events.len());
            assert_ownership_transferred_event(&emitted_events[0], None, Some(creator));
            assert_ownership_transferred_event(&emitted_events[2], Some(creator), None);
        }

        #[ink::test]
        fn renounce_ownership_fails() {
            let mut my_ownable = MyStable::new(None, None, DECIMALS);
            // Change the caller of `renounce_ownership` method.
            change_caller(AccountId::from([0x13; 32]));
            let result = my_ownable.renounce_ownership();
            assert!(result.is_err());
            assert_eq!(result, Err(OwnableError::CallerIsNotOwner));
        }

        #[ink::test]
        fn transfer_ownership_works() {
            let mut my_ownable = MyStable::new(None, None, DECIMALS);
            let caller = my_ownable.env().caller();
            let creator = my_ownable.owner();
            assert_eq!(creator, caller);
            let new_owner = AccountId::from([5u8; 32]);
            assert!(my_ownable.transfer_ownership(new_owner).is_ok());
            assert_eq!(my_ownable.owner(), new_owner);
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(3, emitted_events.len());
            assert_ownership_transferred_event(&emitted_events[0], None, Some(creator));
            assert_ownership_transferred_event(&emitted_events[2], Some(creator), Some(new_owner));
        }

        #[ink::test]
        fn transfer_ownership_fails() {
            let mut my_ownable = MyStable::new(None, None, DECIMALS);
            // Change the caller of `transfer_ownership` method.
            change_caller(AccountId::from([0x13; 32]));
            let new_owner = AccountId::from([5u8; 32]);
            assert_eq!(
                my_ownable.transfer_ownership(new_owner),
                Err(OwnableError::CallerIsNotOwner)
            );
        }

        #[ink::test]
        fn transfer_ownership_fails_zero_account() {
            let mut my_ownable = MyStable::new(None, None, DECIMALS);
            let new_owner = AccountId::from([0u8; 32]);
            assert_eq!(
                my_ownable.transfer_ownership(new_owner),
                Err(OwnableError::NewOwnerIsZero)
            );
        }
    }
}
