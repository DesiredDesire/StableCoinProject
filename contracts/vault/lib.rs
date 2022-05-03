#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod vault {
    //TODO oprocentowanie debt
    use brush::contracts::psp34::PSP34Internal;
    use brush::{contracts::ownable::*, contracts::pausable::*, contracts::psp34::*, modifiers};
    use ink_lang::codegen::EmitEvent;
    use ink_lang::codegen::Env;
    use ink_prelude::vec::Vec;
    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;
    use stable_coin_project::impls::collateralling::*;
    use stable_coin_project::impls::eating::*;
    use stable_coin_project::impls::emiting::*;
    use stable_coin_project::traits::vault::*;

    // const U128MAX: u128 = 340282366920938463463374607431768211455;

    #[ink(storage)]
    #[derive(
        Default,
        SpreadAllocate,
        OwnableStorage,
        PausableStorage,
        PSP34Storage,
        CollaterallingStorage,
        EmitingStorage,
        EatingStorage,
    )]
    pub struct VaultContract {
        #[OwnableStorageField]
        ownable: OwnableData,
        #[PausableStorageField]
        pause: PausableData,
        #[PSP34StorageField]
        psp34: PSP34Data,
        #[CollaterallingStorageField]
        collateral: CollaterallingData,
        #[EmitingStorageField]
        emit: EmitingData,
        #[EatingStorageField]
        eat: EatingData,

        pub collateral_by_id: Mapping<u128, Balance>,
        pub debt_by_id: Mapping<u128, Balance>,
        pub total_debt: Balance,
        pub price_feed_address: AccountId,
        pub minimum_collateral_coefficient_e6: u128,
        pub next_id: u128,

        pub last_interest_coefficient_by_id_e12: Mapping<u128, u128>,
        pub current_interest_coefficient_e12: u128,
        pub interest_coefficient_feeder_address: AccountId,
        pub last_interest_update: u32,

        pub stored_interest: Balance,
    }
    impl Ownable for VaultContract {}
    impl Pausable for VaultContract {}
    impl PSP34 for VaultContract {}
    impl EmitingInternal for VaultContract {}
    impl Emiting for VaultContract {}
    impl CollaterallingInternal for VaultContract {}
    impl Collateralling for VaultContract {}
    impl Eating for VaultContract {}

    impl Vault for VaultContract {
        #[ink(message)]
        fn create_vault(&mut self) -> Result<(), VaultError> {
            let caller = self.env().caller();
            let next_id = self.next_id;
            self._mint_to(caller, Id::U128(next_id))?;
            self.next_id += 1;
            Ok(())
        }
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

            self._transfer_collateral_in(vault_owner, amount)?;

            let collateral = self._get_collateral_by_id(&vault_id)?;
            self.collateral_by_id
                .insert(&vault_id, &(collateral + amount));
            self._emit_deposit_event(vault_id, collateral);
            Ok(())
        }

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
            let vault_collateral = self._get_collateral_by_id(&vault_id)?;
            if amount > vault_collateral {
                return Err(VaultError::CollateralBelowMinimum);
            }
            let vault_debt = self._update_vault_debt(&vault_id)?;
            let collateral_after = vault_collateral - amount;
            if vault_debt * self.minimum_collateral_coefficient_e6
                >= self._collateral_value_e6(collateral_after).unwrap_or(0)
            {
                return Err(VaultError::CollateralBelowMinimum);
            }
            self.collateral_by_id.insert(&vault_id, &collateral_after);
            self._transfer_collateral_out(vault_owner, amount)?;
            self._emit_deposit_event(vault_id, collateral_after);

            Ok(())
        }

        #[ink(message)]
        fn get_debt_ceiling(&self, vault_id: u128) -> Result<Balance, VaultError> {
            match self._get_debt_ceiling(vault_id) {
                Ok(v) => Ok(v),
                Err(e) => {
                    return Err(e);
                }
            }
        }

        #[ink(message)]
        #[brush::modifiers(when_not_paused)]
        fn borrow_token(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError> {
            let vault_owner: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            if self.env().caller() != vault_owner {
                return Err(VaultError::VaultOwnership);
            }
            let debt_ceiling: Balance = match self._get_debt_ceiling(vault_id) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e);
                }
            };
            let debt = self._get_debt_by_id(&vault_id)?;
            if debt + amount >= debt_ceiling {
                return Err(VaultError::CollateralBelowMinimum);
            }
            self.debt_by_id.insert(&vault_id, &(debt + amount));
            self.total_debt += amount;
            self._mint_emited_token(vault_owner, amount)?;
            self._emit_borrow_event(vault_id, debt + amount);
            Ok(())
        }
        #[ink(message)]
        fn pay_back_token(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError> {
            let vault_owner: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            if self.env().caller() != vault_owner {
                return Err(VaultError::VaultOwnership);
            }
            let debt = self._get_debt_by_id(&vault_id)?;
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
        #[ink(message)]
        fn buy_risky_vault(&mut self, vault_id: u128) -> Result<(), VaultError> {
            let caller = self.env().caller();
            let vault_owner: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            let debt_ceiling: Balance = match self._get_debt_ceiling(vault_id) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e);
                }
            };
            let debt = self._get_debt_by_id(&vault_id)?;

            if debt_ceiling > debt {
                return Err(VaultError::CollateralAboveMinimum);
            }

            let minimum_to_pay = (debt - debt_ceiling) + 1;
            self._burn_emited_token(caller, minimum_to_pay)?;

            self.debt_by_id.insert(&vault_id, &(debt - minimum_to_pay));
            self.total_debt -= minimum_to_pay;
            self._emit_pay_back_event(vault_id, debt - minimum_to_pay);
            self._remove_token(&vault_owner, &Id::U128(vault_id))?;
            self._do_safe_transfer_check(
                &caller,
                &vault_owner,
                &caller,
                &Id::U128(vault_id),
                &Vec::<u8>::new(),
            )?;
            self._add_token(&caller, &Id::U128(vault_id))?;
            self._emit_transfer_event(Some(vault_owner), Some(caller), Id::U128(vault_id));
            Ok(())
        }
    }

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

        fn _get_debt_ceiling(&self, vault_id: u128) -> Result<Balance, VaultError> {
            let debt_ceiling =
                self._vault_collateral_value_e6(vault_id)? / self.minimum_collateral_coefficient_e6;
            Ok(debt_ceiling)
        }

        fn _collateral_value_e6(&self, collateral: Balance) -> Result<Balance, VaultError> {
            let collateral_price_e6 = self._get_collateral_price_e6()?;
            Ok(collateral * collateral_price_e6)
        }

        fn _vault_collateral_value_e6(&self, value_id: u128) -> Result<Balance, VaultError> {
            let collateral = self._get_collateral_by_id(&value_id)?;
            self._collateral_value_e6(collateral)
        }

        fn _get_collateral_price_e6(&self) -> Result<u128, VaultError> {
            Ok(1000000)
        }

        fn _update_vault_debt(&mut self, vault_id: &u128) -> Result<Balance, VaultError> {
            let current_interest_coefficient_e12 =
                self._update_cuurent_interest_coefficient_e12()?;
            let last_interest_coefficient_e12 =
                self._get_last_interest_coefficient_by_id_e12(&vault_id)?;
            let debt = self._get_debt_by_id(&vault_id)?;
            let updated_debt =
                debt * current_interest_coefficient_e12 / last_interest_coefficient_e12;
            self.stored_interest += updated_debt - debt;
            self.debt_by_id.insert(&vault_id, &updated_debt);
            Ok(updated_debt)
        }

        fn _update_cuurent_interest_coefficient_e12(&mut self) -> Result<u128, VaultError> {
            let block_number: u32 = self.env().block_number();
            if block_number > self.last_interest_update {
                self.last_interest_update = block_number;
                self.current_interest_coefficient_e12 = 1000000000000000; //RODO get interest coefficient from
            }
            Ok(self.current_interest_coefficient_e12)
        }

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
            minimum_collateral_coefficient_e6: u128,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut VaultContract| {
                instance.collateral.collateral_token_address = collateral_token_address;
                instance.emit.emited_token_address = emited_token_address;
                instance.minimum_collateral_coefficient_e6 = minimum_collateral_coefficient_e6;
                instance.ownable.owner = instance.env().caller();
                instance.eat.feeder_address = feeder_address;
            })
        }
        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn pause(&mut self) -> Result<(), VaultError> {
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

    impl EatingInternal for VaultContract {
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
}
