require("dotenv").config();
require("@nomicfoundation/hardhat-toolbox");

/** @type import('hardhat/config').HardhatUserConfig */
const config = {
  solidity: "0.8.20",
  networks: {
    hardhat: {},
    custom: {
      url: process.env.RPC_URL || "http://127.0.0.1:8545",
      accounts: process.env.PRIVATE_KEY
        ? [process.env.PRIVATE_KEY]
        : []
    }
  }
};

module.exports = config;

