Traits
    -> Vault , VaultView, Vault Internal
    -> VControlling, VControllingView, VControllingInternal     (vault controlling)
    -> SControlling, SControllingView, SControllingInternal     (stable controlling)
    -> SPGenerating, SPGeneratingView, SPGeneratingInternal     (shares profit generating)
    -> SPControlling, SPControllingView, SPControllingInternal  (shares profit controlling)
    -> psp22Rated, PSP22RatedView, PSP22RatedInternal,          (rated and taxed for PSP22)
    -> Pausing                                                  
    -> Oracling
    -> Measuring, MeasuringView
    -> Managing
    -> Emitting, EmittingInternal
    -> Collateralling, CollaterallingInternal

Cntracts
    -> SharesTokenContract
        Describtion: 
            PSP22 with modified mint method and normal burn method that can be Accessed only by MINTER and BURNER roles. It can be paused by Owner to stop minting.
            MINTER role is granted to contracts that implement SPGenerating. BURNER role is granted to Treassury.
        Sorage:
            -> Psp22
            -> Psp22Metadata
            -> Ownable
            -> Pausable
            -> AccessControl
            -> self = total_minted_amount
        Ownable + AccessControl + Pausable + Pausing + Managing + PSP22 + Psp22Metadata + Psp22Mintable + Psp22Burnable

    -> StableCoinContract
        Describtion:
            PSP22 with usual mint and burn method that can be Accessed only by MINTER and BURNER. It can be paused by Owner to stop minting.
            Both MINTER and BURNER are granted to VaultContracts. MINTER is also granted to ShareProfitControllerContract.
            It implements Psp22Rated, which means that balances change with time depending on interest_rate_e12: i128 parameter.
            If interest_rate_e12 > 0 balances decrease with time. If interest_rate_e12 < 0 balances increse with time.
            is_unrated Mapping<AccountId, bool> keeps unformation about Accounts that balances are not changing.
            It is also taxed, which means that if tax_e6 : u128 parameter is > 0 there is tax on transfer.
            If the transfer is taxed depends if the reciever is_tax_free Mapping<AccountId,bool>.
            !!!THESE MECHANISM ARE TURNED ON ONLY TO KEEP PRICE PEGGED!!!
            Adjusting interest rate is intuitional.
            Taxing works only to keep price from rising to high, so is turned when pirce > peg.
            For example: lets say that Swap Pair (like Uniswap) is un_taxed. Then selling StableCoint is untaxed and buying it is taxed.
            However in tax clalculation the accound debt (from vaults) is taken into account. Thus if someone has a debt and has not enought stables to pay it back he is untaxed.
            This is in order to protect people who mint stables.
            The interest_rate_e12 and tax_e6 are controlled by stable_controller    
        Storage:
            -> Ownable
            -> Pausable
            -> AccessControl
            -> Psp22
            -> Psp22Metadata
            -> SPGenerating
            -> self =rated_psp22
        Ownable + Pausable + AccessControl + Managing + Pausing + SPGenerating + SPControllingInternal + SPControllingView +
        + Psp22Burnable + Psp22Mintable + Psp22Metadata + Psp22 + Psp22Rated + PSP22RatedView
        
    -> VaultContract
        Describtion:
            PSP34 tokens are proof of ownership of a given lock. In a lock one can deposit collateral and borrow (mint) stable_coin against it.
            Owner can pause borrowing.
            The Debt is rated. The interest_rate depend on market situation and can be poth positive and negative.
            In case of positive interest rate the lock generates income for protocol and the owner of lock is rewarded by Shares token (SPGenerating component).
            The minimum_collateral_ratio_e6 is maerket dependend. It gets lower if the price of stable coin is to high.
            It is in order to increase amount of minted tokens and lower amount of liquidated vaults. For example for vault with 200% collateral ratio the minimum ratio is 175%
        Storage: 
            -> Ownable
            -> Pausable
            -> Psp34
            -> Psp34Metadata
            -> Collateralling
            -> Emitting
            -> SPGenerating
            -> self = vault_storage
        Ownable + Pausable + Pausing + PSP34 + EmittingInternal + Emitting + CollaterallingInternal + Collateralling + SPGenerating + SPGeneratingInternal + SPGeneratingView
    
    -> MeasurerContract
        Describtion
            Based on oracle pride feeds it measures the peg of token with the stability_measure_parameter.
            If the price is higher than peg once per some period stability_measure_parameter += 1.
            If the price is lower than peg once per some perrod stability_measure_parameter -= 1.
            Depending on stability measure the parameters of vault and stable coin are set by vault contraoller and stable controller.
        Storage:
            -> Ownable
            -> Measuring
        Ownable + Measuring + MeasuringView
    
    -> ShareProfitController
        Describtion:
            It collects profit and debts from vault and stable coin.
            The profits can be minted to treassury and owner. splitted with use of treassury_part_e6.
            It controlls minting of share token  by SPGenerators with shareing_part_e6
        Storage
            -> Ownable
            -> SPControlling
        Ownable + SPControlling + SPControllingView

    -> StableControllerContract
        Describtion
            Based on stability_measure_parameter set current tax_e6 and interest_rate_e12 of stable coin
        Storage:
            -> Ownable
            -> SControlling
        Ownable + SControlling + SControllingView

    -> VaultControllerContract
        Describtion:
            Based on stability_measure_parameter set current interest_rate_e12 and current_collateral_cofficient_e6 in vault
        Sorage:
            -> Ownable
            -> VControlling
        Ownable + VControlling + VControllingView


