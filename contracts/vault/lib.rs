#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod vault {
    use brush::contracts::psp34::PSP34Internal;
    use brush::{contracts::ownable::*, contracts::pausable::*, contracts::psp34::*, modifiers};
    use ink_lang::codegen::EmitEvent;
    use ink_lang::codegen::Env;
    use ink_prelude::vec::Vec;
    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;
    use stable_coin_project::impls::collateralling::*;
    use stable_coin_project::impls::emitting::*;
    use stable_coin_project::impls::vault_eating::*;
    use stable_coin_project::traits::vault::*;

    // const U128MAX: u128 = 340282366920938463463374607431768211455;
    const E12: u128 = 10 ^ 12;

    #[ink(storage)]
    #[derive(
        Default,
        SpreadAllocate,
        OwnableStorage,
        PausableStorage,
        PSP34Storage,
        CollaterallingStorage,
        EmittingStorage,
        VEatingStorage,
    )]
    pub struct VaultContract {
        #[OwnableStorageField]
        ownable: OwnableData,
        #[PausableStorageField]
        pause: PausableData,
        #[PSP34StorageField] // vault ownership
        psp34: PSP34Data,
        #[CollaterallingStorageField] // collateral_token_address
        collateral: CollaterallingData,
        #[EmittingStorageField] // emited_token_address
        emit: EmittingData,
        #[VEatingStorageField] // feeder_contract_ address
        eat: VEatingData,

        pub collateral_by_id: Mapping<u128, Balance>,
        pub debt_by_id: Mapping<u128, Balance>,
        pub last_interest_coefficient_by_id_e12: Mapping<u128, u128>, // the last interest coefficient (acumulated interest) used for vault with id
        pub current_interest_coefficient_e12: u128, // the current interest coefficient (acmulated interest)
        pub last_interest_coefficient_e12_update: u128, // last block number when current_interest_coefficient_e12 was updated
        pub total_debt: Balance,
        pub next_id: u128,

        pub earned_interest: Balance, // amount of emitted token that can be mint, collecting debt interest
    }
    impl Ownable for VaultContract {} // owner can pause contract
    impl Pausable for VaultContract {} // when paused borrowing is imposible
    impl PSP34 for VaultContract {} // PSP34 is prove of being vault_owner
    impl EmittingInternal for VaultContract {} // minting and burning emited_token
    impl Emitting for VaultContract {} // emited_amount() = minted - burned
    impl CollaterallingInternal for VaultContract {} // transfer in, transfer out
    impl Collateralling for VaultContract {} // rescue[only_owner], amount of collaterall
    impl VEating for VaultContract {} // source of collateral_price, interest_rate and minimum_collateral_amount

    impl Vault for VaultContract {
        // mints a NFT to caller that represent vault
        #[ink(message)]
        fn create_vault(&mut self) -> Result<(), VaultError> {
            let caller = self.env().caller();
            let next_id = self.next_id;
            self._mint_to(caller, Id::U128(next_id))?;
            self.next_id += 1;
            Ok(())
        }
        // burns a NFT from a caller that represent vault
        #[ink(message)]
        fn destroy_vault(&mut self, vault_id: u128) -> Result<(), VaultError> {
            let vault_owner: AccountId = match self._owner_of(&Id::U128(vault_id)) {
                Some(v) => v,
                None => return Err(VaultError::OwnerUnexists),
            };
            if self.env().caller() != vault_owner {
                return Err(VaultError::VaultOwnership);
            }
            if self._get_debt_by_id(&vault_id)? != 0 {
                return Err(VaultError::HasDebt);
            }
            if self._get_collateral_by_id(&vault_id)? != 0 {
                return Err(VaultError::NotEmpty);
            }
            self._burn_from(vault_owner, Id::U128(vault_id))?;
            Ok(())
        }

        // deposit collateral to the callers vault
        #[ink(message)]
        fn deposit_collateral(
            &mut self,
            vault_id: u128,
            amount: Balance,
        ) -> Result<(), VaultError> {
            let vault_owner: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            if self.env().caller() != vault_owner {
                return Err(VaultError::VaultOwnership);
            }

            // transfer in and increase collateral
            let collateral = self._get_collateral_by_id(&vault_id)?;
            self._transfer_collateral_in(vault_owner, amount)?;
            self.collateral_by_id
                .insert(&vault_id, &(collateral + amount));

            // /event
            self._emit_deposit_event(vault_id, collateral);
            Ok(())
        }

        // updates vault debt and withdraws collateral if there is enought
        #[ink(message)]
        fn withdraw_collateral(
            &mut self,
            vault_id: u128,
            amount: Balance,
        ) -> Result<(), VaultError> {
            let vault_owner: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            if self.env().caller() != vault_owner {
                return Err(VaultError::VaultOwnership);
            }

            // check if there is enought collateral to withdraw
            let vault_collateral = self._get_collateral_by_id(&vault_id)?;
            if amount > vault_collateral {
                return Err(VaultError::CollateralBelowMinimum);
            }

            // check if after withdraw vault is not undercollaterized
            let vault_debt = self._update_vault_debt(&vault_id)?;
            let collateral_after = vault_collateral - amount;
            if vault_debt * self.eat_minimum_collateral_coefficient_e6()?
                >= self._collateral_value_e6(collateral_after).unwrap_or(0)
            {
                return Err(VaultError::CollateralBelowMinimum);
            }

            // transfer out and decrease collateral
            self.collateral_by_id.insert(&vault_id, &collateral_after);
            self._transfer_collateral_out(vault_owner, amount)?;

            //event
            self._emit_deposit_event(vault_id, collateral_after);
            Ok(())
        }

        // returns maximum debt for a vault
        #[ink(message)]
        fn get_debt_ceiling(&self, vault_id: u128) -> Result<Balance, VaultError> {
            match self._get_debt_ceiling(vault_id) {
                Ok(v) => Ok(v),
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // updates vault and borrows tokens if possible
        #[ink(message)]
        #[brush::modifiers(when_not_paused)]
        fn borrow_token(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError> {
            let vault_owner: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            if self.env().caller() != vault_owner {
                return Err(VaultError::VaultOwnership);
            }

            // check if after borrow vault is not undercollaterized
            let debt_ceiling: Balance = match self._get_debt_ceiling(vault_id) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e);
                }
            };
            let debt = self._update_vault_debt(&vault_id)?;
            if debt + amount >= debt_ceiling {
                return Err(VaultError::CollateralBelowMinimum);
            }

            // increase debt and borrow tokens
            self.debt_by_id.insert(&vault_id, &(debt + amount));
            self.total_debt += amount;
            self._mint_emited_token(vault_owner, amount)?;

            //event
            self._emit_borrow_event(vault_id, debt + amount);
            Ok(())
        }

        // updates debt and pay back some debt
        #[ink(message)]
        fn pay_back_token(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError> {
            let vault_owner: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            if self.env().caller() != vault_owner {
                return Err(VaultError::VaultOwnership);
            }
            let debt = self._update_vault_debt(&vault_id)?;
            if amount >= debt {
                self._burn_emited_token(vault_owner, debt)?;
                self.debt_by_id.insert(&vault_id, &(0));
                self.total_debt -= debt;
                self._emit_pay_back_event(vault_id, 0);
            } else {
                self._burn_emited_token(vault_owner, amount)?;
                self.debt_by_id.insert(&vault_id, &(debt - amount));
                self.total_debt -= amount;
                self._emit_pay_back_event(vault_id, debt - amount);
            }
            Ok(())
        }
        // if vault has not enough collateral, callers pays back some debt than transfer vault to caller
        #[ink(message)]
        fn buy_risky_vault(&mut self, vault_id: u128) -> Result<(), VaultError> {
            let caller = self.env().caller();
            let vault_owner: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();

            //check if debt_ceiling >= debt, if it is return, else continiue and buy risky vault
            let debt_ceiling: Balance = match self._get_debt_ceiling(vault_id) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e);
                }
            };
            let debt = self._update_vault_debt(&vault_id)?;
            if debt_ceiling >= debt {
                return Err(VaultError::CollateralAboveMinimum);
            }

            // regulating vault so it is not undercollaterized
            let minimum_to_pay = (debt - debt_ceiling) + 1;
            self._burn_emited_token(caller, minimum_to_pay)?;
            self.debt_by_id.insert(&vault_id, &(debt - minimum_to_pay));
            self.total_debt -= minimum_to_pay;

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
            self._emit_pay_back_event(vault_id, debt - minimum_to_pay);
            self._emit_transfer_event(Some(vault_owner), Some(caller), Id::U128(vault_id));

            Ok(())
        }
    }
    pub trait VaultView {
        fn get_total_debt(&self) -> Balance;
        fn get_vault_details(&self, vault_id: u128) -> Result<(Balance, Balance), VaultError>;
    }
    impl VaultView for VaultContract {
        // return total debt
        fn get_total_debt(&self) -> Balance {
            self.total_debt
        }

        // returns cault collateral and debt
        fn get_vault_details(&self, vault_id: u128) -> Result<(Balance, Balance), VaultError> {
            Ok((
                self._get_collateral_by_id(&vault_id)?,
                self._get_debt_by_id(&vault_id)?
                    * self._get_last_interest_coefficient_by_id_e12(&vault_id)?
                    / self._get_current_interest_coefficient_e12()?,
            ))
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
        current_debt: Balance,
    }
    #[ink(event)]
    pub struct PayBack {
        #[ink(topic)]
        vault_id: u128,
        current_debt: Balance,
    }

    impl VaultInternal for VaultContract {
        fn _emit_deposit_event(&self, _vault_id: u128, _current_collateral: Balance) {
            self.env().emit_event(Deposit {
                vault_id: _vault_id,
                current_collateral: _current_collateral,
            });
        }

        fn _emit_withdraw_event(&self, _vault_id: u128, _current_collateral: Balance) {
            self.env().emit_event(Deposit {
                vault_id: _vault_id,
                current_collateral: _current_collateral,
            });
        }

        fn _emit_borrow_event(&self, _vault_id: u128, _current_debt: Balance) {
            self.env().emit_event(Borrow {
                vault_id: _vault_id,
                current_debt: _current_debt,
            });
        }

        fn _emit_pay_back_event(&self, _vault_id: u128, _current_debt: Balance) {
            self.env().emit_event(PayBack {
                vault_id: _vault_id,
                current_debt: _current_debt,
            });
        }

        // return maximal debt for a vault
        fn _get_debt_ceiling(&self, vault_id: u128) -> Result<Balance, VaultError> {
            let debt_ceiling = self._vault_collateral_value_e6(vault_id)?
                / self.eat_minimum_collateral_coefficient_e6()?;
            Ok(debt_ceiling)
        }

        // collateral amount -> collateral value
        fn _collateral_value_e6(&self, collateral: Balance) -> Result<Balance, VaultError> {
            let collateral_price_e6 = self.eat_collateral_price_e6()?;
            Ok(collateral * collateral_price_e6)
        }

        // returns value of vaults collateral
        fn _vault_collateral_value_e6(&self, value_id: u128) -> Result<Balance, VaultError> {
            let collateral = self._get_collateral_by_id(&value_id)?;
            self._collateral_value_e6(collateral)
        }

        // updates current interest coefficient, updates vaults debt and increments stored interest
        fn _update_vault_debt(&mut self, vault_id: &u128) -> Result<Balance, VaultError> {
            // get state
            let current_interest_coefficient_e12 =
                self._update_cuurent_interest_coefficient_e12()?;
            let last_interest_coefficient_e12 =
                self._get_last_interest_coefficient_by_id_e12(&vault_id)?;
            let debt = self._get_debt_by_id(&vault_id)?;

            // update
            let updated_debt =
                debt * current_interest_coefficient_e12 / last_interest_coefficient_e12;
            self.earned_interest += updated_debt - debt;
            self.debt_by_id.insert(&vault_id, &updated_debt);
            self.last_interest_coefficient_by_id_e12
                .insert(&vault_id, &current_interest_coefficient_e12);

            Ok(updated_debt)
        }

        // calculates, updates and returns current interest coefficient
        fn _update_cuurent_interest_coefficient_e12(&mut self) -> Result<u128, VaultError> {
            let block_number: u128 = self.env().block_number() as u128;
            let last_block_number = self.last_interest_coefficient_e12_update;
            if block_number > last_block_number {
                self.last_interest_coefficient_e12_update = block_number;
                let interest_rate = self.eat_interest_rate_e12()?;
                self.current_interest_coefficient_e12 = self.current_interest_coefficient_e12
                    * (E12 + (block_number - last_block_number) * interest_rate)
                    / E12;
            }
            Ok(self.current_interest_coefficient_e12)
        }

        // calculates and retuns current interest coefficient
        fn _get_current_interest_coefficient_e12(&self) -> Result<u128, VaultError> {
            let block_number: u128 = self.env().block_number() as u128;
            let last_block_number = self.last_interest_coefficient_e12_update;
            let mut ret = self.current_interest_coefficient_e12;
            if block_number > last_block_number {
                let interest_rate = self.eat_interest_rate_e12()?;
                ret = ret * (E12 + (block_number - last_block_number) * interest_rate) / E12;
            }
            Ok(ret)
        }

        // returns vaule from mapping
        fn _get_debt_by_id(&self, vault_id: &u128) -> Result<Balance, VaultError> {
            match self.debt_by_id.get(&vault_id) {
                Some(v) => {
                    return Ok(v);
                }
                None => {
                    return Err(VaultError::DebtUnexists);
                }
            }
        }

        // returns value from mapping
        fn _get_collateral_by_id(&self, vault_id: &u128) -> Result<Balance, VaultError> {
            match self.collateral_by_id.get(&vault_id) {
                Some(v) => {
                    return Ok(v);
                }
                None => {
                    return Err(VaultError::CollateralUnexists);
                }
            }
        }

        // returns value from mapping
        fn _get_last_interest_coefficient_by_id_e12(
            &self,
            vault_id: &u128,
        ) -> Result<Balance, VaultError> {
            match self.last_interest_coefficient_by_id_e12.get(&vault_id) {
                Some(v) => {
                    return Ok(v);
                }
                None => {
                    return Err(VaultError::CollateralUnexists);
                }
            }
        }
    }

    impl VaultContract {
        #[ink(constructor)]
        pub fn new(
            collateral_token_address: AccountId,
            emited_token_address: AccountId,
            feeder_address: AccountId,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut VaultContract| {
                instance.collateral.collateral_token_address = collateral_token_address;
                instance.emit.emited_token_address = emited_token_address;
                instance.ownable.owner = instance.env().caller();
                instance.eat.feeder_address = feeder_address;
            })
        }
        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn pause(&mut self) -> Result<(), VaultError> {
            //TODO check if pause is implementen in Pausable for VaultContract
            self._pause()
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
    #[ink(event)]
    pub struct FeederChanged {
        #[ink(topic)]
        old_feeder: Option<AccountId>,
        #[ink(topic)]
        new_feeder: Option<AccountId>,
        #[ink(topic)]
        caller: AccountId,
    }
    impl VEatingInternal for VaultContract {
        fn _emit_feeder_changed_event(
            &self,
            _old_feeder: Option<AccountId>,
            _new_feeder: Option<AccountId>,
            _caller: AccountId,
        ) {
            self.env().emit_event(FeederChanged {
                old_feeder: _old_feeder,
                new_feeder: _new_feeder,
                caller: _caller,
            })
        }
    }

    // #[ink::test]
    // fn constructor_works() {
    //     // Constructor works.
    //     let accounts = accounts();
    //     let mut vault = VaultContract::new(None, None, DECIMALS);
    //     // Transfer event triggered during initial construction.
    //     let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
    //     assert_eq!(emitted_events.len(), 1);
    //     assert_e1!(vault.owner_of(), accounts.alice);
    //     // Get the token total supply.
    //     assert_eq!(psp22.total_supply(), 0);
    //     assert_eq!(psp22.taxed_supply(), 0);
    //     assert_eq!(psp22.untaxed_supply(), 0);
    //     assert_eq!(psp22.undivided_taxed_supply(), 0);
    // }
}
