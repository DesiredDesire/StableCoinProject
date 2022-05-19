import { patract, network } from 'redspot';
import { AccountId } from '@polkadot/types/interfaces';
import { expect, fromSigner, setupContract } from '../scripts/helpers';
import { deploySystem } from '../scripts/ourDeployRated';
import { Signer } from 'redspot/types';
import Contract from '@redspot/patract/contract';
const { getSigners, api } = network;
import { consts } from '../scripts/constants';

const DECIMALS = 18;
describe('Deployment', () => {
  let owner: Signer;
  let oracleContract: Contract;
  let measurerContract: Contract;
  let sharesContract: Contract;
  let sharesProfitControllerContract: Contract;
  let stableCoinContract: Contract;
  let stableControllerContract: Contract;
  let collateralTokenContract: Contract;
  let vaultContract: Contract;
  let vaultControllerContract: Contract;

  beforeEach('setup system', async () => {
    const accounts = await getSigners();
    owner = accounts[0];
    const contracts = await deploySystem(owner);
    oracleContract = contracts.oracleContract;
    measurerContract = contracts.measurerContract;
    sharesContract = contracts.sharesContract;
    sharesProfitControllerContract = contracts.sharesProfitControllerContract;
    stableCoinContract = contracts.stableCoinContract;
    stableControllerContract = contracts.stableControllerContract;
    collateralTokenContract = contracts.collateralTokenContract;
    vaultContract = contracts.vaultContract;
    vaultControllerContract = contracts.vaultControllerContract;
  });

  describe('Tests', async () => {
    it('check owners', async () => {
      await expect(oracleContract.query.owner()).to.have.output(owner.address);
      await expect(measurerContract.query.owner()).to.have.output(owner.address);
      await expect(sharesContract.query.owner()).to.have.output(owner.address);
      await expect(sharesProfitControllerContract.query.owner()).to.have.output(owner.address);
      await expect(stableCoinContract.query.owner()).to.have.output(owner.address);
      await expect(stableControllerContract.query.owner()).to.have.output(owner.address);
      await expect(vaultContract.query.owner()).to.have.output(owner.address);
      await expect(vaultControllerContract.query.owner()).to.have.output(owner.address);
    });

    it('ckeck assignations', async () => {
      console.log('measurer');
      await expect(measurerContract.query.getOracleAddress()).to.have.output(oracleContract.address);

      console.log('share_profit_controller');
      await expect(sharesProfitControllerContract.query.getStableCoinAddress()).to.have.output(stableCoinContract.address);

      console.log('stable_coin');
      await expect(stableCoinContract.query.getSharesTokenAddress()).to.have.output(sharesContract.address);
      await expect(stableCoinContract.query.getSharesProfitControllerAddress()).to.have.output(sharesProfitControllerContract.address);

      console.log('stable_coin_controller');
      await expect(stableControllerContract.query.getStableCoinAddress()).to.have.output(stableCoinContract.address);
      await expect(stableControllerContract.query.getMeasurerAddress()).to.have.output(measurerContract.address);

      console.log('vault');
      await expect(vaultContract.query.getOracleAddress()).to.have.output(oracleContract.address);
      await expect(vaultContract.query.getSharesTokenAddress()).to.have.output(sharesContract.address);
      await expect(vaultContract.query.getSharesProfitControllerAddress()).to.have.output(sharesProfitControllerContract.address);
      await expect(vaultContract.query.getVaultControllerAddress()).to.have.output(vaultControllerContract.address);
      await expect(vaultContract.query.getCollateralTokenAddress()).to.have.output(collateralTokenContract.address);
      await expect(vaultContract.query.getEmitedTokenAddress()).to.have.output(stableCoinContract.address);

      console.log('vault_controller');
      await expect(vaultControllerContract.query.getVaultAddress()).to.have.output(vaultContract.address);
      await expect(vaultControllerContract.query.getMeasurerAddress()).to.have.output(measurerContract.address);
    });

    it('check role assignations', async () => {
      console.log('share_token');
      await expect(sharesContract.query.hasRole(consts.MINTER, vaultContract.address)).to.have.output(true);
      await expect(sharesContract.query.hasRole(consts.MINTER, stableCoinContract.address)).to.have.output(true);
      console.log('shares_profit_controller');
      await expect(sharesProfitControllerContract.query.isGenerator(vaultContract.address.toString())).to.have.output(true);
      await expect(sharesProfitControllerContract.query.isGenerator(stableCoinContract.address.toString())).to.have.output(true);
      console.log('stable_coint');
      await expect(stableCoinContract.query.hasRole(consts.MINTER, vaultContract.address)).to.have.output(true);
      await expect(stableCoinContract.query.hasRole(consts.BURNER, vaultContract.address)).to.have.output(true);
      await expect(stableCoinContract.query.hasRole(consts.VAULT, vaultContract.address)).to.have.output(true);
      await expect(stableCoinContract.query.hasRole(consts.MINTER, sharesProfitControllerContract.address)).to.have.output(true);
    });
  });
});
