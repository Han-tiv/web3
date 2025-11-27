{
  "name": "$PROJECT_NAME",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "test": "npx hardhat test",
    "poc": "npx hardhat test --grep '$VULN_TYPE'"
  },
  "devDependencies": {
    "@nomicfoundation/hardhat-toolbox": "^5.0.0",
    "dotenv": "^16.3.1",
    "hardhat": "^2.22.5"
  }
}

