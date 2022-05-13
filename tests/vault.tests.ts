import { expect, fromSigner, setupContract } from './helpers';
// import { consts } from './constants';
const DECIMALS = 18;
describe('Vault', () => {
  async function setupOracle() {
    return setupContract('oracle_contract', 'new');
  }
  async function setupMeasurer(oracleAddress: string) {
    return setupContract('measurer_contract', 'new', oracleAddress);
  }
  async function setupCollateralMock() {
    return setupContract('psp22_emitable_contract', 'new', 'emitable_coin', 'sample_description', DECIMALS);
  }
  async function setupController(measurerAddress: string, vaultAddress: string) {
    return setupContract('vault_controller_contract', 'new', measurerAddress, vaultAddress);
  }
  async function setupEmmitedToken() {
    return setupContract('stable_coin_contract', 'new', 'stable_coin', 'description', DECIMALS);
  }

  async function setupVault(
    oracleAddress: string,
    collateralTokenAddress: string,
    emittedTokenAddress: string,
    interestRateStepValue: number,
    maximumCollateral: number,
    interestStep: number
  ) {
    return setupContract(
      'vault_contract',
      'new',
      oracleAddress,
      collateralTokenAddress,
      emittedTokenAddress,
      interestRateStepValue,
      maximumCollateral,
      interestStep
    );
  }

  it('deployment test', async () => {
    const { contract: oracleContract } = await setupOracle();
    const { contract: emittedTokenContract } = await setupEmmitedToken();
    const { contract: collateralTokenContract } = await setupCollateralMock();
    const { contract: measurerContract } = await setupMeasurer(oracleContract.address.toString());
    const {
      query,
      defaultSigner: sender,
      contract: vaultContract,
    } = await setupVault(
      oracleContract.address.toString(),
      collateralTokenContract.address.toString(),
      emittedTokenContract.address.toString(),
      0,
      0,
      0
    );
    const { contract: vaultControllerContract } = await setupController(
      measurerContract.address.toString(),
      vaultContract.address.toString()
    );
    await expect(vaultContract.tx.setControllerAddress(vaultControllerContract.address.toString())).to.eventually.be.fulfilled;
    // Assert - Sender is by default the owner of the contract

    await expect(vaultContract.tx.createVault()).to.eventually.be.fulfilled;
    await expect(query.totalSupply()).to.have.output(1);
    await expect(query.ownerOf({ u128: 0 })).to.have.output(sender.address);
  });

  it('creates a vault and mints an nft', async () => {
    const { contract: oracleContract } = await setupOracle();
    const { contract: emittedTokenContract } = await setupEmmitedToken();
    const { contract: collateralTokenContract } = await setupCollateralMock();
    const { contract: measurerContract } = await setupMeasurer(oracleContract.address.toString());
    const {
      query,
      defaultSigner: sender,
      contract: vaultContract,
    } = await setupVault(
      oracleContract.address.toString(),
      collateralTokenContract.address.toString(),
      emittedTokenContract.address.toString(),
      0,
      0,
      0
    );
    const { contract: vaultControllerContract } = await setupController(
      measurerContract.address.toString(),
      vaultContract.address.toString()
    );
    await expect(vaultContract.tx.setControllerAddress(vaultControllerContract.address.toString())).to.eventually.be.fulfilled;

    // Assert - Sender is by default the owner of the contract

    await expect(vaultContract.tx.createVault()).to.eventually.be.fulfilled;
    await expect(query.totalSupply()).to.have.output(1);
    await expect(query.ownerOf({ u128: 0 })).to.have.output(sender.address);
  });

  it('not an owner creates a vault and gets an nft minted', async () => {
    const { contract: oracleContract } = await setupOracle();
    const { contract: emittedTokenContract } = await setupEmmitedToken();
    const { contract: collateralTokenContract } = await setupCollateralMock();
    const { contract: measurerContract } = await setupMeasurer(oracleContract.address.toString());
    const {
      query,
      accounts: [alice],
      contract: vaultContract,
    } = await setupVault(
      oracleContract.address.toString(),
      collateralTokenContract.address.toString(),
      emittedTokenContract.address.toString(),
      0,
      0,
      0
    );
    const { contract: vaultControllerContract } = await setupController(
      measurerContract.address.toString(),
      vaultContract.address.toString()
    );
    await expect(vaultContract.tx.setControllerAddress(vaultControllerContract.address.toString())).to.eventually.be.fulfilled;

    // Assert - Sender is by default the owner of the contract
    await expect(fromSigner(vaultContract, alice.address).tx.createVault()).to.eventually.be.fulfilled;
    await expect(query.ownerOf({ u128: 0 })).to.have.output(alice.address);
  });

  it('creates a vault and destroys it', async () => {
    const { contract: oracleContract } = await setupOracle();
    const { contract: emittedTokenContract } = await setupEmmitedToken();
    const { contract: collateralTokenContract } = await setupCollateralMock();
    const { contract: measurerContract } = await setupMeasurer(oracleContract.address.toString());
    const { contract: vaultContract } = await setupVault(
      oracleContract.address.toString(),
      collateralTokenContract.address.toString(),
      emittedTokenContract.address.toString(),
      0,
      0,
      0
    );
    const { contract: vaultControllerContract } = await setupController(
      measurerContract.address.toString(),
      vaultContract.address.toString()
    );
    await expect(vaultContract.tx.setControllerAddress(vaultControllerContract.address.toString())).to.eventually.be.fulfilled;

    // Assert - Sender is by default the owner of the contract
    await vaultContract.tx.createVault();
    const id = vaultContract.abi.registry.createType('u128', 0);
    await expect(vaultContract.tx.destroyVault(id)).to.eventually.be.fulfilled;
  });

  it('fails to destroy a vault if it does not exist', async () => {
    const { contract: oracleContract } = await setupOracle();
    const { contract: emittedTokenContract } = await setupEmmitedToken();
    const { contract: collateralTokenContract } = await setupCollateralMock();
    const { contract: measurerContract } = await setupMeasurer(oracleContract.address.toString());
    const { contract: vaultContract } = await setupVault(
      oracleContract.address.toString(),
      collateralTokenContract.address.toString(),
      emittedTokenContract.address.toString(),
      0,
      0,
      0
    );
    const { contract: vaultControllerContract } = await setupController(
      measurerContract.address.toString(),
      vaultContract.address.toString()
    );
    await expect(vaultContract.tx.setControllerAddress(vaultControllerContract.address.toString())).to.eventually.be.fulfilled;

    // Assert - Sender is by default the owner of the contract
    await vaultContract.tx.createVault();
    await expect(vaultContract.tx.destroyVault(1)).to.eventually.be.rejected; //TODO check actual reason or rejectedWith
  });

  it('fails to destroy a vault if the owner is not the caller', async () => {
    const { contract: oracleContract } = await setupOracle();
    const { contract: emittedTokenContract } = await setupEmmitedToken();
    const { contract: collateralTokenContract } = await setupCollateralMock();
    const { contract: measurerContract } = await setupMeasurer(oracleContract.address.toString());
    const {
      contract: vaultContract,
      accounts: [alice],
    } = await setupVault(
      oracleContract.address.toString(),
      collateralTokenContract.address.toString(),
      emittedTokenContract.address.toString(),
      0,
      0,
      0
    );
    const { contract: vaultControllerContract } = await setupController(
      measurerContract.address.toString(),
      vaultContract.address.toString()
    );
    await expect(vaultContract.tx.setControllerAddress(vaultControllerContract.address.toString())).to.eventually.be.fulfilled;

    // Assert - Sender is by default the owner of the contract
    await vaultContract.tx.createVault();
    await expect(fromSigner(vaultContract, alice.address).tx.destroyVault(0)).to.eventually.be.rejected; //TODO check actual reason or rejectedWith
  });
});
