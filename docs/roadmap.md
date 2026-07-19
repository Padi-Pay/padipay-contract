# PadiPay Contracts Roadmap

> Living roadmap for the **PadiPay Soroban Escrow Contracts**.
>
> This document outlines the project's current progress, upcoming milestones, and future contributor opportunities. It should be updated as milestones are completed.

---

# Vision

PadiPay aims to provide a trust-minimized escrow protocol for informal commerce using Stellar Soroban smart contracts.

Our long-term goal is to power secure, low-cost escrow transactions for everyday trade while abstracting blockchain complexity through the broader PadiPay platform.

---

# Project Status

| Milestone                     | Status         |
| ----------------------------- | -------------- |
| v0.1.0 — Happy Path MVP       | ✅ Completed    |
| v0.2.0 — Contract Hardening   | 🚧 In Progress |
| v0.3.0 — Human Oracle         | 📋 Planned     |
| v0.4.0 — Production Readiness | 📋 Planned     |

---

# v0.1.0 — Happy Path MVP

**Goal**

Deliver a deployable escrow contract demonstrating the complete happy path on Stellar Testnet.

## Features

### Escrow State

* [x] EscrowStatus enum
* [x] EscrowState model
* [x] Storage keys
* [x] Storage helpers

### Authentication

* [x] Buyer authorization
* [x] Seller authorization

### Token Operations

* [x] Soroban Token Client
* [x] Lock funds
* [x] Release funds
* [x] Refund buyer

### Contract Safety

* [x] Input validation
* [x] State transition validation
* [x] Error handling
* [x] Validation helpers

### Events

* [x] EscrowCreated
* [x] FundsLocked
* [x] FundsReleased
* [x] EscrowRefunded

### Testing

* [x] Happy path tests
* [x] Failure-path tests
* [x] Authorization tests
* [x] Shared test utilities

### Developer Experience

* [x] Improved README
* [x] GitHub Actions CI
* [x] Testnet deployment
* [x] Release notes

---

# v0.2.0 — Contract Hardening

Focus on making the contract safer, more resilient, and easier to maintain.

## Planned Features

* Escrow expiration
* Cancellation improvements
* [x] Storage optimization (Multi-Escrow Manager architecture)
* Additional validation
* Improved error coverage
* Better event payloads
* Contract documentation
* Performance improvements
* Security review
* Additional integration tests

---

# v0.3.0 — Human Oracle

Introduce the Human-in-the-Loop dispute resolution layer that powers PadiPay's trust model.

## Planned Features

* Mediator role
* Oracle registry
* Dispute creation
* Evidence submission
* [x] Dispute resolution
* Oracle authorization
* Dispute events
* Administrative tooling

---

# v0.4.0 — Production Readiness

Prepare the contract for broader ecosystem adoption.

## Planned Features

* Milestone-based escrow
* Partial fund releases
* Configurable protocol fees
* Multi-party escrow
* Contract upgrade strategy
* Indexing support
* SDK improvements
* Fuzz testing
* Security audit
* Mainnet deployment

---

# Future Ideas

These ideas are intentionally deferred beyond the MVP and may become future community contributions.

* Escrow templates
* Merchant profiles
* Reputation integration
* Batch escrows
* Scheduled releases
* Stablecoin abstraction
* Multi-token support
* Analytics events
* Off-chain indexing service
* DAO-managed mediator registry

---

# Contribution Opportunities

New contributors are encouraged to start with issues labeled:

* `good first issue`
* `help wanted`
* `mvp`

More experienced contributors may explore:

* Authentication
* Token operations
* Testing
* Contract optimization
* Developer tooling

See:

* `CONTRIBUTING.md`
* GitHub Issues
* GitHub Project Board

---

# Definition of Success

The project reaches **v0.1.0** when:

* ✅ The contract compiles successfully.
* ✅ The contract deploys to Stellar Testnet.
* ✅ Buyers can create escrows.
* ✅ Funds can be locked.
* ✅ Funds can be released.
* ✅ Buyers can receive refunds.
* ✅ Lifecycle events are emitted.
* ✅ The complete test suite passes.
* ✅ Documentation is up to date.

---

# Guiding Principles

As the project evolves, we aim to:

* Keep pull requests small and reviewable.
* Prefer reusable, modular contract design.
* Maintain comprehensive test coverage.
* Document architectural decisions.
* Build in public and encourage community contributions.
* Deliver incremental milestones without overbuilding.

---

*Last Updated: v0.1.0 Development*

