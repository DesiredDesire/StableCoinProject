#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod my_psp22 {

    use brush::{
        contracts::access_control::*,
        contracts::ownable::*,
        contracts::psp22::extensions::burnable::*,
        contracts::psp22::extensions::metadata::*,
        contracts::psp22::extensions::mintable::*,
        modifiers,
        traits::{AccountIdExt, Flush},
    };

    use ink_env::{CallFlags, Error as EnvError};
    use ink_prelude::{string::String, vec::Vec};

    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;

    const E18: u128 = 10 ^ 18;

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

    impl Ownable for MyStable {}

    const MINTER: RoleType = ink_lang::selector_id!("MINTER");
    const BURNER: RoleType = ink_lang::selector_id!("BURNER");
    const SETTER: RoleType = ink_lang::selector_id!("SETTER");

    impl AccessControl for MyStable {}
    impl AccessControlInternal for MyStable {}

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
        // fn new_init(&mut self) {
        //     let caller = Self::env().caller();
        //     self._init_with_owner(caller);
        //     self.tax_interest_update_period = 3600;
        //     self.tax_interest_applied = 0;
        //     self.tax_rate_e18 = 1000001000000000000;
        //     self.tax_last_block = Self::env().block_number() as u128;
        //     self.tax_denom_e18 = E18;
        // }

        fn _block_number(&self) -> u128 {
            self.env().block_number() as u128
        }
        fn _caller(&self) -> AccountId {
            self.env().caller()
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

    pub trait PSP22Internal {
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

    impl PSP22Internal for MyStable {
        fn _balance_of(&mut self, owner: &AccountId) -> Balance {
            if self.is_untaxed.get(owner).unwrap_or(true) {
                return self.untaxed_balances.get(owner).unwrap_or(0);
            } else {
                return self.taxed_balances.get(owner).unwrap_or(0) / self._tax_denom();
            }
        }

        fn _balance_of_view(&self, owner: &AccountId) -> Balance {
            if self.is_untaxed.get(owner).unwrap_or(true) {
                return self.untaxed_balances.get(owner).unwrap_or(0);
            } else {
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
            //self._emit_approval_event(owner, spender, amount);
            Ok(())
        }

        fn _mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
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
                let old_balance = self.taxed_balances.get(account).unwrap_or_default();
                self.taxed_balances
                    .insert(&account, &(old_balance + taxed_amount));
                self.taxed_supply += taxed_amount;
            }
            self.supply += amount;
            // self._after_token_transfer(Some(&account), None, &amount)?;
            //self._emit_transfer_event(None, Some(account), amount);
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
}
