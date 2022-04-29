#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod vault {
    use brush::contract::psp22::*;
    use brush::{contracts::pausable::*, contracts::psp34::*};
    use ink_lang::codegen::Env;
    use ink_lang::ToAccountId;
    use ink_prelude::string::String;
    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;
    use psp34::extensions::{burnable::*, metadata::*, mintable::*};
    use stable_coin_project::impls::minter::*;
    use stable_coin_project::impls::vault::*;
    use stable_coin_project::traits::vault::*;

    #[ink(storage)]
    #[derive(
        Default, SpreadAllocate, PSP34Storage, VaultStorage, PausableStorage, MinterStorage,
    )]
    pub struct VaultContract {
        #[PausableStorageField]
        pause: PausableData,
        #[MinterStorageField]
        minting: MinterData,
        #[PSP34StorageField]
        psp34: PSP34Data,
        #[VaultStorageField]
        vault: VaultData,
        next_id: u128,
        _minimumCollateralPercentage: u128,
    }

    impl PSP34 for VaultContract {}
    impl Minter for VaultContract {}
    impl VaultInternal for VaultContract {}

    impl Vault for VaultContract {
        #[ink(message)]
        fn create_vault(&mut self) -> Result<Id, VaultError> {
            let caller = self.env().caller();
            self._mint_to(caller, Id::U128(self.next_id))?;

            // self.vault.vault_data_by_id.insert(
            //     &caller,
            //     SingleVaultData {
            //         collateral: 0,
            //         debt: 0,
            //     },
            // );
            self.next_id += 1;
            Ok(Id::U128(self.next_id))
            //TODO EMIT EVENT - PSP34::mint will emit event => TODO implement PSP34 event
        }
        #[ink(message)]
        fn destroy_vault(&mut self, vault_id: Id) -> Result<(), VaultError> {
            //let vault_data = self._get_caller_vault_by_id(&vault_id)?;
            if (self.env().caller() != self.owner_of(vault_id).unwrap_or_default()) {
                return Err(VaultError::VaultOwnershipError);
            }

            if (self.vault.debt_by_id.get(vault_id).unwrap_or(1) != 0) {
                return Err(VaultError::HasDebt);
            }
            if (self.vault.collateral_by_id.get(vault_id).unwrap_or(1) != 0) {
                return Err(VaultError::NotEmpty);
            }
            self._burn_from(&owner_account_id, vault_id);
            Ok(())
            //TODO EMIT EVENT - PSP34::burn will emit event => TODO implement PSP34 event
        }

        #[ink(message)]
        fn deposit_collateral(&mut self, vault_id: Id, amount: Balance) -> Result<(), VaultError> {
            let caller = self.env().caller();
            if (self.env().caller() != self.owner_of(vault_id).unwrap_or_default()) {
                return Err(VaultOwnershipError);
            }
            result = match PSP22Ref::transfer_from(
                &(self.vault.collaterall_token_address),
                caller,
                self.env().account_id(),
                amount,
            ) {
                Ok(r) => Ok(r),
                Err(e) => Err(VaultError::from(e)),
            };
            if (result == Ok(())) {
                current_collateral = self._collateral_of(&vault_id);
                self.vault
                    .collateral_by_id
                    .insert(&vault_id, &(current_collateral + amount));
            }
            Ok(())
        }

        #[ink(message)]
        fn withdraw_collateral(&mut self, vault_id: Id, amount: Balance) -> Result<(), VaultError> {
            // let vault_data = self._get_caller_vault_by_id(&vault_id)?;
            // if vault_data.collateral > amount {
            //     return Err(VaultError::WithdrawError::InsufficientCollateralError);
            // }
            let vault_collateral = self.vault.collateral_by_id.get(&vault_id).unwrap_or(0);
            let vault_debt = self.vault.debt_by_id.get(&vault_id).unwrap_or(0); //TODO if we set 0 and the true debt is > 0 it is a BIG PROBLEM
            let collateral_after = vault_collateral - amount;
            result = match self._before_collateral_withdrawal(collateral_after, vault_debt) {
                Ok(..) => Ok(()),
                Err(e) => Err(Vault22Error::from(e)),
            };
            //TODO EMIT EVENT
            if (result == Ok(())) {
                return match PSP22Ref::transfer_from(
                    &(self.vault.collaterall_token_address),
                    self.env().account_id(),
                    caller,
                    amount,
                ) {
                    Ok(..) => Ok(()),
                    Err(e) => Err(Vault22Error::from(e)),
                };
            }
            Ok(())
        }

        #[ink(message)]
        fn get_debt_ceiling(&mut self) -> Balance {
            //TODO dynamic or static debt ceiling?
            0
        }
    }

    impl VaultContract {
        #[ink(constructor)]
        pub fn new(
            collaterall_token_address: AccountId,
            minted_token_address: AccountId,
            minimumCollateralPercentage: u128,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut VaultContract| {
                instance.vault.collaterall_token_address = collaterall_token_address;
                instance.minting.minted_token_address = minted_token_address;
                instance.vault.minimumCollateralPercentage = minimumCollateralPercentage;
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
        // ) -> Result<(), MinterError> {
        //     if collateral != 0 {
        //         self::Minter::mint(&to, collateral)?;
        //     }
        //     Ok(())
        // }

        fn _before_collateral_withdrawal(
            &mut self,
            collateral: Balance,
            debt: Balance,
        ) -> Result<(), CollateralError> {
            let collateral_value_e6 = self._calculate_collateral_value_e6(collateral).unwrap_or(0); //TODO propagate error

            let collateralPercentage = collateral_value_e6 / debt;

            if collateralPercentage >= self.vault.minimumCollateralPercentage {
                return Err(CollateralError::CollateralBelowMinimumPercentageError);
            }
            Ok(())
        }

        fn _calculate_collateral_value_e6(
            &mut self,
            collateral: Balance,
        ) -> Result<Balance, CollateralError> {
            let collateral_price_e6 = self.get_eth_price_source(); // assuming that price is multiplied by 10^6. If price is 1.01 then we get 1010000
            if collateral_price_e6 == 0 {
                return Err(CollateralError::PriceEqualsZeroError);
            }
            Ok(collateral * collateral_price_e6)
        }

        // fn get_token_price_source(&mut self) -> Balance {
        //     self.token_peg
        // }

        fn get_eth_price_source(&mut self) -> Balance {
            //TODO get from oracle
            0
        }
    }
}
