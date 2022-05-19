import { patract, network } from 'redspot';
import { expect, fromSigner, setupContract } from '../scripts/helpers';
import { consts } from '../scripts/constants';
import { Signer } from 'redspot/types';
import Contract from '@redspot/patract/contract';
import { deploySystem } from '../scripts/ourDeployRated';
const { getSigners, api } = network;

const E6: bigint = 1000000n;
const STA_DEC: bigint = E6;
const COL_DEC: bigint = E6 * E6;
describe('Vault', () => {
  let users: Signer[];
  let owner: Signer;
  let oracleContract: Contract;
  let stableCoinContract: Contract;
  let collateralTokenContract: Contract;
  let measurerContract: Contract;
  let vaultContract: Contract;
  let vaultControllerContract: Contract;

  beforeEach('setup system', async () => {
    users = await getSigners();
    owner = users.shift() as Signer;
    const contracts = await deploySystem(owner);
    oracleContract = contracts.oracleContract;
    stableCoinContract = contracts.stableCoinContract;
    collateralTokenContract = contracts.collateralTokenContract;
    measurerContract = contracts.measurerContract;
    vaultContract = contracts.vaultContract;
    vaultControllerContract = contracts.vaultControllerContract;
  });

  describe.only('vaults creation and destruction', async () => {
    it('owner creates a vault and mints an nft', async () => {
      await expect(fromSigner(vaultContract, owner.address).tx.createVault()).to.eventually.be.fulfilled;
      await expect(vaultContract.query.totalSupply()).to.have.output(1);
      await expect(vaultContract.query.ownerOf({ u128: 0 })).to.have.output(owner.address);
    });

    it('not an owner creates a vault and gets an nft minted', async () => {
      await expect(fromSigner(vaultContract, users[0].address).tx.createVault()).to.eventually.be.fulfilled;
      await expect(vaultContract.query.totalSupply()).to.have.output(1);
      await expect(vaultContract.query.ownerOf({ u128: 0 })).to.have.output(users[0].address);
    });

    it('creates a vault and destroys it', async () => {
      await fromSigner(vaultContract, users[0].address).tx.createVault();
      const id = vaultContract.abi.registry.createType('u128', 0);
      await expect(fromSigner(vaultContract, users[0].address).tx.destroyVault(id)).to.eventually.be.fulfilled;
    });

    it('fails to destroy a vault if it does not exist', async () => {
      await fromSigner(vaultContract, users[0].address).tx.createVault();
      await expect(fromSigner(vaultContract, users[0].address).tx.destroyVault(1)).to.eventually.be.rejected; //TODO check actual reason or rejectedWith
    });

    it('fails to destroy a vault if the owner is not the caller', async () => {
      await fromSigner(vaultContract, users[0].address).tx.createVault();
      await expect(fromSigner(vaultContract, users[1].address).tx.destroyVault(0)).to.eventually.be.rejected; //TODO check actual reason or rejectedWith
    });
  });

  describe('collateral actions', async () => {
    const MINTED_AMOUNT: bigint = BigInt('4313514311412321412');
    const AZERO_USD_PRICE: bigint = BigInt('1200000');
    beforeEach('create vault', async () => {
      await fromSigner(vaultContract, users[0].address).tx.createVault();
      await fromSigner(collateralTokenContract, users[0].address).tx.mintAnyCaller(users[0].address, MINTED_AMOUNT);
      await fromSigner(collateralTokenContract, users[0].address).tx.approve(vaultContract.address, MINTED_AMOUNT);
      await fromSigner(oracleContract, owner.address).tx.feedAzeroUsdPriceE6(AZERO_USD_PRICE);
    });

    it('deposit works', async () => {
      const depositAmount = MINTED_AMOUNT;
      await fromSigner(vaultContract, users[0].address).tx.depositCollateral(0, depositAmount);
      await expect(collateralTokenContract.query.balanceOf(vaultContract.address)).to.have.output(depositAmount);
      const res = await vaultContract.query.getVaultDetails(0);
      await expect(vaultContract.query.getVaultDetails(0)).to.have.output([depositAmount, 0]);
    });
    it('deposit fails if not enough balace', async () => {
      const depositAmount = MINTED_AMOUNT + 1n;
      await expect(fromSigner(vaultContract, users[0].address).tx.depositCollateral(0, depositAmount)).to.eventually.be.rejected;
    });

    it('non_empty vault can not be destoryed', async () => {
      const depositAmount = MINTED_AMOUNT;
      await fromSigner(vaultContract, users[0].address).tx.depositCollateral(0, depositAmount);
      await expect(fromSigner(vaultContract, users[0].address).tx.destroyVault(0)).to.eventually.be.rejected; //TODO check actual reason or rejectedWith
    });

    it('withdraw of collateral works', async () => {
      const depositAmount = MINTED_AMOUNT;
      const withdrawAmount = MINTED_AMOUNT / 2n;
      const difference = depositAmount - withdrawAmount;
      await fromSigner(vaultContract, users[0].address).tx.depositCollateral(0, depositAmount);
      await fromSigner(vaultContract, users[0].address).tx.withdrawCollateral(0, withdrawAmount);
      await expect(vaultContract.query.getVaultDetails(0)).to.have.output([difference, 0]);
    });

    it('after withdrawing all, vault can be destroyed test', async () => {
      const depositAmount = MINTED_AMOUNT;
      const withdrawAmount = MINTED_AMOUNT;
      await fromSigner(vaultContract, users[0].address).tx.depositCollateral(0, depositAmount);
      await fromSigner(vaultContract, users[0].address).tx.withdrawCollateral(0, withdrawAmount);
      await expect(vaultContract.query.getVaultDetails(0)).to.have.output([0, 0]);
      await expect(fromSigner(vaultContract, users[0].address).tx.destroyVault(0)).to.eventually.be.fulfilled;
    });
  });

  describe('oracle should work for vault', async () => {
    const AZERO_USD_PRICE: bigint = BigInt('1200000');
    beforeEach('set price', async () => {
      await fromSigner(oracleContract, owner.address).tx.feedAzeroUsdPriceE6(AZERO_USD_PRICE);
    });
    it('price should have been set', async () => {
      await expect(oracleContract.query.getAzeroUsdPriceE6()).to.have.output(AZERO_USD_PRICE);
    });
  });

  describe('Emiting actions', async () => {
    const MINTED_AMOUNT: bigint = BigInt('4313514311412321412');
    const AZERO_USD_PRICE: bigint = BigInt('1200000');
    const DEPOSITED_AMOUNT: bigint = BigInt('1000000000000');
    beforeEach('create vault and make deposit', async () => {
      await fromSigner(vaultContract, users[0].address).tx.createVault();
      await fromSigner(collateralTokenContract, users[0].address).tx.mintAnyCaller(users[0].address, MINTED_AMOUNT);
      await fromSigner(collateralTokenContract, users[0].address).tx.approve(vaultContract.address, MINTED_AMOUNT);
      await fromSigner(oracleContract, owner.address).tx.feedAzeroUsdPriceE6(AZERO_USD_PRICE);
      await fromSigner(vaultContract, users[0].address).tx.depositCollateral(0, DEPOSITED_AMOUNT);
    });

    it('get debt ceiling returns correct value', async () => {
      console.log('start');
      const debtCeiling = (((DEPOSITED_AMOUNT * AZERO_USD_PRICE) / COL_DEC) * STA_DEC) / 2000000n;
      await expect(vaultContract.query.getDebtCeiling(0)).to.have.output(debtCeiling);
    });

    it('borrow should work for debt ceiling', async () => {
      console.log('start');

      const debtCeiling = await BigInt((await vaultContract.query.getDebtCeiling(0)).output?.toString() as string);
      await expect(fromSigner(vaultContract, users[0].address).tx.borrowToken(0, debtCeiling)).to.eventually.be.fulfilled;
      await expect(stableCoinContract.query.balanceOf(users[0].address)).to.have.output(debtCeiling);
      await expect(vaultContract.query.getVaultDetails(0)).to.have.output([DEPOSITED_AMOUNT, debtCeiling]);
    });

    it('borrow should work for debt ceiling - 1', async () => {
      console.log('start');

      const debtCeiling = await BigInt((await vaultContract.query.getDebtCeiling(0)).output?.toString() as string);
      await expect(fromSigner(vaultContract, users[0].address).tx.borrowToken(0, debtCeiling - 1n)).to.eventually.be.fulfilled;
      await expect(stableCoinContract.query.balanceOf(users[0].address)).to.have.output(debtCeiling - 1n);
      await expect(vaultContract.query.getVaultDetails(0)).to.have.output([DEPOSITED_AMOUNT, debtCeiling - 1n]);
    });
  });
});
