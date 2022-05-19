import { Signer } from 'redspot/types/provider';
import {
  deployOracle,
  deployEmmitedToken,
  deployCollateralMock,
  deployMeasurer,
  deployVault,
  deployShareToken,
  deployVaultController,
} from './ourHelpers';
import { consts } from './constants';
import { fromSigner } from './helpers';

export async function deploySystem(owner: Signer) {
  const ownerAddress = owner.address;
  const { contract: oracleContract } = await deployOracle(ownerAddress);
  const { contract: stableTokenContract } = await deployEmmitedToken(consts.STABLE_DECIMALS, ownerAddress);
  const { contract: collateralTokenContract } = await deployCollateralMock(consts.COLLATERAL_DECIMALS, ownerAddress);
  const { contract: measurerContract } = await deployMeasurer(oracleContract.address.toString(), ownerAddress);
  const { contract: shareContract } = await deployShareToken(undefined, undefined, undefined, ownerAddress);
  const { contract: vaultContract } = await deployVault(
    shareContract.address.toString(),
    collateralTokenContract.address.toString(),
    stableTokenContract.address.toString(),
    2000000,
    10000,
    0,
    ownerAddress
  );
  const { contract: vaultControllerContract } = await deployVaultController(
    measurerContract.address.toString(),
    vaultContract.address.toString(),
    ownerAddress
  );

  await fromSigner(vaultContract, owner.address).tx.setControllerAddress(vaultControllerContract.address);
  await fromSigner(vaultContract, owner.address).tx.setOracleAddress(oracleContract.address);
  await fromSigner(stableTokenContract, owner.address).tx.setupRole(consts.MINTER, vaultContract.address);
  await fromSigner(stableTokenContract, owner.address).tx.setupRole(consts.BURNER, vaultContract.address);
  await fromSigner(stableTokenContract, owner.address).tx.setupRole(consts.SETTER, owner.address);
  await fromSigner(stableTokenContract, owner.address).tx.setupRole(consts.SETTER, owner.address);

  return { oracleContract, stableTokenContract, collateralTokenContract, measurerContract, vaultContract, vaultControllerContract };
}
