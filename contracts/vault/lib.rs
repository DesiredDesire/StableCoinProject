#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod vault {
    use brush::{contracts::pausable::*, contracts::psp34::*};
    use ink_lang::codegen::Env;
    use ink_lang::ToAccountId;
    use ink_prelude::string::String;
    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;
    use psp34::extensions::{burnable::*, metadata::*, mintable::*};
    use stable_coin_project::impls::minter::*;
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
    impl PSP34Mintable for VaultContract {}
    impl PSP34Burnable for VaultContract {}
    impl Minter for VaultContract {}

    impl Vault for VaultContract {
        #[ink(message)]
        fn create_vault(&mut self) -> Result<Id, VaultError> {
            let caller_id = self.env().caller();
            self::PSP34Mintable::mint(&caller_id, Id::U128(self.next_id))?;

            self.vault.vault_data_by_id.insert(
                &caller_id,
                SingleVaultData {
                    collateral: 0,
                    debt: 0,
                },
            );
            self.next_id += 1;
            Ok(Id::U128(self.next_id))
            //TODO EMIT EVENT
        }
        #[ink(message)]
        fn destroy_vault(&mut self, vault_id: Id) -> Result<(), VaultError> {
            let vault_data = self._get_caller_vault_by_id(&vault_id)?;
            self::PSP34Burnable::burn(&owner_account_id, &vault_id);
            self._mint_collateral_if_not_zero(&vault_data.collateral, &owner_account_id)?;
            Ok(())
            //TODO EMIT EVENT
        }

        #[ink(message)]
        fn withdraw_collateral(&mut self, vault_id: Id, amount: Balance) -> Result<(), VaultError> {
            let vault_data = self._get_caller_vault_by_id(&vault_id)?;
            if vault_data.collateral > amount {
                return Err(WithdrawError::InsufficientCollateralError);
            }
            let collateral_after = vault_data.collateral - amount;
            self._before_collateral_withdrawal(collateral_after, vault_data.debt)?;
            self._mint_collateral_if_not_zero(&vault_data.collateral, &owner_account_id)?;
            //TODO EMIT EVENT
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
                instance.minimumCollateralPercentage = minimumCollateralPercentage;
            })
        }
        fn _get_caller_vault_by_id(vault_id: Id) -> Result<SingleVaultData, VaultError> {
            let vault_data = self
                .vault
                .vault_data_by_id
                .get(&vault_id)
                .ok_or(VaultError::NonExistingVaultError)?;
            let owner_account_id = self.owner_of(&vault_id);
            if owner_account_id.unwrap_or_default() != self.env().caller() {
                return Err(VaultError::VaultOwnershipError);
            }
        }

        fn _mint_collateral_if_not_zero(
            collateral: u128,
            to: AccountId,
        ) -> Result<(), MinterError> {
            if collateral != 0 {
                self::Minter::mint(&to, collateral)?;
            }
            Ok(());
        }

        fn _before_collateral_withdrawal(
            &mut self,
            collateral: u128,
            debt: u128,
        ) -> Result<(), CollateralError> {
            let (collateral_value_times_100, debt_value) =
                self._calculate_collateral_properties(collateral, debt);

            let collateralPercentage = collateral_value_times_100 / debt_value;

            if collateralPercentage >= _minimumCollateralPercentage {
                return Err(CollateralError::CollateralBelowMinimumPercentageError);
            }
            Ok(())
        }

        fn _calculate_collateral_properties(
            &mut self,
            collateral: Balance,
            debt: Balance,
        ) -> Result<(Balance, Balance), CollateralError> {
            let eth_price_source = self.get_eth_price_source();
            let token_price_source = self.get_token_price_source();
            if eth_price_source == 0 || token_price_source == 0 {
                return Err(CollateralError::PriceEqualsZeroError);
            }
            let collateral_value = collateral * eth_price_source;
            let debt_value = debt * token_price_source;

            let collateral_value_times_100 = collateral_value * 100;

            return (collateral_value_times_100, debt_value);
        }

        fn get_token_price_source(&mut self) -> Balance {
            self.token_peg
        }

        fn get_eth_price_source(&mut self) -> Balance {
            //TODO get from oracle
            0
        }
    }
}
