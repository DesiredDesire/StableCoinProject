import { Signer } from 'redspot/types/provider';
import { setupOracle, setupEmmitedToken, setupCollateralMock, setupMeasurer, setupVault, setupVaultController } from './ourHelpers';
import { consts } from './constants';
import { fromSigner } from './helpers';

export async function setupSystem(owner: Signer) {
  const ownerAddress = owner.address;
  const { contract: oracleContract } = await setupOracle(ownerAddress);
  const { contract: emittedTokenContract } = await setupEmmitedToken(consts.STABLE_DECIMALS, ownerAddress);
  const { contract: collateralTokenContract } = await setupCollateralMock(consts.COLLATERAL_DECIMALS, ownerAddress);
  const { contract: measurerContract } = await setupMeasurer(oracleContract.address.toString(), ownerAddress);
  const {
    query,
    defaultSigner: sender,
    contract: vaultContract,
  } = await setupVault(
    oracleContract.address.toString(),
    collateralTokenContract.address.toString(),
    emittedTokenContract.address.toString(),
    0,
    2000000,
    0,
    ownerAddress
  );
  const { contract: vaultControllerContract } = await setupVaultController(
    measurerContract.address.toString(),
    vaultContract.address.toString(),
    ownerAddress
  );

  await fromSigner(vaultContract, owner.address).tx.setControllerAddress(vaultControllerContract.address);
  await fromSigner(emittedTokenContract, owner.address).tx.setupRole(consts.MINTER, vaultContract.address);
  await fromSigner(emittedTokenContract, owner.address).tx.setupRole(consts.BURNER, vaultContract.address);
  await fromSigner(emittedTokenContract, owner.address).tx.setupRole(consts.SETTER, owner.address);

  return { oracleContract, emittedTokenContract, collateralTokenContract, measurerContract, vaultContract, vaultControllerContract };
}
