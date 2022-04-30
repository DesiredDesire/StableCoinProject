#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod vault {
    use brush::{
        contracts::pausable::*, contracts::psp22::*, contracts::psp34::*, modifier_definition,
        modifiers, traits::Flush,
    };
    use ink_lang::codegen::Env;
    use ink_prelude::vec::Vec;
    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;
    use psp34::extensions::{burnable::*, mintable::*};
    use stable_coin_project::impls::emiting::*;
    use stable_coin_project::traits::vault::*;

    const e4: u128 = 10000;
    const e6: u128 = 1000000;

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, PSP34Storage, PausableStorage, EmitingStorage)]
    pub struct VaultContract {
        #[PausableStorageField]
        pause: PausableData,
        #[EmitingStorageField]
        emit: EmitingData,
        #[PSP34StorageField]
        psp34: PSP34Data,

        pub collateral_by_id: Mapping<u128, Balance>,
        pub debt_by_id: Mapping<u128, Balance>,
        pub collaterall_token_address: AccountId,
        pub minimum_collateral_percentage_e4: u128,
        pub last_collateral_price: u128,
        pub next_id: u128,
    }

    impl PSP34 for VaultContract {}
    impl PSP34Internal for VaultContract {}
    impl Emiting for VaultContract {}

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
            let owner_account_id: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            if self.env().caller() != owner_account_id {
                return Err(VaultError::VaultOwnership);
            }

            if self.debt_by_id.get(vault_id).unwrap_or(1) != 0 {
                return Err(VaultError::HasDebt);
            }
            if self.collateral_by_id.get(vault_id).unwrap_or(1) != 0 {
                return Err(VaultError::NotEmpty);
            }
            self._burn_from(owner_account_id, Id::U128(vault_id));
            Ok(())
            //TODO EMIT EVENT - PSP34::burn will emit event => TODO implement PSP34 event
        }

        #[ink(message)]
        fn deposit_collateral(
            &mut self,
            vault_id: u128,
            amount: Balance,
        ) -> Result<(), VaultError> {
            let owner_account_id: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            if self.env().caller() != owner_account_id {
                return Err(VaultError::VaultOwnership);
            }

            if let Err(err) = PSP22Ref::transfer_from(
                &self.collaterall_token_address,
                owner_account_id,
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
            let owner_account_id: AccountId = self.owner_of(Id::U128(vault_id)).unwrap_or_default();
            if self.env().caller() != owner_account_id {
                return Err(VaultError::VaultOwnership);
            }
            let contract = self.env().account_id();
            let collateral_address = self.collaterall_token_address;
            let vault_collateral = self.collateral_by_id.get(&vault_id).unwrap_or(0);
            let vault_debt = self.debt_by_id.get(&vault_id).unwrap_or(0); //TODO if we set 0 and the true debt is > 0 it is a BIG PROBLEM
            let collateral_after = vault_collateral - amount;
            match self._check_collateral(collateral_after, vault_debt) {
                Ok(v) => (),
                Err(e) => {
                    return Err(VaultError::from(e));
                }
            };
            //TODO EMIT EVENT
            match PSP22Ref::transfer_from(
                &(collateral_address),
                self.env().account_id(),
                self.env().caller(),
                amount,
                Vec::<u8>::new(),
            ) {
                Ok(v) => (),
                Err(e) => {
                    return Err(VaultError::from(e));
                }
            };

            Ok(())
        }

        #[ink(message)]
        fn get_debt_ceiling(&mut self, vault_id: u128) -> Result<Balance, VaultError> {
            match self._vault_collateral_value_e6_view(vault_id) {
                Ok(v) => return Ok(v * e4 / self.minimum_collateral_percentage_e4),
                Err(e) => {
                    return Err(e);
                }
            }
        }

        #[ink(message)]
        fn borrow_token(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError> {
            //WORKING HERE
            Ok(())
        }
        #[ink(message)]
        fn pay_back_token(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError> {
            Ok(())
        }
        #[ink(message)]
        fn buy_risky_vault(&mut self, vault_id: u128) -> () {}
    }

    impl VaultInternal for VaultContract {
        fn _check_collateral(
            &mut self,
            collateral: Balance,
            debt: Balance,
        ) -> Result<(), VaultError> {
            let collateral_value_e6 = match self._collateral_value_e6(collateral) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e);
                }
            };
            let collateralPercentage = collateral_value_e6 / debt;

            if collateralPercentage >= self.minimum_collateral_percentage_e4 {
                return Err(VaultError::CollateralBelowMinimumPercentage);
            }
            Ok(())
        }

        fn _collateral_value_e6(&mut self, collateral: Balance) -> Result<Balance, VaultError> {
            let collateral_price_e6 = match self._update_collateral_price() {
                Ok(v) => v,
                Err(e) => {
                    return Err(e);
                }
            };
            Ok(collateral * collateral_price_e6)
        }

        fn _collateral_value_e6_view(&self, collateral: Balance) -> Result<Balance, VaultError> {
            let collateral_price_e6 = match self._get_collateral_price() {
                Ok(v) => v,
                Err(e) => {
                    return Err(e);
                }
            };
            Ok(collateral * collateral_price_e6)
        }

        fn _vault_collateral_value_e6(&mut self, value_id: u128) -> Result<Balance, VaultError> {
            let collateral = self.collateral_by_id.get(&value_id).unwrap_or(0);
            self._collateral_value_e6(collateral)
        }

        fn _vault_collateral_value_e6_view(&self, value_id: u128) -> Result<Balance, VaultError> {
            let collateral = self.collateral_by_id.get(&value_id).unwrap_or(0);
            self._collateral_value_e6_view(collateral)
        }

        fn _update_collateral_price(&mut self) -> Result<u128, VaultError> {
            self.last_collateral_price = 100000;
            Ok(1000000)
        }

        fn _get_collateral_price(&self) -> Result<u128, VaultError> {
            Ok(1000000)
        }
    }

    impl VaultContract {
        #[ink(constructor)]
        pub fn new(
            collaterall_token_address: AccountId,
            minted_token_address: AccountId,
            minimum_collateral_percentage_e4: u128,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut VaultContract| {
                instance.collaterall_token_address = collaterall_token_address;
                instance.emit.emited_token_address = minted_token_address;
                instance.minimum_collateral_percentage_e4 = minimum_collateral_percentage_e4;
            })
        }
        // fn _get_caller_vault_by_id(vault_id: Id) -> Result<SingleVaultData, VaultError> {
        //     let vault_data = self
        //
        //         _data_by_id
        //         .get(&vault_id)
        //         .ok_or(VaultError::NonExistingVaultError)?;
        //     let owner_account_id = self.owner_of(&vault_id);
        //     if owner_account_id.unwrap_or_default() != self.env().caller() {
        //         return Err(VaultError::VaultOwnershipError);
        //     }
        // }

        // COLLATERALL IS NOT MINTED. STABLE COIN IS MINTED
        // fn _mint_collateral_if_not_zero(
        //     collateral: u128,
        //     to: AccountId,
        // ) -> Result<(), EmitingError> {
        //     if collateral != 0 {
        //         self::Emiting::mint(&to, collateral)?;
        //     }
        //     Ok(())
        // }

        // fn _check_debt_collateral(
        //     &mut self,
        //     collateral: Balance,
        //     debt: Balance,
        // ) -> Result<(), CollateralError> {
        //     let collateral_value_e6 = self
        //         ._calculate_collateral_value_e6(collateral)
        //         .unwrap_or(0)?; //TODO propagate error

        //     let collateralPercentage = collateral_value_e6 / debt;

        //     if collateralPercentage >= self.minimum_collateral_prcentage {
        //         return Err(CollateralError::CollateralBelowMinimumPercentageError);
        //     }
        //     Ok(())
        // }

        // fn _calculate_collateral_value_e6(
        //     &mut self,
        //     collateral: Balance,
        // ) -> Result<Balance, CollateralError> {
        //     let collateral_price_e6 = self.get_eth_price_source(); // assuming that price is multiplied by 10^6. If price is 1.01 then we get 1010000
        //     if collateral_price_e6 == 0 {
        //         return Err(CollateralError::PriceEqualsZeroError);
        //     }
        //     Ok(collateral * collateral_price_e6)
        // }

        // // fn get_token_price_source(&mut self) -> Balance {
        // //     self.token_peg
        // // }

        // fn get_eth_price_source(&mut self) -> Balance {
        //     //TODO get from oracle
        //     0
        // }
    }
}
