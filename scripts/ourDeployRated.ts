import { Signer } from 'redspot/types/provider';
import {
  deployOracle,
  deployCollateralMock,
  deployMeasurer,
  deployShareToken,
  setupStableCoinContract,
  deployShareProfitController,
  setupVaultContract,
  setupSharesProfitControllerContract,
} from './ourHelpers';
import { consts } from './constants';
import { fromSigner } from './helpers';

const INIT_AZERO_USD_PRICE_E6 = 1200000;
export async function deploySystem(owner: Signer) {
  console.log(`delpoying with: ${owner.address}`);
  const { contract: oracleContract } = await deployOracle(owner.address);
  await fromSigner(oracleContract, owner.address).tx.feedAzeroUsdPriceE6(INIT_AZERO_USD_PRICE_E6);
  const { contract: measurerContract } = await deployMeasurer(oracleContract.address.toString(), owner.address);
  const { contract: sharesContract } = await deployShareToken(undefined, undefined, undefined, owner.address);
  const stableSetupResults = await setupStableCoinContract(
    undefined,
    undefined,
    undefined,
    measurerContract,
    sharesContract,
    owner.address
  );
  const { contract: stableCoinContract } = stableSetupResults.stableCoin;
  const { contract: stableControllerContract } = stableSetupResults.stableController;

  const { contract: sharesProfitControllerContract } = await setupSharesProfitControllerContract(stableCoinContract, owner.address);

  const { contract: collateralTokenContract } = await deployCollateralMock(consts.COLLATERAL_DECIMALS, owner.address);

  const vaultSetupResults = await setupVaultContract(
    oracleContract,
    measurerContract,
    sharesProfitControllerContract,
    sharesContract,
    collateralTokenContract,
    stableCoinContract,
    2000000,
    10000,
    0,
    owner.address
  );

  const { contract: vaultContract } = vaultSetupResults.vault;
  const { contract: vaultControllerContract } = vaultSetupResults.vaultController;

  return {
    oracleContract,
    measurerContract,
    sharesContract,
    sharesProfitControllerContract,
    stableCoinContract,
    stableControllerContract,
    collateralTokenContract,
    vaultContract,
    vaultControllerContract,
  };
}
