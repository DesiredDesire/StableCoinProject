#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod my_stable_coin {

    use brush::test_utils::*;
    use brush::{
        contracts::access_control::*,
        contracts::ownable::*,
        contracts::psp22::extensions::burnable::*,
        contracts::psp22::extensions::metadata::*,
        contracts::psp22::extensions::mintable::*,
        contracts::psp22::*,
        modifiers,
        traits::{AccountIdExt, Flush, ZERO_ADDRESS},
    };

    use ink_env::{CallFlags, Error as EnvError};
    use ink_lang::codegen::EmitEvent;
    use ink_lang::codegen::Env;
    use ink_prelude::{string::String, vec::Vec};

    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;

    const E12: u128 = 1000000000000;

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
        pub tax_rate_e12: u128,
        pub tax_last_block: u128,
        pub tax_denom_e12: u128,

        pub treassury: AccountId,
    }

    impl Ownable for MyStable {}

    impl OwnableInternal for MyStable {
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

    const MINTER: RoleType = ink_lang::selector_id!("MINTER");
    const BURNER: RoleType = ink_lang::selector_id!("BURNER");
    const SETTER: RoleType = ink_lang::selector_id!("SETTER");

    impl AccessControl for MyStable {}

    impl AccessControlInternal for MyStable {
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

    impl PSP22 for MyStable {
        #[ink(message)]
        fn total_supply(&self) -> Balance {
            self.supply.clone()
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance {
            self._balance_of(&owner)
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
            let from = self.env().caller();
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
            let caller = self.env().caller();
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
            let owner = self.env().caller();
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
            let owner = self.env().caller();
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
            let owner = self.env().caller();
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
            ink_env::debug_println!("MINT | START");
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
                instance.tax_rate_e12 = E12 + 10000000000;
                instance.tax_last_block = Self::env().block_number() as u128;
                instance.tax_denom_e12 = E12;

                instance.treassury = caller;
                instance.is_untaxed.insert(&caller, &(true));
            })
        }

        fn _init_with_owner(&mut self, owner: AccountId) {
            self.ownable.owner = owner;
            self._emit_ownership_transferred_event(None, Some(owner));
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

        #[ink(message)]
        pub fn tax_denom_e12(&mut self) -> Balance {
            self._tax_denom_e12()
        }

        #[ink(message)]
        pub fn tax_denom_e12_view(&self) -> Balance {
            self._tax_denom_e12_view()
        }

        #[ink(message)]
        pub fn taxed_supply(&mut self) -> Balance {
            self._taxed_supply()
        }

        #[ink(message)]
        pub fn taxed_supply_view(&self) -> Balance {
            self._taxed_supply_view()
        }

        #[ink(message)]
        pub fn untaxed_supply(&self) -> Balance {
            self.untaxed_supply
        }

        #[ink(message)]
        pub fn undivided_taxed_supply(&self) -> Balance {
            self._undivided_taxed_supply()
        }

        #[ink(message)]
        pub fn undivided_taxed_balances(&self, account: AccountId) -> Balance {
            self._undivided_taxed_balances(account)
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
                self._switch_is_untaxed(account);
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

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn change_treassury(&mut self, new_treassury: AccountId) -> Result<(), OwnableError> {
            let old_treassury = self.treassury;
            self._switch_is_untaxed(new_treassury);
            self.treassury = new_treassury;
            self._switch_is_untaxed(old_treassury);
            Ok(())
        }

        #[ink(message)]
        pub fn collect_tax(&mut self) -> Result<(), PSP22Error> {
            let tax: Balance = self.supply - self.untaxed_supply - self._taxed_supply();
            self._mint(self.treassury, tax)
        }
    }

    impl PSP22Internal for MyStable {
        fn _emit_transfer_event(
            &self,
            _from: Option<AccountId>,
            _to: Option<AccountId>,
            _amount: Balance,
        ) {
            self.env().emit_event(Transfer {
                from: _from,
                to: _to,
                value: _amount,
            })
        }
        fn _emit_approval_event(&self, _owner: AccountId, _spender: AccountId, _amount: Balance) {
            self.env().emit_event(Approval {
                owner: _owner,
                spender: _spender,
                value: _amount,
            })
        }
        fn _balance_of(&self, owner: &AccountId) -> Balance {
            if self.is_untaxed.get(owner).unwrap_or(false) {
                return self.untaxed_balances.get(owner).unwrap_or(0);
            } else {
                return self.taxed_balances.get(owner).unwrap_or(0) * E12 / self.tax_denom_e12;
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
                self.env().caller(),
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
            ink_env::debug_println!("_TRANSFER_FROM_TO | START");
            if from.is_zero() {
                return Err(PSP22Error::ZeroSenderAddress);
            }
            if to.is_zero() {
                return Err(PSP22Error::ZeroRecipientAddress);
            }
            ink_env::debug_println!("_TRANSFER_FROM_TO | afer zero check");
            // self._do_safe_transfer_check(&from, &to, &amount, &data)?;
            // ink_env::debug_println!("_TRANSFER_FROM_TO | after _do_safe_transfer_check");

            let from_untaxed: bool = self.is_untaxed.get(from).unwrap_or_default();
            let to_untaxed: bool = self.is_untaxed.get(to).unwrap_or_default();
            // self._before_token_transfer(Some(&account), None, &amount)?;
            ink_env::debug_println!(
                "_TRANSFER_FROM_TO | from_bool: {}, to_bool {}",
                from_untaxed,
                to_untaxed
            );
            self._tax_denom_e12();
            let result;
            if from_untaxed && to_untaxed {
                result = self._decrease_untaxed_balance(from, amount);
                self._increase_untaxed_balance(to, amount);
            } else if from_untaxed && !to_untaxed {
                result = self._decrease_untaxed_balance(from, amount);
                self._increase_taxed_balance(to, amount);
            } else if !from_untaxed && to_untaxed {
                result = self._decrease_taxed_balance(from, amount);
                self._increase_untaxed_balance(to, amount);
            } else {
                ink_env::debug_println!("now decrease");
                result = self._decrease_taxed_balance(from, amount);
                ink_env::debug_println!("after decrease:");
                self._increase_taxed_balance(to, amount);
            }
            ink_env::debug_println!("_TRANSFER_FROM_TO | after logic");
            ink_env::debug_println!(
                "_TRANSFER_FROM_TO | from_balance: {}, to_balance: {}",
                self.balance_of(from),
                self.balance_of(to)
            );
            // self._after_token_transfer(Some(&account), None, &amount)?;
            self._emit_transfer_event(Some(from), Some(to), amount);
            result
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
            self._emit_approval_event(owner, spender, amount);
            Ok(())
        }

        fn _mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            if account.is_zero() {
                return Err(PSP22Error::ZeroRecipientAddress);
            }
            // self._before_token_transfer(Some(&account), None, &amount)?;

            self._tax_denom_e12();
            if self.is_untaxed.get(account).unwrap_or_default() {
                self._increase_untaxed_balance(account, amount);
            } else {
                self._increase_taxed_balance(account, amount);
            }
            ink_env::debug_println!(
                "balance_after_mint: {}, {}, mint_amount: {}",
                self.taxed_balances.get(account).unwrap_or(0),
                self.untaxed_balances.get(account).unwrap_or(0),
                amount
            );
            self.supply += amount;
            // self._after_token_transfer(Some(&account), None, &amount)?;
            self._emit_transfer_event(None, Some(account), amount);
            Ok(())
        }

        fn _burn_from(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            if account.is_zero() {
                return Err(PSP22Error::ZeroRecipientAddress);
            }
            self._tax_denom_e12();
            let result;
            if self.is_untaxed.get(account).unwrap_or_default() {
                result = self._decrease_untaxed_balance(account, amount);
            } else {
                result = self._decrease_taxed_balance(account, amount);
            }

            self.supply -= amount;
            // self._after_token_transfer(Some(&account), None, &amount)?;
            self._emit_transfer_event(Some(account), None, amount);

            result
        }
    }

    pub trait MyStableInternals {
        fn _tax_denom_e12(&mut self) -> u128;
        fn _tax_denom_e12_view(&self) -> u128;
        fn _undivided_taxed_supply(&self) -> Balance;
        fn _undivided_taxed_balances(&self, account: AccountId) -> Balance;
        fn _taxed_supply(&mut self) -> Balance;
        fn _taxed_supply_view(&self) -> Balance;
        fn _switch_is_untaxed(&mut self, account: AccountId) -> Result<(), PSP22Error>;
        fn _increase_untaxed_balance(&mut self, account: AccountId, amount: Balance);
        fn _decrease_untaxed_balance(
            &mut self,
            account: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error>;
        fn _increase_taxed_balance(&mut self, account: AccountId, amount: Balance);
        fn _decrease_taxed_balance(
            &mut self,
            account: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error>;
    }

    impl MyStableInternals for MyStable {
        fn _tax_denom_e12(&mut self) -> u128 {
            //TODO add tests
            let current_block: u128 = self.env().block_number() as u128;
            let mut uncounted_blocks: u128 = current_block - self.tax_last_block;

            if uncounted_blocks > 0 {
                let update_period: u128 = self.tax_interest_update_period;
                let mut tax_denom_e12: u128 = self.tax_denom_e12;
                ink_env::debug_println!(
                    "TAX_DENOM_START | current_block: {}; uncounted_blocks: {}; current_tax_denom: {}",
                    current_block,
                    uncounted_blocks,
                    tax_denom_e12
                );
                while uncounted_blocks + self.tax_interest_applied > update_period {
                    let add_e12: u128 = tax_denom_e12 * (self.tax_rate_e12 - E12) / E12;
                    ink_env::debug_println!("TAX_DENOM_START | add_e12{}", add_e12,);
                    let blocks_with_this_inerest: u128 = update_period - self.tax_interest_applied;
                    tax_denom_e12 += add_e12 * blocks_with_this_inerest;
                    uncounted_blocks -= blocks_with_this_inerest;
                    self.tax_interest_applied = 0;
                }
                let add_e12: u128 = tax_denom_e12 * (self.tax_rate_e12 - E12) / E12;
                tax_denom_e12 += add_e12 * uncounted_blocks;
                self.tax_interest_applied = uncounted_blocks;
                self.tax_last_block = current_block;
                self.tax_denom_e12 = tax_denom_e12;
            }

            ink_env::debug_println!(
                "TAX_DENOM_END| current_block: {}; uncounted_blocks: {}; current_tax_denom: {}",
                current_block,
                uncounted_blocks,
                self.tax_denom_e12
            );
            return self.tax_denom_e12;
        }

        fn _tax_denom_e12_view(&self) -> u128 {
            let current_block: u128 = self.env().block_number() as u128;
            let mut tax_denom_e12: u128 = self.tax_denom_e12;
            let mut tax_interest_applied = self.tax_interest_applied;
            if current_block > self.tax_last_block {
                let mut uncounted_blocks: u128 = current_block - self.tax_last_block;
                let update_period: u128 = self.tax_interest_update_period;
                while uncounted_blocks + tax_interest_applied > update_period {
                    let add_e12: u128 = tax_denom_e12 * (self.tax_rate_e12 - E12) / E12;
                    let blocks_with_this_inerest: u128 = update_period - self.tax_interest_applied;
                    tax_denom_e12 += add_e12 * blocks_with_this_inerest;
                    uncounted_blocks -= blocks_with_this_inerest;
                    tax_interest_applied = 0;
                }
                let add_e12: u128 = tax_denom_e12 * (self.tax_rate_e12 - E12) / E12;
                tax_denom_e12 += add_e12 * uncounted_blocks;
            }
            return tax_denom_e12;
        }

        fn _undivided_taxed_supply(&self) -> Balance {
            self.taxed_supply
        }

        fn _undivided_taxed_balances(&self, account: AccountId) -> Balance {
            self.taxed_balances.get(&account).unwrap_or(0)
        }

        fn _taxed_supply(&mut self) -> Balance {
            return self.taxed_supply * E12 / self._tax_denom_e12();
        }
        fn _taxed_supply_view(&self) -> Balance {
            return self.taxed_supply * E12 / self._tax_denom_e12_view();
        }

        fn _switch_is_untaxed(&mut self, account: AccountId) -> Result<(), PSP22Error> {
            if account == self.treassury {
                return Ok(());
            }
            let tax_denom_e12 = self._tax_denom_e12();
            if self.is_untaxed.get(account).unwrap_or_default() {
                let untaxed_balance: Balance =
                    self.untaxed_balances.get(account).unwrap_or(0) as u128;
                self._decrease_untaxed_balance(account, untaxed_balance)?;
                let taxed_balance: Balance = untaxed_balance * tax_denom_e12 / E12;
                self._increase_taxed_balance(account, taxed_balance);
                self.is_untaxed.insert(&account, &(false));
            } else {
                let taxed_balance = self._undivided_taxed_balances(account);
                self._decrease_taxed_balance(account, taxed_balance)?;
                let untaxed_balance = taxed_balance * E12 / tax_denom_e12;
                self._increase_untaxed_balance(account, untaxed_balance);
                self.is_untaxed.insert(&account, &(true));
            }
            return Ok(());
        }
        fn _increase_untaxed_balance(&mut self, account: AccountId, amount: Balance) {
            let balance: Balance = self.untaxed_balances.get(&account).unwrap_or(0);
            self.untaxed_balances.insert(&account, &(balance + amount));
            self.untaxed_supply += amount;
        }
        fn _decrease_untaxed_balance(
            &mut self,
            account: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error> {
            let balance: Balance = self.untaxed_balances.get(&account).unwrap_or(0);
            if balance < amount {
                return Err(PSP22Error::InsufficientBalance);
            }
            self.untaxed_balances.insert(&account, &(balance - amount));
            self.untaxed_supply -= amount;
            Ok(())
        }
        fn _increase_taxed_balance(&mut self, account: AccountId, amount: Balance) {
            let balance: Balance = self.taxed_balances.get(&account).unwrap_or(0);
            let multiplied_amount: Balance = amount * self.tax_denom_e12 / E12;
            ink_env::debug_println!(
                "increase_taxed_balance | amount {}, multiplied_amount {}, tax_denom: {}, E12: {}",
                amount,
                multiplied_amount,
                self.tax_denom_e12,
                E12,
            );
            self.taxed_balances
                .insert(&account, &(balance + multiplied_amount));
            self.taxed_supply += multiplied_amount;
        }
        fn _decrease_taxed_balance(
            &mut self,
            account: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error> {
            ink_env::debug_println!("_DECREASE_TAXED | START");
            let balance: Balance = self.taxed_balances.get(&account).unwrap_or(0);
            let multiplied_amount: Balance = amount * self.tax_denom_e12 / E12 + 1; // round up
            if self._undivided_taxed_balances(account) < multiplied_amount {
                return Err(PSP22Error::InsufficientBalance);
            }
            ink_env::debug_println!("_DECREASE_TAXED | before_balance: {}", balance);
            self.taxed_balances
                .insert(&account, &(balance - multiplied_amount));
            ink_env::debug_println!(
                "_DECREASE_TAXED | after_balance: {}",
                self.taxed_balances.get(&account).unwrap_or(0)
            );
            self.taxed_supply -= multiplied_amount;
            Ok(())
        }
    }

    //
    // EVENT DEFINITIONS
    //
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
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
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
        admin: AccountId,
    }

    //
    // tests
    //

    #[cfg(test)]
    mod tests {
        use super::*;
        use brush::traits::AccountId;
        use ink_lang as ink;
        type Event = <MyStable as ::ink_lang::reflect::ContractEventBase>::Type;
        use ink_env::test::DefaultAccounts;
        use ink_env::DefaultEnvironment;

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

        /// Access Control Tests

        fn assert_role_admin_change_event(
            event: &ink_env::test::EmittedEvent,
            expected_role: RoleType,
            expected_prev_admin: RoleType,
            expected_new_admin: RoleType,
        ) {
            if let Event::RoleAdminChanged(RoleAdminChanged {
                role,
                previous_admin_role,
                new_admin_role,
            }) = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer")
            {
                assert_eq!(
                    role, expected_role,
                    "Roles were not equal: encountered role {:?}, expected role {:?}",
                    role, expected_role
                );
                assert_eq!(
                previous_admin_role, expected_prev_admin,
                "Previous admins were not equal: encountered previous admin {:?}, expected {:?}",
                previous_admin_role, expected_prev_admin
            );
                assert_eq!(
                    new_admin_role, expected_new_admin,
                    "New admins were not equal: encountered new admin {:?}, expected {:?}",
                    new_admin_role, expected_new_admin
                );
            }
        }

        fn assert_role_granted_event(
            event: &ink_env::test::EmittedEvent,
            expected_role: RoleType,
            expected_grantee: AccountId,
            expected_grantor: Option<AccountId>,
        ) {
            if let Event::RoleGranted(RoleGranted {
                role,
                grantee,
                grantor,
            }) = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer")
            {
                assert_eq!(
                    role, expected_role,
                    "Roles were not equal: encountered role {:?}, expected role {:?}",
                    role, expected_role
                );
                assert_eq!(
                    grantee, expected_grantee,
                    "Grantees were not equal: encountered grantee {:?}, expected {:?}",
                    grantee, expected_grantee
                );
                assert_eq!(
                    grantor, expected_grantor,
                    "Grantors were not equal: encountered grantor {:?}, expected {:?}",
                    grantor, expected_grantor
                );
            }
        }

        fn assert_role_revoked_event(
            event: &ink_env::test::EmittedEvent,
            expected_role: RoleType,
            expected_account: AccountId,
            expected_admin: AccountId,
        ) {
            if let Event::RoleRevoked(RoleRevoked {
                role,
                account,
                admin,
            }) = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer")
            {
                assert_eq!(
                    role, expected_role,
                    "Roles were not equal: encountered role {:?}, expected role {:?}",
                    role, expected_role
                );
                assert_eq!(
                    account, expected_account,
                    "Accounts were not equal: encountered account {:?}, expected {:?}",
                    account, expected_account
                );
                assert_eq!(
                    admin, expected_admin,
                    "Admins were not equal: encountered admin {:?}, expected {:?}",
                    admin, expected_admin
                );
            }
        }

        fn setup() -> DefaultAccounts<DefaultEnvironment> {
            let accounts = accounts();

            accounts
        }

        #[ink::test]
        fn should_init_with_default_admin() {
            let accounts = setup();
            let access_control = MyStable::new(None, None, DECIMALS);
            assert!(access_control.has_role(DEFAULT_ADMIN_ROLE, accounts.alice));
            assert_eq!(
                access_control.get_role_admin(DEFAULT_ADMIN_ROLE),
                DEFAULT_ADMIN_ROLE
            );
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_role_granted_event(&emitted_events[0], DEFAULT_ADMIN_ROLE, accounts.alice, None);
        }

        #[ink::test]
        fn should_grant_role() {
            let accounts = setup();
            let alice = accounts.alice;
            let mut access_control = MyStable::new(None, None, DECIMALS);

            assert!(access_control.grant_role(SETTER, alice).is_ok());
            assert!(access_control.grant_role(MINTER, alice).is_ok());
            assert!(access_control.grant_role(BURNER, alice).is_ok());

            assert!(access_control.has_role(DEFAULT_ADMIN_ROLE, alice));
            assert!(access_control.has_role(SETTER, alice));
            assert!(access_control.has_role(MINTER, alice));
            assert!(access_control.has_role(BURNER, alice));

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_role_granted_event(&emitted_events[1], DEFAULT_ADMIN_ROLE, alice, None);
            assert_role_granted_event(&emitted_events[2], SETTER, alice, Some(alice));
            assert_role_granted_event(&emitted_events[3], MINTER, alice, Some(alice));
            assert_role_granted_event(&emitted_events[4], BURNER, alice, Some(alice));
        }

        #[ink::test]
        fn should_grant_role_fail() {
            let accounts = setup();
            let alice = accounts.alice;
            let mut access_control = MyStable::new(None, None, DECIMALS);

            assert!(access_control.grant_role(MINTER, alice).is_ok());
            assert_eq!(
                access_control.grant_role(MINTER, alice),
                Err(AccessControlError::RoleRedundant)
            );
        }

        #[ink::test]
        fn should_revoke_role() {
            let accounts = setup();
            let mut access_control = MyStable::new(None, None, DECIMALS);

            assert!(access_control.grant_role(SETTER, accounts.bob).is_ok());
            assert!(access_control.has_role(SETTER, accounts.bob));
            assert!(access_control.revoke_role(SETTER, accounts.bob).is_ok());

            assert!(!access_control.has_role(SETTER, accounts.bob));

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_role_granted_event(&emitted_events[0], DEFAULT_ADMIN_ROLE, accounts.alice, None);
            assert_role_granted_event(
                &emitted_events[2],
                SETTER,
                accounts.bob,
                Some(accounts.alice),
            );
            assert_role_revoked_event(&emitted_events[2], SETTER, accounts.bob, accounts.alice);
        }

        #[ink::test]
        fn should_renounce_role() {
            let accounts = setup();
            let mut access_control = MyStable::new(None, None, DECIMALS);
            change_caller(accounts.alice);

            assert!(access_control.grant_role(SETTER, accounts.eve).is_ok());
            assert!(access_control.has_role(SETTER, accounts.eve));
            change_caller(accounts.eve);
            assert!(access_control.renounce_role(SETTER, accounts.eve).is_ok());

            assert!(!access_control.has_role(SETTER, accounts.eve));

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_role_granted_event(&emitted_events[1], DEFAULT_ADMIN_ROLE, accounts.alice, None);
            assert_role_granted_event(
                &emitted_events[2],
                SETTER,
                accounts.eve,
                Some(accounts.alice),
            );
            assert_role_revoked_event(&emitted_events[3], SETTER, accounts.eve, accounts.eve);
        }

        #[ink::test]
        fn should_change_role_admin() {
            let accounts = setup();
            let mut access_control = MyStable::new(None, None, DECIMALS);

            assert!(access_control.grant_role(MINTER, accounts.eve).is_ok());
            access_control._set_role_admin(SETTER, MINTER);
            change_caller(accounts.eve);
            assert!(access_control.grant_role(SETTER, accounts.bob).is_ok());

            assert_eq!(access_control.get_role_admin(MINTER), DEFAULT_ADMIN_ROLE);
            assert_eq!(access_control.get_role_admin(SETTER), MINTER);

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_role_granted_event(&emitted_events[1], DEFAULT_ADMIN_ROLE, accounts.alice, None);
            assert_role_granted_event(
                &emitted_events[2],
                MINTER,
                accounts.eve,
                Some(accounts.alice),
            );
            assert_role_admin_change_event(&emitted_events[2], SETTER, DEFAULT_ADMIN_ROLE, MINTER);
            assert_role_granted_event(&emitted_events[3], SETTER, accounts.bob, Some(accounts.eve));
        }

        #[ink::test]
        fn should_return_error_when_not_admin_grant_role() {
            let accounts = setup();
            let mut access_control = MyStable::new(None, None, DECIMALS);

            assert!(access_control.grant_role(MINTER, accounts.eve).is_ok());
            assert!(access_control.grant_role(SETTER, accounts.bob).is_ok());
            access_control._set_role_admin(SETTER, MINTER);

            assert_eq!(
                access_control.grant_role(SETTER, accounts.eve),
                Err(AccessControlError::MissingRole)
            );
        }

        #[ink::test]
        fn should_return_error_when_not_admin_revoke_role() {
            let accounts = setup();
            let mut access_control = MyStable::new(None, None, DECIMALS);

            assert!(access_control.grant_role(MINTER, accounts.eve).is_ok());
            assert!(access_control.grant_role(SETTER, accounts.bob).is_ok());
            access_control._set_role_admin(SETTER, MINTER);

            change_caller(accounts.bob);

            assert_eq!(
                access_control.revoke_role(MINTER, accounts.bob),
                Err(AccessControlError::MissingRole)
            );
        }

        #[ink::test]
        fn should_return_error_when_not_self_renounce_role() {
            let accounts = setup();
            let mut access_control = MyStable::new(None, None, DECIMALS);

            assert!(access_control.grant_role(SETTER, accounts.bob).is_ok());
            assert_eq!(
                access_control.renounce_role(SETTER, accounts.bob),
                Err(AccessControlError::InvalidCaller)
            );
        }

        #[ink::test]
        fn should_return_error_when_account_doesnt_have_role() {
            let accounts = setup();
            change_caller(accounts.alice);
            let mut access_control = MyStable::new(None, None, DECIMALS);

            assert_eq!(
                access_control.renounce_role(SETTER, accounts.alice),
                Err(AccessControlError::MissingRole)
            );
        }

        /// PSP22TEST

        fn assert_transfer_event(
            event: &ink_env::test::EmittedEvent,
            expected_from: Option<AccountId>,
            expected_to: Option<AccountId>,
            expected_value: Balance,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::Transfer(Transfer { from, to, value }) = decoded_event {
                ink_env::debug_println!("from:");
                assert_eq!(from, expected_from, "encountered invalid Transfer.from");
                ink_env::debug_println!("to:");
                assert_eq!(to, expected_to, "encountered invalid Transfer.to");
                ink_env::debug_println!("value: {} {}", value, expected_value);
                assert_eq!(value, expected_value, "encountered invalid Trasfer.value");
            } else {
                panic!("encountered unexpected event kind: expected a Transfer event")
            }
            // let expected_topics = vec![
            //     encoded_into_hash(&PrefixedValue {
            //         value: b"PSP22Struct::Transfer",
            //         prefix: b"",
            //     }),
            //     encoded_into_hash(&PrefixedValue {
            //         prefix: b"PSP22Struct::Transfer::from",
            //         value: &expected_from,
            //     }),
            //     encoded_into_hash(&PrefixedValue {
            //         prefix: b"PSP22Struct::Transfer::to",
            //         value: &expected_to,
            //     }),
            //     encoded_into_hash(&PrefixedValue {
            //         prefix: b"PSP22Struct::Transfer::value",
            //         value: &expected_value,
            //     }),
            // ];
            // for (n, (actual_topic, expected_topic)) in
            //     event.topics.iter().zip(expected_topics).enumerate()
            // {
            //     assert_eq!(
            //         &actual_topic[..],
            //         expected_topic.as_ref(),
            //         "encountered invalid topic at {}",
            //         n
            //     );
            // }
        }

        /// The default constructor does its job.

        #[ink::test]
        fn constructor_works_taxed_coin() {
            // Constructor works.
            let mut psp22 = MyStable::new(None, None, DECIMALS);
            // Transfer event triggered during initial construction.
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 2);
            // Get the token total supply.
            assert_eq!(psp22.total_supply(), 0);
            assert_eq!(psp22.taxed_supply(), 0);
            assert_eq!(psp22.untaxed_supply(), 0);
            assert_eq!(psp22.undivided_taxed_supply(), 0);
            assert_eq!(psp22.tax_denom_e12(), E12);
        }

        #[ink::test]
        fn transfer_event_is_emited_on_mint() {
            let accounts = accounts();
            // Constructor works.
            let mut psp22 = MyStable::new(None, None, DECIMALS);
            // grant minter role and mint
            assert!(psp22.grant_role(MINTER, accounts.bob).is_ok());
            let mut emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 3);

            change_caller(accounts.bob);
            assert!(psp22.mint(accounts.alice, E12).is_ok());
            // Transfer event triggered during initial construction.
            emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 4);
        }

        /// The total supply was applied.
        #[ink::test]
        fn total_supply_works() {
            let accounts = accounts();
            // Constructor works.
            let mut psp22 = MyStable::new(None, None, DECIMALS);
            // grant minter role and mint
            assert!(psp22.grant_role(MINTER, accounts.bob).is_ok());
            change_caller(accounts.bob);
            assert!(psp22.mint(accounts.charlie, E12).is_ok());
            // Transfer event triggered during initial construction.
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 4);
            assert_transfer_event(&emitted_events[3], None, Some(accounts.charlie), E12);
            // Get the token total supply.
            let supply = psp22.total_supply();
            assert_eq!(supply, E12);
        }

        /// Get the actual balance of an account.
        #[ink::test]
        fn balance_of_works() {
            let accounts = accounts();
            // Constructor works.
            let mut psp22 = MyStable::new(None, None, DECIMALS);
            // grant minter role and mint
            assert!(psp22.grant_role(MINTER, accounts.bob).is_ok());
            change_caller(accounts.bob);
            assert!(psp22.mint(accounts.charlie, E12).is_ok());
            // Transfer event triggered during initial construction.
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 4);
            assert_transfer_event(&emitted_events[3], None, Some(accounts.charlie), E12);
            ink_env::debug_println!("balance1");
            assert_eq!(psp22.balance_of(accounts.bob), 0);
            ink_env::debug_println!("balance2");
            assert_eq!(psp22.balance_of(accounts.alice), 0);
            ink_env::debug_println!("balance3");
            assert_eq!(psp22.balance_of(accounts.charlie), E12);
        }

        // TODO, check that taxed balances get lower with each new block
        // #[ink::test]
        // fn taxing_works_() {
        //     let accounts = accounts();
        //     // Constructor works.
        //     let mut psp22 = MyStable::new(None, None, DECIMALS);
        //     // grant minter role and mint
        //     psp22.grant_role(MINTER, accounts.bob);
        //     psp22.mint(accounts.charlie, E12);
        //     // Transfer event triggered during initial construction.
        //     let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
        //     assert_transfer_event(
        //         &emitted_events[3],
        //         None,
        //         Some(accounts.charlie as AccountId),
        //         E12,
        //     );
        //     assert_eq!(psp22.balance_of(accounts.bob), 0);
        //     assert_eq!(psp22.balance_of(accounts.alice), 0);
        //     assert_eq!(psp22.balance_of(accounts.charlie), E12);
        // }

        #[ink::test]
        fn transfer_works() {
            let accounts = accounts();
            // Constructor works.
            let mut psp22 = MyStable::new(None, None, DECIMALS);
            // grant minter role and mint
            assert!(psp22.grant_role(MINTER, accounts.bob).is_ok());
            change_caller(accounts.bob);
            assert!(psp22.mint(accounts.alice, E12).is_ok());

            assert_eq!(psp22.balance_of(accounts.bob), 0);
            assert_eq!(psp22.balance_of(accounts.alice), E12);
            // Alice transfers 10 tokens to Bob.
            change_caller(accounts.alice);
            assert!(psp22.transfer(accounts.bob, 10, Vec::<u8>::new()).is_ok());
            // Bob owns 10 tokens.
            assert_eq!(psp22.balance_of(accounts.bob), 10);
            assert_eq!(psp22.balance_of(accounts.alice), E12 - 11);

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 5);
            // Check first transfer event related to PSP-20 instantiation.
            assert_transfer_event(
                &emitted_events[3],
                None,
                Some(AccountId::from([0x01; 32])),
                E12,
            );
            // Check the second transfer event relating to the actual trasfer.
            assert_transfer_event(
                &emitted_events[4],
                Some(AccountId::from([0x01; 32])),
                Some(AccountId::from([0x02; 32])),
                10,
            );
        }

        #[ink::test]
        fn invalid_transfer_should_fail() {
            let accounts = accounts();
            // Constructor works.
            let mut psp22 = MyStable::new(None, None, DECIMALS);
            // grant minter role and mint
            assert!(psp22.grant_role(MINTER, accounts.bob).is_ok());
            change_caller(accounts.bob);
            assert_eq!(psp22.balance_of(accounts.bob), 0);

            // Bob fails to transfers 10 tokens to Eve.
            assert_eq!(
                psp22.transfer(accounts.eve, 10, Vec::<u8>::new()),
                Err(PSP22Error::InsufficientBalance)
            );
        }

        #[ink::test]
        fn transfer_from_fails() {
            let accounts = accounts();
            // Constructor works.
            let mut psp22 = MyStable::new(None, None, DECIMALS);
            // grant minter role and mint
            assert!(psp22.grant_role(MINTER, accounts.bob).is_ok());
            change_caller(accounts.bob);
            assert!(psp22.mint(accounts.alice, E12).is_ok());

            // Bob fails to transfer tokens owned by Alice.
            assert_eq!(
                psp22.transfer_from(accounts.alice, accounts.eve, 10, Vec::<u8>::new()),
                Err(PSP22Error::InsufficientAllowance)
            );
        }

        #[ink::test]
        fn transfer_from_works() {
            let accounts = accounts();
            // Constructor works.
            let mut psp22 = MyStable::new(None, None, DECIMALS);
            // grant minter role and mint
            assert!(psp22.grant_role(MINTER, accounts.bob).is_ok());
            change_caller(accounts.bob);
            assert!(psp22.mint(accounts.alice, E12).is_ok());

            change_caller(accounts.alice);
            // Alice approves Bob for token transfers on her behalf.
            assert!(psp22.approve(accounts.bob, 10).is_ok());

            // The approve event takes place.
            assert_eq!(ink_env::test::recorded_events().count(), 5);

            change_caller(accounts.bob);

            // Bob transfers tokens from Alice to Eve.
            assert!(psp22
                .transfer_from(accounts.alice, accounts.eve, 10, Vec::<u8>::new())
                .is_ok());
            // Eve owns tokens.
            assert_eq!(psp22.balance_of(accounts.eve), 10);

            // Check all transfer events that happened during the previous calls:
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 7);
            // The second event `emitted_events[1]` is an Approve event that we skip checking.
            assert_transfer_event(
                &emitted_events[5],
                Some(AccountId::from([0x01; 32])),
                Some(AccountId::from([0x05; 32])),
                10,
            );
        }

        #[ink::test]
        fn allowance_must_not_change_on_failed_transfer() {
            let accounts = accounts();
            // Constructor works.
            let mut psp22 = MyStable::new(None, None, DECIMALS);
            // grant minter role and mint
            assert!(psp22.grant_role(MINTER, accounts.bob).is_ok());
            change_caller(accounts.bob);
            assert!(psp22.mint(accounts.alice, E12).is_ok());

            // Alice approves Bob for token transfers on her behalf.
            let alice_balance = psp22.balance_of(accounts.alice);
            change_caller(accounts.alice);
            let initial_allowance = alice_balance + 2;
            assert!(psp22.approve(accounts.bob, initial_allowance).is_ok());
            change_caller(accounts.bob);

            assert_eq!(
                psp22.transfer_from(
                    accounts.alice,
                    accounts.eve,
                    alice_balance + 1,
                    Vec::<u8>::new()
                ),
                Err(PSP22Error::InsufficientBalance)
            );
        }
    }
}
