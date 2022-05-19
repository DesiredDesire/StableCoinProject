#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod vault {
    use brush::{
        contracts::{ownable::*, pausable::*, psp22::*, psp34::extensions::metadata::*, psp34::*},
        modifiers,
    };
    use ink_lang::codegen::EmitEvent;
    use ink_lang::codegen::Env;
    use ink_prelude::string::ToString;
    use ink_prelude::vec::Vec;
    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;
    use stable_coin_project::impls::collateralling::*;
    use stable_coin_project::impls::emitting::*;
    use stable_coin_project::impls::pausing::*;
    use stable_coin_project::impls::shares_profit_generating::*;
    use stable_coin_project::traits::oracling::OraclingRef;
    use stable_coin_project::traits::psp22_rated::*;
    use stable_coin_project::traits::vault::*;

    // const U128MAX: u128 = 340282366920938463463374607431768211455;
    const E6: u128 = 10_u128.pow(6);
    const E12: u128 = 10_u128.pow(12);

    const COLLATERAL_DECIMALS: u128 = 10_u128.pow(12);
    const STABLE_DECIMALS: u128 = 10_u128.pow(6);

    #[ink(storage)]
    #[derive(
        Default,
        SpreadAllocate,
        OwnableStorage,
        PausableStorage,
        PSP34Storage,
        PSP34MetadataStorage,
        CollaterallingStorage,
        EmittingStorage,
        SPGeneratingStorage,
    )]
    pub struct VaultContract {
        #[OwnableStorageField]
        ownable: OwnableData,
        #[PausableStorageField]
        pause: PausableData,
        #[PSP34StorageField] // vault ownership
        psp34: PSP34Data,
        #[PSP34MetadataStorageField] // vault ownership
        metadata: PSP34MetadataData,
        #[CollaterallingStorageField] // collateral_token_address && collateral_amount
        collateral: CollaterallingData,
        #[EmittingStorageField] // emited_token_address && emited_amount
        emit: EmittingData,
        #[SPGeneratingStorageField]
        spgenerate: SPGeneratingData,

        // immutables
        pub maximum_minimum_collateral_coefficient_e6: u128,
        pub interest_rate_step_value_e12: i128,
        pub collateral_step_value_e6: u128,

        // mutables_internal
        pub collateral_by_id: Mapping<u128, Balance>,
        pub debt_by_id: Mapping<u128, Balance>,
        pub total_debt: Balance,
        pub next_id: u128,

        pub current_interest_coefficient_e12: u128, // the current interest coefficient (acmulated interest)
        pub last_interest_coefficient_by_id_e12: Mapping<u128, u128>, // the last interest coefficient (acumulated interest) used for vault with id
        pub last_interest_coefficient_timestamp: Timestamp, // last block number when current_interest_coefficient_e12 was updated

        // mutables_external
        pub oracle_address: AccountId,
        pub controller_address: AccountId, // controlling_contract
        pub liquidator_address: AccountId,

        //// vault parameters
        pub current_interest_rate_e12: i128, // interest_rate_step_value_e12 * current_interest_step( which is stored in vault_controller)
        pub current_minimum_collateral_coefficient_e6: u128, // maximum_minimum_collaterall - collateral_step_value * current_collateral_step (shich is stored in vault_controller)
    }
    impl Ownable for VaultContract {} // owner can pause contract
    impl Pausable for VaultContract {} // when paused borrowing is imposible
    impl Pausing for VaultContract {} // owner can pause and unpause
    impl PSP34 for VaultContract {} // PSP34 is prove of being vault_owner
    impl EmittingInternal for VaultContract {} // minting and burning emited_token
    impl Emitting for VaultContract {} // emited_amount() = minted - burned
    impl CollaterallingInternal for VaultContract {} // transfer in, transfer out
    impl Collateralling for VaultContract {} // amount of collaterall
    impl SPGeneratingInternal for VaultContract {} // modify generated_profit and shares_minting_allowance
    impl SPGenerating for VaultContract {} //manage generated_profit
    impl SPGeneratingView for VaultContract {} //manage generated_profit

    impl VaultContract {
        #[ink(constructor)]
        pub fn new(
            oracle_address: AccountId,
            shares_token_address: AccountId,
            shares_profit_controller_address: AccountId,
            collateral_token_address: AccountId,
            stable_token_address: AccountId,
            maximum_minimum_collateral_coefficient_e6: u128,
            collateral_step_value_e6: u128,
            interest_rate_step_value_e12: i128,
            owner: AccountId,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut VaultContract| {
                instance.oracle_address = oracle_address;
                instance.collateral.collateral_token_address = collateral_token_address;
                instance.emit.emited_token_address = stable_token_address;
                instance.spgenerate.shares_token_address = shares_token_address;
                instance.spgenerate.shares_profit_controller_address =
                    shares_profit_controller_address;
                instance.spgenerate.sharing_part_e6 = E6;
                instance.current_interest_coefficient_e12 = E12;
                instance.last_interest_coefficient_timestamp = instance.env().block_timestamp();
                instance.maximum_minimum_collateral_coefficient_e6 =
                    maximum_minimum_collateral_coefficient_e6;
                instance.current_minimum_collateral_coefficient_e6 =
                    maximum_minimum_collateral_coefficient_e6;
                instance.collateral_step_value_e6 = collateral_step_value_e6;
                instance.interest_rate_step_value_e12 = interest_rate_step_value_e12;
                instance._init_with_owner(owner);
            })
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn _set_attribute(
            &mut self,
            id: Id,
            key: Vec<u8>,
            value: Vec<u8>,
        ) -> Result<(), VaultError> {
            self._set_attribute(id, key, value)?;
            Ok(())
        }
    }

    impl PSP22Receiver for VaultContract {
        #[ink(message)]
        fn before_received(
            &mut self,
            _operator: AccountId,
            _from: AccountId,
            _value: Balance,
            _data: Vec<u8>,
        ) -> Result<(), PSP22ReceiverError> {
            if self.env().caller() != self.collateral.collateral_token_address {
                return Err(PSP22ReceiverError::TransferRejected(
                    "UnacceptedPsp22".to_string(),
                ));
            }
            Ok(())
        }
    }

    impl Vault for VaultContract {
        // mints a NFT to caller that represent vault
        #[ink(message)]
        fn create_vault(&mut self) -> Result<(), VaultError> {
            ink_env::debug_println!("create_vault START");
            let caller = self.env().caller();
            let next_id = self.next_id;

            self._mint_to(caller, Id::U128(next_id))?;
            self.debt_by_id.insert(&next_id, &(0));
            ink_env::debug_println!("create_vault debt: {}", self._get_debt_by_id(&next_id));
            self.collateral_by_id.insert(&next_id, &(0));
            self.last_interest_coefficient_by_id_e12
                .insert(&next_id, &(self.current_interest_coefficient_e12));
            ink_env::debug_println!("create_vault2 debt: {}", self._get_debt_by_id(&next_id));
            self.next_id += 1;
            ink_env::debug_println!("create_vault STOP");
            Ok(())
        }

        // burns a NFT from a caller that represent vault
        #[ink(message)]
        fn destroy_vault(&mut self, vault_id: u128) -> Result<(), VaultError> {
            ink_env::debug_println!("destroy_vault START");
            let vault_owner: AccountId = match self._owner_of(&Id::U128(vault_id)) {
                Some(v) => v,
                None => return Err(VaultError::OwnerUnexists),
            };
            if self.env().caller() != vault_owner {
                return Err(VaultError::VaultOwnership);
            }
            if self._get_debt_by_id(&vault_id) != 0 {
                return Err(VaultError::HasDebt);
            }
            if self._get_collateral_by_id(&vault_id) != 0 {
                return Err(VaultError::NotEmpty);
            }
            self._burn_from(vault_owner, Id::U128(vault_id))?;
            ink_env::debug_println!("destroy_vault STOP");
            Ok(())
        }

        // deposit collateral to the callers vault
        #[ink(message)]
        fn deposit_collateral(
            &mut self,
            vault_id: u128,
            amount: Balance,
        ) -> Result<(), VaultError> {
            ink_env::debug_println!("deposit_collateral START");
            ink_env::debug_println!(
                "deposit_collateral debt: {}",
                self._get_debt_by_id(&vault_id)
            );
            let vault_owner: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            if self.env().caller() != vault_owner {
                return Err(VaultError::VaultOwnership);
            }

            //transfer in and increase collateral
            let collateral = self._get_collateral_by_id(&vault_id);
            self._transfer_collateral_in(vault_owner, amount)?;
            self.collateral_by_id
                .insert(&vault_id, &(collateral + amount));

            // /event
            self._emit_deposit_event(vault_id, collateral);
            ink_env::debug_println!(
                "deposit_collateral2 debt: {}",
                self._get_debt_by_id(&vault_id)
            );
            ink_env::debug_println!("deposit_collateral STOP");
            Ok(())
        }

        // updates vault debt and withdraws collateral if there is enought
        #[ink(message)]
        fn withdraw_collateral(
            &mut self,
            vault_id: u128,
            amount: Balance,
        ) -> Result<(), VaultError> {
            ink_env::debug_println!("withdraw_collateral START");
            ink_env::debug_println!(
                "withdraw_collateral debt: {}",
                self._get_debt_by_id(&vault_id)
            );
            let vault_owner: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            if self.env().caller() != vault_owner {
                return Err(VaultError::VaultOwnership);
            }

            // check if there is enought collateral to withdraw
            let vault_collateral = self._get_collateral_by_id(&vault_id);
            if amount > vault_collateral {
                return Err(VaultError::CollateralBelowMinimum);
            }

            // check if after withdraw vault is not undercollaterized
            ink_env::debug_println!(
                "check_undercollateralize debt {}",
                self._get_debt_by_id(&vault_id)
            );
            let vault_debt = self._update_vault_debt(vault_id)?;
            let collateral_after = vault_collateral - amount;
            ink_env::debug_println!("check_undercollateralize3 {}", vault_debt);
            if vault_debt * self.current_minimum_collateral_coefficient_e6
                > self._collateral_value_e6(collateral_after)
            //TODO something is wrong here
            {
                return Err(VaultError::CollateralBelowMinimum);
            }

            // transfer out and decrease collateral
            ink_env::debug_println!("transfer_out");
            self.collateral_by_id.insert(&vault_id, &collateral_after);
            ink_env::debug_println!("transfer_out2");
            self._transfer_collateral_out(vault_owner, amount)?;

            //event
            ink_env::debug_println!("event");
            self._emit_deposit_event(vault_id, collateral_after);
            ink_env::debug_println!("withdraw_collateral STOP");

            Ok(())
        }

        // updates vault and borrows tokens if possible
        #[ink(message)]
        #[brush::modifiers(when_not_paused)]
        fn borrow_token(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError> {
            ink_env::debug_println!("borrow_token START");
            ink_env::debug_println!("amount: {}", amount);
            ink_env::debug_println!("debt: {}", self._get_debt_by_id(&vault_id));
            let vault_owner: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            if self.env().caller() != vault_owner {
                return Err(VaultError::VaultOwnership);
            }

            // check if after borrow vault is not undercollaterized
            let debt_ceiling: Balance = self._get_debt_ceiling(vault_id);
            let debt = self._update_vault_debt(vault_id)?;
            if debt + amount > debt_ceiling {
                return Err(VaultError::CollateralBelowMinimum);
            }

            // increase debt and borrow tokens
            self.debt_by_id.insert(&vault_id, &(debt + amount));
            PSP22RatedRef::add_account_debt(&self.emit.emited_token_address, vault_owner, amount)?;

            self.total_debt += amount;
            ink_env::debug_println!("amount: {}", amount);
            self._mint_emited_token(vault_owner, amount)?;

            //event
            self._emit_borrow_event(vault_id, amount);
            Ok(())
        }

        // updates debt and pay back some debt
        #[ink(message)]
        fn pay_back_token(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError> {
            if self.env().caller() != self.liquidator_address {
                return Err(VaultError::Liquidator);
            }
            let vault_owner: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            if self.env().caller() != vault_owner {
                return Err(VaultError::VaultOwnership);
            }
            let debt = self._update_vault_debt(vault_id)?;
            if amount >= debt {
                self._burn_emited_token(vault_owner, debt)?;
                self.debt_by_id.insert(&vault_id, &(0));
                PSP22RatedRef::sub_account_debt(
                    &self.emit.emited_token_address,
                    vault_owner,
                    debt,
                )?;
                self.total_debt -= debt;
                self._emit_pay_back_event(vault_id, debt);
            } else {
                self._burn_emited_token(vault_owner, amount)?;
                self.debt_by_id.insert(&vault_id, &(debt - amount));
                PSP22RatedRef::sub_account_debt(
                    &self.emit.emited_token_address,
                    vault_owner,
                    amount,
                )?;
                self.total_debt -= amount;
                self._emit_pay_back_event(vault_id, amount);
            }
            Ok(())
        }
        // if vault has not enough collateral, callers pays back whole debt
        #[ink(message)]
        fn buy_risky_vault(&mut self, vault_id: u128) -> Result<(), VaultError> {
            let caller = self.env().caller();
            let vault_owner: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();

            //check if debt_ceiling >= debt, if it is return, else continiue and buy risky vault
            let debt_ceiling: Balance = self._get_debt_ceiling(vault_id);
            let debt = self._update_vault_debt(vault_id)?;
            if debt_ceiling >= debt {
                return Err(VaultError::CollateralAboveMinimum);
            }

            // regulating vault so it is not undercollaterized
            self._burn_emited_token(caller, debt)?;
            self.debt_by_id.insert(&vault_id, &(0));
            PSP22RatedRef::sub_account_debt(&self.emit.emited_token_address, vault_owner, debt)?;
            self.total_debt -= debt;

            // transferting PSP34 ownership
            self._remove_token(&vault_owner, &Id::U128(vault_id))?;
            self._do_safe_transfer_check(
                &caller,
                &vault_owner,
                &caller,
                &Id::U128(vault_id),
                &Vec::<u8>::new(),
            )?;
            self._add_token(&caller, &Id::U128(vault_id))?;

            // events
            self._emit_pay_back_event(vault_id, debt);
            self._emit_transfer_event(Some(vault_owner), Some(caller), Id::U128(vault_id));

            Ok(())
        }

        #[ink(message)]
        fn be_controlled(
            &mut self,
            current_interest_rate_step: i16,
            current_collateral_step: u16,
            current_stable_coin_interest_rate_step: i16,
        ) -> Result<(), VaultError> {
            let caller = self.env().caller();
            if caller != self.controller_address {
                return Err(VaultError::VaultController);
            }

            self.current_interest_rate_e12 =
                current_interest_rate_step as i128 * self.interest_rate_step_value_e12;

            self.current_minimum_collateral_coefficient_e6 = self
                .maximum_minimum_collateral_coefficient_e6
                - current_collateral_step as u128 * self.collateral_step_value_e6;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        fn set_vault_controller_address(
            &mut self,
            controller_address: AccountId,
        ) -> Result<(), VaultError> {
            self.controller_address = controller_address;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        fn set_oracle_address(&mut self, new_oracle_address: AccountId) -> Result<(), VaultError> {
            self.oracle_address = new_oracle_address;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        fn set_liquidator_address(
            &mut self,
            new_liquidator_address: AccountId,
        ) -> Result<(), VaultError> {
            self.liquidator_address = new_liquidator_address;
            Ok(())
        }
    }

    impl VaultView for VaultContract {
        #[ink(message)]
        fn get_next_id(&mut self) -> u128 {
            self.next_id
        }
        // return total debt
        #[ink(message)]
        fn get_total_debt(&self) -> Balance {
            self.total_debt
        }

        // returns cault collateral and debt
        #[ink(message)]
        fn get_vault_details(&self, vault_id: u128) -> (Balance, Balance) {
            (
                self._get_collateral_by_id(&vault_id),
                self._get_debt_by_id(&vault_id)
                    * self._get_last_interest_coefficient_by_id_e12(&vault_id)
                    / self._get_current_interest_coefficient_e12(),
            )
        }

        // returns maximum debt for a vault
        #[ink(message)]
        fn get_debt_ceiling(&self, vault_id: u128) -> Balance {
            self._get_debt_ceiling(vault_id)
        }

        #[ink(message)]
        fn get_vault_controller_address(&self) -> AccountId {
            self.controller_address
        }

        #[ink(message)]
        fn get_oracle_address(&self) -> AccountId {
            self.oracle_address
        }

        #[ink(message)]
        fn get_liquidator_address(&self) -> AccountId {
            self.liquidator_address
        }
    }
    impl VaultContractCheck for VaultContract {}

    #[ink(event)]
    pub struct Deposit {
        #[ink(topic)]
        vault_id: u128,
        current_collateral: Balance,
    }
    #[ink(event)]
    pub struct Withdraw {
        #[ink(topic)]
        vault_id: u128,
        current_collateral: Balance,
    }
    #[ink(event)]
    pub struct Borrow {
        #[ink(topic)]
        vault_id: u128,
        borrowed: Balance,
    }
    #[ink(event)]
    pub struct PayBack {
        #[ink(topic)]
        vault_id: u128,
        pay_backed: Balance,
    }

    impl VaultInternal for VaultContract {
        fn _emit_deposit_event(&self, _vault_id: u128, _current_collateral: Balance) {
            self.env().emit_event(Deposit {
                vault_id: _vault_id,
                current_collateral: _current_collateral,
            });
        }

        fn _emit_withdraw_event(&self, _vault_id: u128, _current_collateral: Balance) {
            self.env().emit_event(Withdraw {
                vault_id: _vault_id,
                current_collateral: _current_collateral,
            });
        }

        fn _emit_borrow_event(&self, _vault_id: u128, _borrowed: Balance) {
            self.env().emit_event(Borrow {
                vault_id: _vault_id,
                borrowed: _borrowed,
            });
        }

        fn _emit_pay_back_event(&self, _vault_id: u128, _pay_backed: Balance) {
            self.env().emit_event(PayBack {
                vault_id: _vault_id,
                pay_backed: _pay_backed,
            });
        }

        // return maximal debt for a vault
        fn _get_debt_ceiling(&self, vault_id: u128) -> Balance {
            ink_env::debug_println!("_get_debt_ceiling:");
            let debt_ceiling = self._vault_collateral_value_e6(vault_id) * E6
                / self.current_minimum_collateral_coefficient_e6;
            debt_ceiling
        }

        // returns value of vaults collateral
        fn _vault_collateral_value_e6(&self, vault_id: u128) -> u128 {
            ink_env::debug_println!("_vault_collateral_value_e6:");
            let collateral = self._get_collateral_by_id(&vault_id);
            self._collateral_value_e6(collateral)
        }

        // collateral amount -> collateral value
        fn _collateral_value_e6(&self, collateral: Balance) -> u128 {
            ink_env::debug_println!("_collateral_value_e6:");
            let collateral_price_e6 = OraclingRef::get_azero_usd_price_e6(&self.oracle_address);
            collateral * collateral_price_e6 / COLLATERAL_DECIMALS
        }

        // updates current interest coefficient, updates vaults debt and increments stored interest
        fn _update_vault_debt(&mut self, vault_id: u128) -> Result<Balance, VaultError> {
            // get state
            let current_interest_coefficient_e12 = self._update_current_interest_coefficient_e12();
            let last_interest_coefficient_e12 =
                self._get_last_interest_coefficient_by_id_e12(&vault_id);
            let debt = self._get_debt_by_id(&vault_id);
            ink_env::debug_println!("debt: {}", debt);
            ink_env::debug_println!(
                "current_interest_coefficient_e12: {}",
                current_interest_coefficient_e12
            );
            ink_env::debug_println!(
                "last_interest_coefficient_e12: {}",
                last_interest_coefficient_e12
            );
            // update
            let mut updated_debt =
                debt * current_interest_coefficient_e12 / last_interest_coefficient_e12;
            if updated_debt != 0 {
                updated_debt += 1; // round up
            }
            ink_env::debug_println!("updated_debt: {}", updated_debt);
            let vault_owner = self._owner_of(&Id::U128(vault_id)).unwrap_or_default(); //there will always be non default owner as owner must be caller. it is chacked before each _update_vault call
            if updated_debt > debt {
                self._add_profit_and_increase_shares_minting_allowance(
                    updated_debt - debt,
                    vault_owner,
                );
                PSP22RatedRef::add_account_debt(
                    &self.emit.emited_token_address,
                    vault_owner,
                    updated_debt - debt,
                )?;
            } else if updated_debt < debt {
                self._sub_profit(debt - updated_debt);
                PSP22RatedRef::sub_account_debt(
                    &self.emit.emited_token_address,
                    vault_owner,
                    debt - updated_debt,
                )?;
            }
            //TODO calculate share toekn rewards and mint
            self.debt_by_id.insert(&vault_id, &updated_debt);
            self.last_interest_coefficient_by_id_e12
                .insert(&vault_id, &current_interest_coefficient_e12);

            Ok(updated_debt)
        }

        // calculates, updates and returns current interest coefficient
        fn _update_current_interest_coefficient_e12(&mut self) -> u128 {
            let block_timestamp = self.env().block_timestamp();
            let last_block_timestamp = self.last_interest_coefficient_timestamp;
            if block_timestamp > last_block_timestamp {
                self.last_interest_coefficient_timestamp = block_timestamp;
                let interest_rate: i128 = self.current_interest_rate_e12;
                self.current_interest_coefficient_e12 = self.current_interest_coefficient_e12
                    * (E12 as i128
                        + (block_timestamp - last_block_timestamp) as i128 * interest_rate)
                        as u128
                    / E12;
            }
            self.current_interest_coefficient_e12
        }

        // calculates and retuns current interest coefficient
        fn _get_current_interest_coefficient_e12(&self) -> u128 {
            let block_timestamp = self.env().block_timestamp();
            let last_block_timestamp = self.last_interest_coefficient_timestamp;
            let mut ret = self.current_interest_coefficient_e12;
            if block_timestamp > last_block_timestamp {
                let interest_rate = self.current_interest_rate_e12;
                ret = ret
                    * (E12 as i128
                        + (block_timestamp - last_block_timestamp) as i128 * interest_rate)
                        as u128
                    / E12;
            }
            ret
        }

        // returns vaule from mapping
        fn _get_debt_by_id(&self, vault_id: &u128) -> Balance {
            self.debt_by_id.get(&vault_id).unwrap_or(0)
        }

        // returns value from mapping
        fn _get_collateral_by_id(&self, vault_id: &u128) -> Balance {
            self.collateral_by_id.get(&vault_id).unwrap_or(0)
        }

        // returns value from mapping
        fn _get_last_interest_coefficient_by_id_e12(&self, vault_id: &u128) -> Balance {
            self.last_interest_coefficient_by_id_e12
                .get(&vault_id)
                .unwrap_or(0)
        }
    }

    #[ink(event)]
    pub struct OwnershipTransferred {
        #[ink(topic)]
        previous_owner: Option<AccountId>,
        #[ink(topic)]
        new_owner: Option<AccountId>,
    }
    impl OwnableInternal for VaultContract {
        fn _emit_ownership_transferred_event(
            &self,
            _previous_owner: Option<AccountId>,
            _new_owner: Option<AccountId>,
        ) {
            self.env().emit_event(OwnershipTransferred {
                previous_owner: _previous_owner,
                new_owner: _new_owner,
            });
        }
    }
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
    impl PausableInternal for VaultContract {
        /// User must override this method in their contract.
        fn _emit_paused_event(&self, _account: AccountId) {
            self.env().emit_event(Paused { by: Some(_account) });
        }

        /// User must override this method in their contract.
        fn _emit_unpaused_event(&self, _account: AccountId) {
            self.env().emit_event(Unpaused { by: Some(_account) });
        }
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: Id,
    }
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        #[ink(topic)]
        id: Option<Id>,
        approved: bool,
    }
    impl PSP34Internal for VaultContract {
        /// Emits transfer event. This method must be implemented in derived implementation
        fn _emit_transfer_event(&self, _from: Option<AccountId>, _to: Option<AccountId>, _id: Id) {
            self.env().emit_event(Transfer {
                from: _from,
                to: _to,
                id: _id,
            })
        }

        /// Emits approval event. This method must be implemented in derived implementation
        fn _emit_approval_event(
            &self,
            _from: AccountId,
            _to: AccountId,
            _id: Option<Id>,
            approved: bool,
        ) {
            self.env().emit_event(Approval {
                from: _from,
                to: _to,
                id: _id,
                approved: approved,
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use brush::test_utils::{accounts, change_caller};
        use brush::traits::AccountId;
        use ink_lang as ink;
        type Event = <VaultContract as ::ink_lang::reflect::ContractEventBase>::Type;
        use ink_env::test::DefaultAccounts;
        use ink_env::DefaultEnvironment;

        #[ink::test]
        fn constructor_works() {
            // Constructor works.
            let accounts = accounts();
            let mut vault = VaultContract::new(accounts.bob, accounts.charlie, accounts.alice);
            // Transfer event triggered during initial construction.
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 1);
            assert_eq!(vault.owner(), accounts.alice);
            // Get the token total supply.

            assert_eq!(vault.get_collateral_token_address(), accounts.bob);
            assert_eq!(vault.get_emited_token_address(), accounts.charlie);
            assert_eq!(vault.get_vault_controller_address(), accounts.alice);
        }
    }
}
