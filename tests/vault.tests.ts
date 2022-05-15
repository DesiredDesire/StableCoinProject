import { patract, network } from 'redspot';
import { expect, fromSigner, setupContract } from '../scripts/helpers';
import { consts } from '../scripts/constants';
import { Signer } from 'redspot/types';
import Contract from '@redspot/patract/contract';
import { setupSystem } from '../scripts/ourDeploy';
const { getSigners, api } = network;
describe.only('Vault', () => {
  let users: Signer[];
  let owner: Signer;
  let oracleContract: Contract;
  let emittedTokenContract: Contract;
  let collateralTokenContract: Contract;
  let measurerContract: Contract;
  let vaultContract: Contract;
  let vaultControllerContract: Contract;

  beforeEach('setup system', async () => {
    users = await getSigners();
    owner = users.shift() as Signer;
    const contracts = await setupSystem(owner);
    oracleContract = contracts.oracleContract;
    emittedTokenContract = contracts.emittedTokenContract;
    collateralTokenContract = contracts.collateralTokenContract;
    measurerContract = contracts.measurerContract;
    vaultContract = contracts.vaultContract;
    vaultControllerContract = contracts.vaultControllerContract;
  });

  describe('vaults creation and destruction', async () => {
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

  describe.only('collateral actions', async () => {
    const MINTED_AMOUNT: bigint = BigInt('4313514311412321412');
    beforeEach('create vault', async () => {
      await fromSigner(vaultContract, users[0].address).tx.createVault();
      await fromSigner(collateralTokenContract, users[0].address).tx.mintAnyCaller(users[0].address, MINTED_AMOUNT);
      await fromSigner(collateralTokenContract, users[0].address).tx.approve(vaultContract.address, MINTED_AMOUNT);
    });

    it('deposit works', async () => {
      const depositAmount = MINTED_AMOUNT;
      await fromSigner(vaultContract, users[0].address).tx.depositCollateral(0, depositAmount);
      await expect(collateralTokenContract.query.balanceOf(vaultContract.address)).to.have.output(depositAmount);
      const res = await vaultContract.query.getVaultDetails(0);
      console.log(res);
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

    it('after withdrawing all, vault can be destroyed test1', async () => {
      const depositAmount = MINTED_AMOUNT;
      const withdrawAmount = MINTED_AMOUNT;
      await fromSigner(vaultContract, users[0].address).tx.depositCollateral(0, depositAmount);
      await fromSigner(vaultContract, users[0].address).tx.withdrawCollateral(0, withdrawAmount);
      await expect(vaultContract.query.getVaultDetails(0)).to.have.output([0, 0]);
      await expect(fromSigner(vaultContract, users[0].address).tx.destroyVault(0)).to.eventually.be.fulfilled;
    });
    it('after withdrawing all, vault can be destroyed test2', async () => {
      const depositAmount = MINTED_AMOUNT;
      const withdrawAmount = MINTED_AMOUNT + 1n;
      await fromSigner(vaultContract, users[0].address).tx.depositCollateral(0, depositAmount);
      await fromSigner(vaultContract, users[0].address).tx.withdrawCollateral(0, withdrawAmount);
      await expect(vaultContract.query.getVaultDetails(0)).to.have.output([0, 0]);
      await expect(fromSigner(vaultContract, users[0].address).tx.destroyVault(0)).to.eventually.be.fulfilled;
    });
  });
});
