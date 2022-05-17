import { AccountId } from '@polkadot/types/interfaces';
import { expect, fromSigner, setupContract } from './helpers';
import { consts } from './constants';

export async function deployMeasurer(oracleAddress: string, owner: string) {
  return setupContract('measurer_contract', 'new', oracleAddress, owner);
}

export async function deployOracle(owner: string) {
  return setupContract('oracle_contract', 'new', owner);
}

export async function deployEmmitedToken(decimals: number = consts.STABLE_DECIMALS, owner: string) {
  return setupContract('psp22_emitable_contract', 'new', 'stable_coin', 'description', decimals, owner);
}

export async function deploySPController(shares_address: AccountId, owner: string) {
  return setupContract('shares_profit_controller_contract', 'new', shares_address, owner);
}

export async function deployShareToken(
  name: string = 'Tutum share',
  symbol: string = 'TUM',
  decimals: number = consts.SHARES_DECIMALS,
  owner: string
) {
  return setupContract('shares_token_contract', 'new', name, symbol, decimals, owner);
}

export async function deployStableCoinToken(
  name: string = 'USD Alpeh',
  symbol: string = 'USDA',
  decimals: number = consts.STABLE_DECIMALS,
  share_token_address: string,
  owner: string
) {
  return setupContract('psp22_emitable_contract', 'new', name, symbol, decimals, share_token_address, owner);
}

export async function deploySController(measurerAddress: string, stableAddress: string, owner: string) {
  return setupContract('vault_controller_contract', 'new', measurerAddress, stableAddress, owner);
}

export async function deployCollateralMock(decimals: number = consts.COLLATERAL_DECIMALS, owner: string) {
  return setupContract('psp22_emitable_contract', 'new', 'emitable_coin', 'sample_description', decimals, owner);
}

export async function deployVault(
  sharesTokenAddress: string,
  collateralTokenAddress: string,
  stableTokenAddress: string,
  maximumMinimumCollateralCoefficientE6: number = 2000000,
  collateralStepValueE6: number = 10000,
  interestRateStepValue: number = 0,
  owner: string
) {
  return setupContract(
    'vault_contract',
    'new',
    sharesTokenAddress,
    collateralTokenAddress,
    stableTokenAddress,
    maximumMinimumCollateralCoefficientE6,
    collateralStepValueE6,
    interestRateStepValue,
    owner
  );
}

export async function deployVaultController(measurer_address: string, vault_address: string, owner: string) {
  return setupContract('vault_controller_contract', 'new', measurer_address, vault_address, owner);
}
