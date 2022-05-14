import { AccountId } from '@polkadot/types/interfaces';
import { expect, fromSigner, setupContract } from './helpers'; 
import {consts} from './constants'

  export async function setupOracle(owner: string) {
    return setupContract('oracle_contract', 'new',owner);
  }
  export async function setupMeasurer(oracleAddress: string,owner: string) {
    return setupContract('measurer_contract', 'new', oracleAddress,owner);
  }
  export async function setupCollateralMock(decimals: number = consts.COLLATERAL_DECIMALS, owner: string) {
    return setupContract('psp22_emitable_contract', 'new', 'emitable_coin', 'sample_description', decimals ,owner);
  }
  export async function setupController(measurerAddress: string, vaultAddress: string,owner: string) {
    return setupContract('vault_controller_contract', 'new', measurerAddress, vaultAddress,owner);
  }
  export async function setupEmmitedToken(decimals: number = consts.STABLE_DECIMALS, owner: string) {
    return setupContract('stable_coin_contract', 'new', 'stable_coin', 'description', decimals ,owner);
  }

  export async function setupVaultController(
    measurer_address: string,
    vault_address: string,
    owner: string
  ) {
    return setupContract(
      'vault_controller_contract',
      'new',
      measurer_address,
      vault_address,
      owner
    );
  }
  export async function setupVault(
    oracleAddress: string,
    collateralTokenAddress: string,
    emittedTokenAddress: string,
    interestRateStepValue: number = 0,
    maximumCollateral: number = 2000000,
    interestStep: number = 0,
    owner: string,
  ) {
    return setupContract(
      'vault_contract',
      'new',
      oracleAddress,
      collateralTokenAddress,
      emittedTokenAddress,
      interestRateStepValue,
      maximumCollateral,
      interestStep,owner
    );
  }