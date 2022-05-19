#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)] //false positive - without this attribute contract does not compile

#[brush::contract]
pub mod stable_coin {

    use brush::{
        contracts::access_control::*,
        contracts::ownable::*,
        contracts::pausable::*,
        contracts::psp22::extensions::burnable::*,
        contracts::psp22::extensions::metadata::*,
        contracts::psp22::extensions::mintable::*,
        modifiers,
        traits::{AccountIdExt, Flush},
    };
    use stable_coin_project::impls::pausing::*;
    use stable_coin_project::impls::shares_profit_generating::*;
    use stable_coin_project::traits::managing::*;
    use stable_coin_project::traits::psp22_rated::*;

    use ink_env::{CallFlags, Error as EnvError};
    use ink_lang::codegen::EmitEvent;
    use ink_lang::codegen::Env;
    use ink_prelude::{string::String, vec::Vec};

    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;

    const E6: u128 = 10_u128.pow(6);
    const E12: u128 = 10_u128.pow(12);

    const MINTER: RoleType = ink_lang::selector_id!("MINTER");
    const BURNER: RoleType = ink_lang::selector_id!("BURNER");
    // const EMITTER: RoleType = ink_lang::selector_id!("EMITTER");
    // const SETTER: RoleType = ink_lang::selector_id!("SETTER"); // for now we use owner
    const VAULT: RoleType = ink_lang::selector_id!("VAULT");

    #[ink(storage)]
    #[derive(
        Default,
        OwnableStorage,
        PausableStorage,
        SpreadAllocate,
        PSP22Storage,
        PSP22MetadataStorage,
        AccessControlStorage,
        SPGeneratingStorage,
    )]
    pub struct StableCoinContract {
        #[OwnableStorageField]
        ownable: OwnableData,
        #[PausableStorageField]
        pause: PausableData,
        #[AccessControlStorageField]
        access: AccessControlData,
        #[PSP22StorageField]
        psp22: PSP22Data,
        #[PSP22MetadataStorageField]
        metadata: PSP22MetadataData,
        #[SPGeneratingStorageField]
        spgenerate: SPGeneratingData,

        // immutables

        // mutables_internal
        pub rated_supply: Balance,
        pub unrated_supply: Balance,
        pub current_denominator_e12: u128,
        pub last_current_denominator_update_timestamp: Timestamp,
        pub applied_denominator_e12: Mapping<AccountId, u128>,

        // mutables_external
        pub is_unrated: Mapping<AccountId, bool>,
        pub is_tax_free: Mapping<AccountId, bool>,
        pub stable_controller_address: AccountId,

        pub current_interest_rate_e12: i128,
        pub tax_e6: u128,

        pub account_debt: Mapping<AccountId, Balance>, //TODO think about moving this mapping to different contracts
    }

    impl StableCoinContract {
        #[ink(constructor)]
        pub fn new(
            name: Option<String>,
            symbol: Option<String>,
            decimal: u8,
            shares_token_address: AccountId,
            owner: AccountId,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                // metadata
                instance.metadata.name = name;
                instance.metadata.symbol = symbol;
                instance.metadata.decimals = decimal;
                // ownable
                instance._init_with_owner(owner);
                instance._init_with_admin(owner);
                // TaxedCoinData
                instance.spgenerate.shares_token_address = shares_token_address;
                instance.spgenerate.sharing_part_e6 = E6;
                instance.current_denominator_e12 = E12;
                instance.last_current_denominator_update_timestamp =
                    instance.env().block_timestamp();
                instance.current_interest_rate_e12 = 0;
                instance.tax_e6 = 0;
            })
        }
    }

    impl Ownable for StableCoinContract {}

    impl OwnableInternal for StableCoinContract {
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

    impl AccessControlInternal for StableCoinContract {
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

    impl AccessControl for StableCoinContract {}

    impl Managing for StableCoinContract {}

    impl Pausing for StableCoinContract {}

    impl Pausable for StableCoinContract {}

    impl PausableInternal for StableCoinContract {
        /// User must override this method in their contract.
        fn _emit_paused_event(&self, _account: AccountId) {
            self.env().emit_event(Paused { by: Some(_account) });
        }

        /// User must override this method in their contract.
        fn _emit_unpaused_event(&self, _account: AccountId) {
            self.env().emit_event(Unpaused { by: Some(_account) });
        }
    }

    impl SPGenerating for StableCoinContract {}

    impl SPGeneratingView for StableCoinContract {}

    impl SPGeneratingInternal for StableCoinContract {}

    impl PSP22Mintable for StableCoinContract {
        #[ink(message)]
        #[modifiers(only_role(MINTER))]
        #[modifiers(when_not_paused)]
        fn mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            self._mint(account, amount)
        }
    }

    impl PSP22Burnable for StableCoinContract {
        #[ink(message)]
        #[modifiers(only_role(BURNER))]
        fn burn(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            self._burn_from(account, amount)
        }
    }

    impl PSP22Metadata for StableCoinContract {}

    impl PSP22Internal for StableCoinContract {
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
            let unupdated_balance = self._unupdated_balance_of(owner);
            if self._is_unrated(owner) {
                return unupdated_balance;
            }
            let applied_denominator_e12 = self._applied_denominator_e12(owner);
            let current_denominator_e12 = self.current_denominator_e12;
            if current_denominator_e12 > applied_denominator_e12 {
                let denominator_difference_e12 = current_denominator_e12 - applied_denominator_e12;
                let to_add =
                    unupdated_balance * denominator_difference_e12 / current_denominator_e12;
                return unupdated_balance + to_add;
            } else if current_denominator_e12 < applied_denominator_e12 {
                let denominator_difference_e12 = applied_denominator_e12 - current_denominator_e12;
                let to_sub =
                    unupdated_balance * denominator_difference_e12 / current_denominator_e12;
                return unupdated_balance - to_sub;
            } else {
                return unupdated_balance;
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
            // self._before_token_transfer(Some(&account), None, &amount)?;
            self._do_safe_transfer_check(&from, &to, &amount, &data)?;

            let current_denominator_e12 = self._update_current_denominator_e12();
            self._decrease_balance(from, amount, current_denominator_e12)?;
            let tax_e6 = self.tax_e6;
            if tax_e6 == 0 {
                self._increase_balance(to, amount, current_denominator_e12);
            } else {
                let tax = self._calculate_tax(to, amount, tax_e6);
                self._increase_balance(to, amount - tax, current_denominator_e12);
                self._add_profit_and_increase_shares_minting_allowance(tax, to);
                // this comes from tax
            }
            // self._after_token_transfer(Some(&account), None, &amount)?;
            self._emit_transfer_event(Some(from), Some(to), amount);
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

            self.psp22.allowances.insert((&owner, &spender), &amount);
            self._emit_approval_event(owner, spender, amount);
            Ok(())
        }

        fn _mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            if account.is_zero() {
                return Err(PSP22Error::ZeroRecipientAddress);
            }
            // self._before_token_transfer(Some(&account), None, &amount)?;

            let current_denominator_e12 = self._update_current_denominator_e12();
            self._increase_balance(account, amount, current_denominator_e12);
            self.psp22.supply += amount;

            // self._after_token_transfer(Some(&account), None, &amount)?;
            self._emit_transfer_event(None, Some(account), amount);
            Ok(())
        }

        fn _burn_from(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            if account.is_zero() {
                return Err(PSP22Error::ZeroRecipientAddress);
            }
            // self._before_token_transfer(Some(&account), None, &amount)?;

            let current_denominator_e12 = self._update_current_denominator_e12();
            self._decrease_balance(account, amount, current_denominator_e12)?;
            self.psp22.supply -= amount;

            // self._after_token_transfer(Some(&account), None, &amount)?;
            self._emit_transfer_event(Some(account), None, amount);

            Ok(())
        }
    }

    impl PSP22 for StableCoinContract {}

    impl PSP22Rated for StableCoinContract {
        #[ink(message)]
        fn update_current_denominator_e12(&mut self) -> u128 {
            self._update_current_denominator_e12()
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        fn set_is_unrated(&mut self, account: AccountId, set_to: bool) -> Result<(), PSP22Error> {
            let is_unrated: bool = self._is_unrated(&account);
            if is_unrated != set_to {
                self._switch_is_unrated(account)?; //TODO : erroe propagation
            }
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        fn set_is_tax_free(&mut self, account: AccountId, set_to: bool) -> Result<(), PSP22Error> {
            self.is_tax_free.insert(&account, &set_to);
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        fn set_stable_controller_address(
            &mut self,
            new_stable_controller_address: AccountId,
        ) -> Result<(), PSP22Error> {
            self.stable_controller_address = new_stable_controller_address;
            Ok(())
        }

        #[ink(message)]
        fn be_controlled(
            &mut self,
            interest_rate_e12: i128,
            new_tax_e6: u128,
        ) -> Result<(), PSP22Error> {
            if self.env().caller() != self.stable_controller_address {
                return Err(PSP22Error::InsufficientBalance); // TODO error name
            }
            self._update_current_denominator_e12();
            self.current_interest_rate_e12 = interest_rate_e12;
            self.tax_e6 = new_tax_e6;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(VAULT))]
        fn add_account_debt(
            &mut self,
            account: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error> {
            let account_debt: Balance = self._account_debt(&account);
            self.account_debt.insert(&account, &(account_debt + amount));
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(VAULT))]
        fn sub_account_debt(
            &mut self,
            account: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error> {
            let account_debt: Balance = self._account_debt(&account);
            self.account_debt.insert(&account, &(account_debt - amount));
            Ok(())
        }
    }

    impl PSP22RatedView for StableCoinContract {
        #[ink(message)]
        fn rated_supply(&self) -> Balance {
            self.rated_supply
        }

        #[ink(message)]
        fn unrated_supply(&self) -> Balance {
            self.unrated_supply
        }

        #[ink(message)]
        fn current_denominator_e12(&self) -> u128 {
            self.current_denominator_e12
        }

        #[ink(message)]
        fn last_current_denominator_update_timestamp(&self) -> Timestamp {
            self.last_current_denominator_update_timestamp
        }

        #[ink(message)]
        fn applied_denominator_e12(&self, account: AccountId) -> Balance {
            self._applied_denominator_e12(&account)
        }

        #[ink(message)]
        fn is_unrated(&self, account: AccountId) -> bool {
            self._is_unrated(&account)
        }

        #[ink(message)]
        fn stable_controller_address(&self) -> AccountId {
            self.stable_controller_address
        }

        #[ink(message)]
        fn current_interest_rate_e12(&self) -> i128 {
            self.current_interest_rate_e12
        }

        #[ink(message)]
        fn tax_e6(&self) -> u128 {
            self.tax_e6
        }

        #[ink(message)]
        fn is_tax_free(&self, account: AccountId) -> bool {
            self._is_tax_free(&account)
        }

        #[ink(message)]
        fn account_debt(&self, account: AccountId) -> Balance {
            self._account_debt(&account)
        }
    }

    impl PSP22RatedInternals for StableCoinContract {
        fn _unupdated_balance_of(&self, account: &AccountId) -> Balance {
            self.psp22.balances.get(account).unwrap_or(0)
        }

        fn _is_unrated(&self, account: &AccountId) -> bool {
            self.is_unrated.get(account).unwrap_or(false)
        }

        fn _applied_denominator_e12(&self, account: &AccountId) -> u128 {
            self.applied_denominator_e12.get(account).unwrap_or(0) //TODO check
        }

        fn _is_tax_free(&self, account: &AccountId) -> bool {
            self.is_tax_free.get(account).unwrap_or(false)
        }

        fn _account_debt(&self, account: &AccountId) -> Balance {
            self.account_debt.get(account).unwrap_or(0)
        }

        fn _update_current_denominator_e12(&mut self) -> u128 {
            let updated_denominator = self.current_denominator_e12
                * (((self.env().block_timestamp() - self.last_current_denominator_update_timestamp)
                    as i128
                    * self.current_interest_rate_e12) as u128);

            self.current_denominator_e12 = updated_denominator;
            updated_denominator
        }

        fn _switch_is_unrated(&mut self, account: AccountId) -> Result<(), PSP22Error> {
            let current_denominator_e12 = self._update_current_denominator_e12();
            if self._is_unrated(&account) {
                self.is_unrated.insert(&account, &(false));
            } else {
                let unupdated_balance = self._unupdated_balance_of(&account);
                let applied_denominator_e12 = self._applied_denominator_e12(&account);
                // negative interest rates
                if current_denominator_e12 > applied_denominator_e12 {
                    let denominator_difference_e12 =
                        current_denominator_e12 - applied_denominator_e12;
                    let to_substract = unupdated_balance * denominator_difference_e12
                        / current_denominator_e12
                        + 1; //round up
                    self.psp22
                        .balances
                        .insert(&account, &(unupdated_balance - to_substract));
                    self._add_profit_and_increase_shares_minting_allowance(to_substract, account);
                    self.rated_supply -= to_substract;
                    // positive interest rates
                } else if current_denominator_e12 < applied_denominator_e12 {
                    let denominator_difference_e12 =
                        applied_denominator_e12 - current_denominator_e12;
                    let to_add = unupdated_balance * denominator_difference_e12
                        / current_denominator_e12
                        - 1; //round down
                    self.psp22
                        .balances
                        .insert(&account, &(unupdated_balance + to_add));
                    self._sub_profit(to_add);
                    self.rated_supply += to_add;
                }
                self.is_unrated.insert(&account, &(true));
            }
            Ok(())
        }

        // fn _update_balance_of(&mut self, account: AccountId) {
        //     let unupdated_balance: Balance = self._unupdated_balance_of(&account);

        //     if !self._is_unrated(&account) {
        //         let applied_denominator_e12 = self._applied_denominator_e12(&account);
        //         // negative interest rates
        //         if current_denominator_e12 > applied_denominator_e12 {
        //             let denominator_difference_e12 =
        //                 current_denominator_e12 - applied_denominator_e12;
        //             let to_substract = unupdated_balance * denominator_difference_e12
        //                 / current_denominator_e12
        //                 + 1; //round up
        //             self._add_profit_and_increase_shares_minting_allowance(to_substract, account);
        //             self.rated_supply = self.rated_supply - to_substract;
        //             // positive interest rates
        //         } else if current_denominator_e12 < applied_denominator_e12 {
        //             let denominator_difference_e12 =
        //                 applied_denominator_e12 - current_denominator_e12;
        //             let to_add = unupdated_balance * denominator_difference_e12
        //                 / current_denominator_e12
        //                 - 1; //round down
        //             self._sub_profit(to_add);
        //             self.rated_supply = self.rated_supply + to_add;
        //         }
        //     }
        //     self.applied_denominator_e12
        //         .insert(&account, &current_denominator_e12);
        // }

        fn _increase_balance(
            &mut self,
            account: AccountId,
            amount: Balance,
            current_denominator_e12: u128,
        ) {
            let unupdated_balance: Balance = self._unupdated_balance_of(&account);

            if !self._is_unrated(&account) {
                let applied_denominator_e12 = self._applied_denominator_e12(&account);
                // negative interest rates
                if current_denominator_e12 > applied_denominator_e12 {
                    let denominator_difference_e12 =
                        current_denominator_e12 - applied_denominator_e12;
                    let to_substract = unupdated_balance * denominator_difference_e12
                        / current_denominator_e12
                        + 1; //round up
                    self.psp22
                        .balances
                        .insert(&account, &(unupdated_balance - to_substract + amount));
                    self._add_profit_and_increase_shares_minting_allowance(to_substract, account);
                    self.rated_supply = self.rated_supply - to_substract + amount;
                    // positive interest rates
                } else if current_denominator_e12 < applied_denominator_e12 {
                    let denominator_difference_e12 =
                        applied_denominator_e12 - current_denominator_e12;
                    let to_add = unupdated_balance * denominator_difference_e12
                        / current_denominator_e12
                        - 1; //round down
                    self.psp22
                        .balances
                        .insert(&account, &(unupdated_balance + to_add + amount));
                    self._sub_profit(to_add);
                    self.rated_supply = self.rated_supply + to_add + amount;
                } else {
                    self.psp22
                        .balances
                        .insert(&account, &(unupdated_balance + amount));
                    self.rated_supply += amount;
                }
            } else {
                self.psp22
                    .balances
                    .insert(&account, &(unupdated_balance + amount));
                self.unrated_supply += amount;
            }
            self.applied_denominator_e12
                .insert(&account, &current_denominator_e12);
        }

        fn _decrease_balance(
            &mut self,
            account: AccountId,
            amount: Balance,
            current_denominator_e12: u128,
        ) -> Result<(), PSP22Error> {
            let unupdated_balance: Balance = self._unupdated_balance_of(&account);

            if !self._is_unrated(&account) {
                let applied_denominator_e12 = self._applied_denominator_e12(&account);
                // negative interest rates
                if current_denominator_e12 > applied_denominator_e12 {
                    let denominator_difference_e12 =
                        current_denominator_e12 - applied_denominator_e12;
                    let to_substract = unupdated_balance * denominator_difference_e12
                        / current_denominator_e12
                        + 1; //round up
                    let updated_balance = unupdated_balance - to_substract;
                    if amount > updated_balance {
                        return Err(PSP22Error::InsufficientBalance);
                    }
                    self.psp22
                        .balances
                        .insert(&account, &(updated_balance - amount));
                    self._add_profit_and_increase_shares_minting_allowance(to_substract, account); // OK, this comes from tax not rates
                    self.rated_supply = self.rated_supply - to_substract - amount;
                    // positive interest rates
                } else if current_denominator_e12 < applied_denominator_e12 {
                    let denominator_difference_e12 =
                        applied_denominator_e12 - current_denominator_e12;
                    let to_add = unupdated_balance * denominator_difference_e12
                        / current_denominator_e12
                        - 1; //round down
                    let updated_balance = unupdated_balance + to_add;
                    if amount > updated_balance {
                        return Err(PSP22Error::InsufficientBalance);
                    }
                    self.psp22
                        .balances
                        .insert(&account, &(updated_balance + amount));
                    self._sub_profit(to_add);
                    self.rated_supply = self.rated_supply + to_add - amount;
                } else {
                    if amount > unupdated_balance {
                        return Err(PSP22Error::InsufficientBalance);
                    }
                    self.psp22
                        .balances
                        .insert(&account, &(unupdated_balance - amount));
                    self.rated_supply -= amount;
                }
            } else {
                if amount > unupdated_balance {
                    return Err(PSP22Error::InsufficientBalance);
                }
                self.psp22
                    .balances
                    .insert(&account, &(unupdated_balance - amount));
                self.unrated_supply -= amount;
            }
            self.applied_denominator_e12
                .insert(&account, &current_denominator_e12);

            Ok(())
        }

        fn _calculate_tax(&self, account: AccountId, amount: Balance, tax_e6: u128) -> Balance {
            if self._is_tax_free(&account) {
                return 0;
            }

            let account_debt = self._account_debt(&account);
            if account_debt == 0 {
                return amount * tax_e6 / E6;
            } else {
                let account_balance = self.balance_of(account);
                if account_debt >= account_balance {
                    return 0;
                } else {
                    let taxed_amount = account_balance - account_debt;
                    return taxed_amount * tax_e6 / E6;
                }
            }
        }
    }

    //
    // EVENT DEFINITIONS
    //

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
        use brush::test_utils::{accounts, change_caller};
        use brush::traits::AccountId;
        use ink_lang as ink;
        type Event = <StableCoinContract as ::ink_lang::reflect::ContractEventBase>::Type;
        use ink_env::test::DefaultAccounts;
        use ink_env::DefaultEnvironment;

        const DECIMALS: u8 = 18;

        // #[ink::test]
        // fn should_emit_transfer_event_after_mint() {
        //     // Constructor works.
        //     let amount_to_mint = 100;
        //     let accounts = accounts();
        //     change_caller(accounts.alice);
        //     let mut psp22 = StableCoinContract::new(None, None, DECIMALS);
        //     assert!(psp22.setup_role(MINTER, accounts.bob).is_ok());

        //     change_caller(accounts.bob);
        //     assert!(psp22.mint(accounts.charlie, amount_to_mint).is_ok());
        //     assert_eq!(psp22.balance_of(accounts.charlie), amount_to_mint);
        // }

        /// OWNABLE TEST

        #[ink::test]
        fn constructor_works(name: String, symbol: String, decimals: u8) {
            let accounts = accounts();
            change_caller(accounts.alice);
            let instance =
                StableCoinContract::new(None, None, DECIMALS, accounts.bob, accounts.charlie);

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(2, emitted_events.len());
        }
    }
}
