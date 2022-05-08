import { RedspotUserConfig } from "redspot/types";
import "@redspot/patract";
import "@redspot/chai";
import "@redspot/gas-reporter";
import "@redspot/known-types";
import "@redspot/watcher";
import "@redspot/explorer";
import "@redspot/decimals";
const types = {
  ContractsPsp34Id: {
    _enum: {
      U8: 'u8',
      U16: 'u16',
      U32: 'u32',
      U64: 'u64',
      U128: 'u128',
      Bytes: 'Vec<u8>'
    }
  },
  ContractsDiamondFacetCut: {
    hash: '[u8; 32]',
    selectors: 'Vec<[u8; 4]>'
  }
}
export default {
  defaultNetwork: "development",
  contract: {
    ink: {
      docker: false,
      toolchain: "nightly",
      sources: ["contracts/**/*"],
    },
  },
  networks: {
    development: {
      endpoint: "ws://127.0.0.1:9944",
      gasLimit: "400000000000",
      types,
      explorerUrl: 'https://polkadot.js.org/apps/#/explorer/query/?rpc=ws://127.0.0.1:9944/'
    },
    jupiter: {
      endpoint: "wss://jupiter-poa.elara.patract.io",
      gasLimit: "400000000000",
      accounts: ["//Alice"],
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
