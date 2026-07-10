# Deployment Guide

This document is divided into two sections: instructions on how to perform a deployment, and the recorded details of the actual deployment that has already taken place for the MVP.

---

## Part 1: Deployment Procedure

This section provides generic instructions for deploying the PadiPay MVP Escrow Contract to the Stellar network.

### Prerequisites

1. **Rust Toolchain:** Ensure you have Rust installed and the `wasm32v1-none` target added.
2. **Stellar CLI:** Ensure you have the `stellar` CLI installed.
   ```bash
   cargo install --locked stellar-cli --features opt
   ```

### 1. Build the Contract

First, compile the contract to WebAssembly. This command ensures the contract is optimized and checks for overflows:

```bash
stellar contract build
```

This will generate the optimized `.wasm` file at:
`target/wasm32v1-none/release/soroban_escrow_contracts.wasm`

### 2. Configure the Deployer Identity

You must select or generate a Stellar identity to deploy the contract.

To generate a new identity:
```bash
stellar keys generate <DEPLOYER_IDENTITY>
```

Once generated, you must fund the identity on the Testnet using Friendbot before you can deploy:
```bash
stellar keys fund <DEPLOYER_IDENTITY> --network testnet
```

### 3. Deploy the Contract

Run the following command to deploy the contract to the Stellar Testnet:

```bash
stellar contract deploy \
    --wasm target/wasm32v1-none/release/soroban_escrow_contracts.wasm \
    --source <DEPLOYER_IDENTITY> \
    --network testnet
```

### 4. Record the Output

Upon successful deployment, the CLI will output the deployed **Contract ID**.

---

## Part 2: Recorded Testnet Deployment

This section records the details of the actual deployment of the PadiPay MVP contract to the Stellar Testnet.

### Deployment Details

- **Network:** Stellar Testnet
- **Deployment Date:** 2026-07-11
- **Deployer Identity:** `padipay-deployer`
- **Contract ID:** 
[CBREAC6HOK5EUD43NXBTMPEEYOBWMNLPPQYBBI4LJAPCOIWO4MUBI6UF](https://lab.stellar.org/smart-contracts/contract-explorer?$=network$id=testnet&label=Testnet&horizonUrl=https:////horizon-testnet.stellar.org&rpcUrl=https:////soroban-testnet.stellar.org&passphrase=Test%20SDF%20Network%20/;%20September%202015;&smartContracts$explorer$contractId=CBREAC6HOK5EUD43NXBTMPEEYOBWMNLPPQYBBI4LJAPCOIWO4MUBI6UF;;)

### Verification

To verify the contract is live, you can query the contract's interface or invoke a view function using the actual Contract ID and deployer alias:

```bash
stellar contract invoke \
    --id CBREAC6HOK5EUD43NXBTMPEEYOBWMNLPPQYBBI4LJAPCOIWO4MUBI6UF \
    --source padipay-deployer \
    --network testnet \
    -- \
    --help
```
