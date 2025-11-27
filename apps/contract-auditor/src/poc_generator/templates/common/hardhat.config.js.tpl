require("dotenv").config();
require("@nomicfoundation/hardhat-toolbox");

/** @type import('hardhat/config').HardhatUserConfig */
const config = {
  solidity: "0.8.20",
  networks: {
    hardhat: {},
    custom: {
      url: process.env.$ENV_RPC_URL || "http://127.0.0.1:8545",
      accounts: process.env.$ENV_PRIVATE_KEY
        ? [process.env.$ENV_PRIVATE_KEY]
        : []
    }
  }
};

module.exports = config;

