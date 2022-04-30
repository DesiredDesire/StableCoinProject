#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod vault {
    use brush::contracts::psp22::*;
    use brush::{contracts::pausable::*, contracts::psp34::*};
    use ink_lang::codegen::Env;
    use ink_storage::traits::SpreadAllocate;
    use psp34::extensions::{burnable::*, metadata::*, mintable::*};
    use stable_coin_project::impls::emiting::*;
    use stable_coin_project::traits::vault::*;

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, PSP34Storage, PausableStorage, EmitingStorage)]
    pub struct VaultContract {
        #[PausableStorageField]
        pause: PausableData,
        #[EmitingStorageField]
        emit: EmitingData,
        #[PSP34StorageField]
        psp34: PSP34Data,

        pub collateral_by_id: Mapping<Id, Balance>,
        pub debt_by_id: Mapping<Id, Balance>,
        pub collaterall_token_address: AccountId,
        pub minimum_collateral_prcentage: u128,
        pub next_id: u128,
        pub minimum_collateral_percentage: u128,
    }

    impl PSP34 for VaultContract {}
    impl Emiting for VaultContract {}
    impl VaultInternal for VaultContract {}

    impl Vault for VaultContract {
        #[ink(message)]
        fn create_vault(&mut self) -> Result<Id, VaultError> {
            let caller = self.env().caller();
            self._mint_to(caller, Id::U128(self.next_id))?;
            self.next_id += 1;
            Ok(Id::U128(self.next_id))
            //TODO EMIT EVENT - PSP34::mint will emit event => TODO implement PSP34 event
        }
        #[ink(message)]
        fn destroy_vault(&mut self, vault_id: Id) -> Result<(), VaultError> {
            //let vault_data = self._get_caller_vault_by_id(&vault_id)?;
            let owner_account_id: AccountId = self.owner_of(vault_id).unwrap_or_default();
            if self.env().caller() != owner_account_id {
                return Err(VaultError::VaultOwnershipError);
            }

            if self.vault.debt_by_id.get(vault_id).unwrap_or(1) != 0 {
                return Err(VaultError::HasDebt);
            }
            if self.vault.collateral_by_id.get(vault_id).unwrap_or(1) != 0 {
                return Err(VaultError::NotEmpty);
            }
            self._burn_from(owner_account_id, vault_id);
            Ok(())
            //TODO EMIT EVENT - PSP34::burn will emit event => TODO implement PSP34 event
        }

        #[ink(message)]
        fn deposit_collateral(&mut self, vault_id: Id, amount: Balance) -> Result<(), VaultError> {
            let owner_account_id: AccountId = self.owner_of(vault_id).unwrap_or_default();
            if self.env().caller() != owner_account_id {
                return Err(VaultError::VaultOwnershipError);
            }
            // // PSP22Ref::transfer_from(
            // //     &(self.vault.collaterall_token_address),
            // //     caller,
            // //     self.env().account_id(),
            // //     amount,
            // //     Vec::<u8>::new(),
            // // )?;
            // let current_collateral = self._collateral_of(&vault_id).unwrap_or(0);
            // self.vault
            //     .collateral_by_id
            //     .insert(&vault_id, &(current_collateral + amount));
            Ok(())
        }

        #[ink(message)]
        fn withdraw_collateral(&mut self, vault_id: Id, amount: Balance) -> Result<(), VaultError> {
            let owner_account_id: AccountId = self.owner_of(vault_id).unwrap_or_default();
            if self.env().caller() != owner_account_id {
                return Err(VaultError::VaultOwnershipError);
            }
            // let contract = self.env().account_id();
            // let collateral_address = self.vault.collaterall_token_address;
            // let vault_collateral = self.vault.collateral_by_id.get(&vault_id).unwrap_or(0);
            // let vault_debt = self.vault.debt_by_id.get(&vault_id).unwrap_or(0); //TODO if we set 0 and the true debt is > 0 it is a BIG PROBLEM
            // let collateral_after = vault_collateral - amount;
            // match self._before_collateral_withdrawal(collateral_after, vault_debt) {
            //     Ok(v) => (),
            //     Err(e) => {
            //         return Err(VaultError::from(e));
            //     }
            // };
            // //TODO EMIT EVENT
            // match PSP22Ref::transfer_from(
            //     &(collateral_address),
            //     self.env().account_id(),
            //     self.env().caller(),
            //     amount,
            //     Vec::<u8>::new(),
            // ) {
            //     Ok(v) => (),
            //     Err(e) => {
            //         return Err(VaultError::from(e));
            //     }
            // };

            Ok(())
        }

        #[ink(message)]
        fn get_debt_ceiling(&mut self) -> Balance {
            //TODO dynamic or static debt ceiling?
            0
        }

        #[ink(message)]
        fn borrow_token(&mut self, vault_id: Id, amount: Balance) -> () {}
        #[ink(message)]
        fn pay_back_token(&mut self, vault_id: Id, amount: Balance) -> () {}
        #[ink(message)]
        fn buy_risky_vault(&mut self, vault_id: Id) -> () {}
    }

    impl VaultContract {
        #[ink(constructor)]
        pub fn new(
            collaterall_token_address: AccountId,
            minted_token_address: AccountId,
            minimum_collateral_prcentage: u128,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut VaultContract| {
                instance.vault.collaterall_token_address = collaterall_token_address;
                instance.minting.minted_token_address = minted_token_address;
                instance.vault.minimum_collateral_prcentage = minimum_collateral_prcentage;
            })
        }
        // fn _get_caller_vault_by_id(vault_id: Id) -> Result<SingleVaultData, VaultError> {
        //     let vault_data = self
        //         .vault
        //         .vault_data_by_id
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

        //     if collateralPercentage >= self.vault.minimum_collateral_prcentage {
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
