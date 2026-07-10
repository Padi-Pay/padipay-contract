# Local Setup Guide

Follow these steps to set up your environment for Soroban smart contract development.

## 1. Install Rust Toolchain

Install Rust using rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Add the `wasm32v1-none` target (required for compiling Soroban contracts to WebAssembly):
```bash
rustup target add wasm32v1-none
```

## 2. Install Stellar CLI

The `stellar` CLI is required to build, test, and deploy Soroban contracts.

```bash
cargo install --locked stellar-cli --features opt
```

## 3. Build the Contract

Compile the contract to WebAssembly:
```bash
stellar contract build
```
This will generate a `.wasm` file in the `target/wasm32v1-none/release/` directory.

## 4. Run Tests

Execute the unit tests locally:
```bash
cargo test
```
