import { expect, fromSigner, setupContract } from './helpers';
import { consts } from './constants';

describe('Vault', () => {
  async function setupVault(emittedTokenAddress: string) {
    return setupContract('vault_contract', 'new', consts.EMPTY_ADDRESS, emittedTokenAddress, consts.EMPTY_ADDRESS);
  }

  async function setupEmmitedToken() {
    return setupContract('stable_coin_contract', 'new', 'stable_coin', 'description', '18');
  }

  it('creates a vault and mints an nft', async () => {
    const { contract: emittedTokenContract } = await setupEmmitedToken();
    const { query, defaultSigner: sender, contract: vaultContract } = await setupVault(emittedTokenContract.address.toString());

    // Assert - Sender is by default the owner of the contract

    await expect(vaultContract.tx.createVault()).to.eventually.be.fulfilled;
    await expect(query.totalSupply()).to.have.output(1);
    await expect(query.ownerOf({ u128: 0 })).to.have.output(sender.address);
  });

  it('not an owner creates a vault and gets an nft minted', async () => {
    const { contract: emittedTokenContract } = await setupEmmitedToken();
    const {
      query,
      accounts: [alice],
      contract: vaultContract,
    } = await setupVault(emittedTokenContract.address.toString());

    // Assert - Sender is by default the owner of the contract
    await expect(fromSigner(vaultContract, alice.address).tx.createVault()).to.eventually.be.fulfilled;
    await expect(query.ownerOf({ u128: 0 })).to.have.output(alice.address);
  });

  it('creates a vault and destroys it', async () => {
    const { contract: emittedTokenContract } = await setupEmmitedToken();
    const { contract: vaultContract } = await setupVault(emittedTokenContract.address.toString());

    // Assert - Sender is by default the owner of the contract
    await vaultContract.tx.createVault();
    const id = vaultContract.abi.registry.createType('u128', 0);
    await expect(vaultContract.tx.destroyVault(id)).to.eventually.be.fulfilled;
  });

  it('fails to destroy a vault if it does not exist', async () => {
    const { contract: emittedTokenContract } = await setupEmmitedToken();
    const { contract: vaultContract } = await setupVault(emittedTokenContract.address.toString());

    // Assert - Sender is by default the owner of the contract
    await vaultContract.tx.createVault();
    await expect(vaultContract.tx.destroyVault(1)).to.eventually.be.rejected; //TODO check actual reason or rejectedWith
  });

  it('fails to destroy a vault if the owner is not the caller', async () => {
    const { contract: emittedTokenContract } = await setupEmmitedToken();
    const {
      accounts: [alice],
      contract: vaultContract,
    } = await setupVault(emittedTokenContract.address.toString());

    // Assert - Sender is by default the owner of the contract
    await vaultContract.tx.createVault();
    await expect(fromSigner(vaultContract, alice.address).tx.destroyVault(0)).to.eventually.be.rejected; //TODO check actual reason or rejectedWith
  });
});
