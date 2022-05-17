import { patract, network } from 'redspot';
import { AccountId } from '@polkadot/types/interfaces';
import { expect, fromSigner, setupContract } from '../scripts/helpers';
import { deploySystem } from '../scripts/ourDeploy';
import { Signer } from 'redspot/types';
import Contract from '@redspot/patract/contract';
const { getSigners, api } = network;
import { consts } from '../scripts/constants';

const DECIMALS = 18;
describe('Deployment', () => {
  let owner: Signer;
  let oracleContract: Contract;
  let stableTokenContract: Contract;
  let collateralTokenContract: Contract;
  let measurerContract: Contract;
  let vaultContract: Contract;
  let vaultControllerContract: Contract;

  beforeEach('setup system', async () => {
    const accounts = await getSigners();
    owner = accounts[0];
    const contracts = await deploySystem(owner);
    oracleContract = contracts.oracleContract;
    stableTokenContract = contracts.stableTokenContract;
    collateralTokenContract = contracts.collateralTokenContract;
    measurerContract = contracts.measurerContract;
    vaultContract = contracts.vaultContract;
    vaultControllerContract = contracts.vaultControllerContract;
  });

  describe('Tests', async () => {
    it('check owners', async () => {
      await expect(oracleContract.query.owner()).to.have.output(owner.address);
      await expect(measurerContract.query.owner()).to.have.output(owner.address);
      await expect(vaultContract.query.owner()).to.have.output(owner.address);
    });

    it('ckeck assignations', async () => {
      await expect(measurerContract.query.getOracleAddress()).to.have.output(oracleContract.address);
      await expect(vaultControllerContract.query.getVaultAddress()).to.have.output(vaultContract.address);
      await expect(vaultContract.query.getControllerAddress()).to.have.output(vaultControllerContract.address);
      await expect(vaultContract.query.getOracleAddress()).to.have.output(oracleContract.address);
      await expect(vaultContract.query.getCollateralTokenAddress()).to.have.output(collateralTokenContract.address);
    });

    it('check vault role assignations', async () => {
      await expect(stableTokenContract.query.hasRole(consts.MINTER, vaultContract.address)).to.have.output(true);
      await expect(stableTokenContract.query.hasRole(consts.BURNER, vaultContract.address)).to.have.output(true);
      await expect(stableTokenContract.query.hasRole(consts.SETTER, owner.address)).to.have.output(true);
    });
  });
});
