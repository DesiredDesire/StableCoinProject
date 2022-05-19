import { RedspotUserConfig } from 'redspot/types';
import '@redspot/patract';
import '@redspot/chai';
import '@redspot/gas-reporter';
import '@redspot/known-types';
import '@redspot/watcher';
import '@redspot/explorer';
import '@redspot/decimals';

const types = {
  ContractsPsp34Id: {
    _enum: {
      U8: 'u8',
      U16: 'u16',
      U32: 'u32',
      U64: 'u64',
      U128: 'u128',
      Bytes: 'Vec<u8>',
    },
  },
  ContractsErrorsPsp22Psp22Error: {
    _enum: {
      Custom: 'String',
      InsufficientBalance: null,
      InsufficientAllowance: null,
      ZeroRecipientAddress: null,
      ZeroSenderAddress: null,
      SafeTransferCheckFailed: 'String',
    },
  },
  ContractsErrorsOwnableOwnableError: {
    _enum: {
      CallerIsNotOwner: null,
      NewOwnerIsZero: null,
    },
  },
  ContractsErrorsPausablePausableError: {
    _enum: {
      Paused: null,
      NotPaused: null,
    },
  },
  ContractsErrorsPsp34Psp34Error: {
    _enum: {
      Custom: 'String',
      SelfApprove: null,
      NotApproved: null,
      TokenExists: null,
      TokenNotExists: null,
      SafeTransferCheckFailed: 'String',
    },
  },
  ContractsErrorsPsp22ReceiverPsp22ReceiverError: {
    _enum: {
      TransferRejected: 'String',
    },
  },

  StableCoinProjectCollaterallingCollaterallingError: {
    _enum: {
      OwnableError: 'ContractsErrorsOwnableOwnableError',
      PSP22Error: 'ContractsErrorsPsp22Psp22Error',
      PSP22ReceiverError: 'ContractsErrorsPsp22ReceiverPsp22ReceiverError',
    },
  },

  StableCoinProjectEmitingEmitingError: {
    _enum: {
      PausableError: 'ContractsErrorsPausablePausableError',
      CouldntMint: null,
      PSP22Error: 'ContractsErrorsPsp22Psp22Error',
    },
  },

  StableCoinProjectSPControllingSPControllingError: {
    _enum: {
      Generator: null,
      NoProfit: null,
      One: null,
      PausableError: 'ContractsErrorsPausablePausableError',
      OwnableError: 'ContractsErrorsOwnableOwnableError',
      PSP22Error: 'ContractsErrorsPsp22Psp22Error',
      EmittingError: 'StableCoinProjectEmitingEmitingError',
    },
  },

  StableCoinProjectSPGeneratingSPGeneratingError: {
    _enum: {
      Controller: null,
      PausableError: 'ContractsErrorsPausablePausableError',
      OwnableError: 'ContractsErrorsOwnableOwnableError',
      PSP22Error: 'ContractsErrorsPsp22Psp22Error',
    },
  },

  StableCoinProjectStableControllerStableControllerError: {
    _enum: {
      CouldntFeed: null,
      OwnableError: 'ContractsErrorsOwnableOwnableError',
      MeasuringError: 'StableCoinProjectMeasuringMeasuringError',
      PSP22Error: 'ContractsErrorsPsp22Psp22Error',
    },
  },

  StableCoinProjectVaultControllerVaultControllerError: {
    _enum: {
      CouldntFeed: null,
      MeasuringError: 'StableCoinProjectMeasuringMeasuringError',
      VaultError: 'StableCoinProjectVaultVaultError',
    },
  },

  StableCoinProjectSPControllingSPControllingError: {
    _enum: {
      Generator: null,
      NoProfit: null,
      One: null,
      PSP22Error: 'ContractsErrorsPsp22Psp22Error',
      OwnableError: 'ContractsErrorsOwnableOwnableError',
      SPGeneratingError: 'StableCoinProjectSPGeneratingSPGeneratingError',
    },
  },

  StableCoinProjectVaultVaultError: {
    _enum: {
      VaultController: null,
      OwnerUnexists: null,
      DebtUnexists: null,
      CollateralUnexists: null,
      HasDebt: null,
      NotEmpty: null,
      VaultOwnership: null,
      CollateralBelowMinimum: null,
      CollateralAboveMinimum: null,
      PSP22Error: 'ContractsErrorsPsp22Psp22Error',
      PSP34Error: 'ContractsErrorsPsp34Psp34Error',
      PausableError: 'ContractsErrorsPausablePausableError',
      OwnableError: 'ContractsErrorsOwnableOwnableError',
      CollaterallingError: 'StableCoinProjectCollaterallingCollaterallingError',
      EmittingError: 'StableCoinProjectEmitingEmitingError',
    },
  },
  ContractsDiamondFacetCut: {
    hash: '[u8; 32]',
    selectors: 'Vec<[u8; 4]>',
  },
};

export default {
  defaultNetwork: 'development',
  contract: {
    ink: {
      docker: false,
      toolchain: 'nightly',
      sources: ['contracts/**/*'],
    },
  },
  networks: {
    development: {
      endpoint: 'ws://127.0.0.1:9944',
      gasLimit: '400000000000',
      types,
      explorerUrl: 'https://polkadot.js.org/apps/#/explorer/query/?rpc=ws://127.0.0.1:9944/',
    },
    jupiter: {
      endpoint: 'wss://jupiter-poa.elara.patract.io',
      gasLimit: '400000000000',
      accounts: ['//Alice'],
      types,
    },
  },
  mocha: {
    timeout: 60000,
  },
  docker: {
    sudo: false,
    runTestnet:
      "docker run -p 9944:9944 --rm redspot/contract /bin/bash -c 'canvas --rpc-cors all --tmp --dev --ws-port=9944 --ws-external'",
  },
} as RedspotUserConfig;
