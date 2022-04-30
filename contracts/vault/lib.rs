#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod vault {
    //TODO oprocentowanie debt
    use brush::{
        contracts::ownable::*, contracts::pausable::*, contracts::psp22::extensions::burnable::*,
        contracts::psp22::extensions::mintable::*, contracts::psp34::*, modifiers,
    };
    use ink_lang::codegen::Env;
    use ink_prelude::vec::Vec;
    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;
    use stable_coin_project::impls::eating::*;
    use stable_coin_project::impls::emiting::*;
    use stable_coin_project::traits::eating::*;
    use stable_coin_project::traits::vault::*;

    const U128MAX: u128 = 340282366920938463463374607431768211455;

    #[ink(storage)]
    #[derive(
        Default,
        SpreadAllocate,
        PSP34Storage,
        PausableStorage,
        EmitingStorage,
        EatingStorage,
        OwnableStorage,
    )]
    pub struct VaultContract {
        #[OwnableStorageField]
        ownable: OwnableData,
        #[PausableStorageField]
        pause: PausableData,
        #[EmitingStorageField]
        emit: EmitingData,
        #[PSP34StorageField]
        psp34: PSP34Data,
        #[EatingStorageField]
        eat: EatingData,

        pub collateral_by_id: Mapping<u128, Balance>,
        pub debt_by_id: Mapping<u128, Balance>,
        pub price_feed_address: AccountId,
        pub collaterall_token_address: AccountId,
        pub stable_coin_token_address: AccountId,
        pub minimum_collateral_coefficient_e6: u128,
        pub last_collateral_price: u128,
        pub next_id: u128,
    }

    impl PSP34 for VaultContract {}
    impl PSP34Internal for VaultContract {}
    impl Emiting for VaultContract {}
    impl Eating for VaultContract {}

    impl Vault for VaultContract {
        #[ink(message)]
        fn create_vault(&mut self) -> Result<(), VaultError> {
            let caller = self.env().caller();
            let next_id = self.next_id;
            self._mint_to(caller, Id::U128(next_id))?;
            self.next_id += 1;
            Ok(())
            //TODO EMIT EVENT - PSP34::mint will emit event => TODO implement PSP34 event
        }
        #[ink(message)]
        fn destroy_vault(&mut self, vault_id: u128) -> Result<(), VaultError> {
            let vault_owner: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            if self.env().caller() != vault_owner {
                return Err(VaultError::VaultOwnership);
            }

            if self.debt_by_id.get(vault_id).unwrap_or(1) != 0 {
                return Err(VaultError::HasDebt);
            }
            if self.collateral_by_id.get(vault_id).unwrap_or(1) != 0 {
                return Err(VaultError::NotEmpty);
            }
            self._burn_from(vault_owner, Id::U128(vault_id))?;
            Ok(())
            //TODO EMIT EVENT - PSP34::burn will emit event => TODO implement PSP34 event
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

            if let Err(err) = PSP22Ref::transfer_from(
                &self.collaterall_token_address,
                vault_owner,
                self.env().account_id(),
                amount,
                Vec::<u8>::new(),
            ) {
                return Err(From::from(err));
            }
            //TODO EMIT EVENT
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
            let contract = self.env().account_id();
            let collateral_address = self.collaterall_token_address;
            let vault_collateral = self.collateral_by_id.get(&vault_id).unwrap_or(0);
            let vault_debt = self.debt_by_id.get(&vault_id).unwrap_or(U128MAX);
            let collateral_after = vault_collateral - amount;
            if vault_debt * self.minimum_collateral_coefficient_e6
                >= self._collateral_value_e6(collateral_after).unwrap_or(0)
            {
                return Err(VaultError::CollateralBelowMinimum);
            }
            //TODO EMIT EVENT
            match PSP22Ref::transfer_from(
                &(collateral_address),
                contract,
                vault_owner,
                amount,
                Vec::<u8>::new(),
            ) {
                Ok(..) => (),
                Err(e) => {
                    return Err(VaultError::from(e));
                }
            };

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
            let debt = self.debt_by_id.get(vault_id).unwrap_or(U128MAX);
            if debt + amount >= debt_ceiling {
                return Err(VaultError::CollateralBelowMinimum);
            }
            match PSP22MintableRef::mint(&self.stable_coin_token_address, vault_owner, amount) {
                Ok(..) => (),
                Err(e) => {
                    return Err(VaultError::from(e));
                }
            };
            //TODO emitevent
            Ok(())
        }
        #[ink(message)]
        fn pay_back_token(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError> {
            let vault_owner: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            if self.env().caller() != vault_owner {
                return Err(VaultError::VaultOwnership);
            }
            let debt = self.debt_by_id.get(vault_id).unwrap_or(U128MAX);
            if amount >= debt {
                match PSP22BurnableRef::burn(&self.stable_coin_token_address, vault_owner, debt) {
                    Ok(..) => (),
                    Err(e) => {
                        return Err(VaultError::from(e));
                    }
                };
                self.debt_by_id.insert(&vault_id, &(0));
                //TODO emit event
            } else {
                match PSP22BurnableRef::burn(&self.stable_coin_token_address, vault_owner, amount) {
                    Ok(..) => (),
                    Err(e) => {
                        return Err(VaultError::from(e));
                    }
                };
                self.debt_by_id.insert(&vault_id, &(debt - amount));
                //TODO emit event
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
            let debt = self.debt_by_id.get(&vault_id).unwrap_or(U128MAX); //IMPORTANT TODO! udnerstand these unwraps. if something goes wrong one can't rebuy the vault!

            if debt_ceiling > debt {
                return Err(VaultError::CollateralAboveMinimum);
            }

            let minimum_to_pay = (debt - debt_ceiling) + 1;
            match PSP22BurnableRef::burn(&self.stable_coin_token_address, caller, minimum_to_pay) {
                Ok(..) => (),
                Err(e) => {
                    return Err(VaultError::from(e));
                }
            }
            self.debt_by_id.insert(&vault_id, &(debt - minimum_to_pay));
            self._remove_token(&vault_owner, &Id::U128(vault_id))?;
            self._do_safe_transfer_check(
                &caller,
                &vault_owner,
                &caller,
                &Id::U128(vault_id),
                &Vec::<u8>::new(),
            )?;
            self._add_token(&caller, &Id::U128(vault_id))?;
            //TODO emit event
            Ok(())
        }
    }

    impl VaultInternal for VaultContract {
        fn _get_debt_ceiling(&self, vault_id: u128) -> Result<Balance, VaultError> {
            match self._vault_collateral_value_e6(vault_id) {
                Ok(v) => return Ok(v / self.minimum_collateral_coefficient_e6),
                Err(e) => {
                    return Err(e);
                }
            }
        }

        fn _collateral_value_e6(&self, collateral: Balance) -> Result<Balance, VaultError> {
            let collateral_price_e6 = match self._get_collateral_price_e6() {
                Ok(v) => v,
                Err(e) => {
                    return Err(e);
                }
            };
            Ok(collateral * collateral_price_e6)
        }

        fn _vault_collateral_value_e6(&self, value_id: u128) -> Result<Balance, VaultError> {
            let collateral = self.collateral_by_id.get(&value_id).unwrap_or(0);
            self._collateral_value_e6(collateral)
        }

        fn _get_collateral_price_e6(&self) -> Result<u128, VaultError> {
            Ok(1000000)
        }
    }

    impl VaultContract {
        #[ink(constructor)]
        pub fn new(
            collaterall_token_address: AccountId,
            minted_token_address: AccountId,
            minimum_collateral_coefficient_e6: u128,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut VaultContract| {
                instance.collaterall_token_address = collaterall_token_address;
                instance.emit.emited_token_address = minted_token_address;
                instance.minimum_collateral_coefficient_e6 = minimum_collateral_coefficient_e6;
            })
        }
        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn pause(&mut self) -> Result<(), VaultError> {
            self._pause()
        }
    }
}
