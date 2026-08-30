# Changelog

All notable changes to the PadiPay Soroban Escrow Contracts will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **Event Schema Versioning**: Every contract event now emits a schema version
  symbol (`v1`) as its second topic, directly after the event name. Off-chain
  indexers can filter by `(event_name, version)` and skip events whose schema
  they do not recognise instead of silently misparsing them. The version is
  exported as `events::EVENT_SCHEMA_VERSION`; the strategy is documented in
  `docs/integration.md`.

## [0.1.0] - 2026-07-10

### Added
- **Core Escrow Lifecycle (Happy Path MVP)**: Delivered the fundamental state machine allowing for escrow creation, funding, and resolution.
- **Escrow Creation**: Buyers can successfully create escrow agreements assigning a seller and a fixed amount.
- **Funds Locking**: Buyers can deposit Stellar assets into the contract, securely locking the tokens under the contract's custody.
- **Funds Release**: Buyers can explicitly release locked funds to the designated seller.
- **Refunds**: Buyers can be refunded if the escrow is cancelled or aborted from a locked state.
- **Role-Based Authorization**: Integrated Soroban SDK `require_auth` to ensure only the designated Buyer or Seller can perform privileged state transitions.
- **Contract Events**: Added Soroban events (`EscrowCreated`, `FundsLocked`, `FundsReleased`, `EscrowRefunded`) for off-chain indexing.
- **Stellar Testnet Deployment**: Successfully verified and deployed the initial v0.1.0 MVP contract to the Stellar Testnet.

### Changed
- **Architecture Refactoring**: Separated contract logic into distinct modules (state, storage, token, validation, events) to keep the codebase maintainable.
- **Continuous Integration**: Configured automated GitHub Actions workflows for formatting, linting, and unit testing (`wasm32v1-none` target).
- **Documentation**: Overhauled `README.md`, `CONTRIBUTING.md`, and deployment guides to align with the current MVP scope and assist future open-source contributors.

### Known Limitations
- The current contract assumes a strict happy path with full cooperation between the Buyer and Seller.
- Escrow agreements do not currently have expiration dates or timeout triggers.
- Partial releases of locked funds are not supported.

### Intentionally Deferred
- **Human Oracle Integration**: Dispute resolution via third-party mediators has been deferred to `v0.3.0`.
- **Contract Hardening**: Expiration times, cancellation limits, and extensive storage optimization have been deferred to `v0.2.0`.
- **Production Readiness**: Protocol fee extraction and complex milestone payments are deferred to `v0.4.0`.

*For an overview of upcoming work and contributor opportunities, see the [Project Roadmap](docs/roadmap.md).*
