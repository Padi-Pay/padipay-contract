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
| v0.1.0 — Happy Path MVP       | 🚧 In Progress |
| v0.2.0 — Contract Hardening   | 📋 Planned     |
| v0.3.0 — Human Oracle         | 📋 Planned     |
| v0.4.0 — Production Readiness | 📋 Planned     |

---

# v0.1.0 — Happy Path MVP

**Goal**

Deliver a deployable escrow contract demonstrating the complete happy path on Stellar Testnet.

## Features

### Escrow State

* [ ] EscrowStatus enum
* [ ] EscrowState model
* [ ] Storage keys
* [ ] Storage helpers

### Authentication

* [ ] Buyer authorization
* [ ] Seller authorization

### Token Operations

* [ ] Soroban Token Client
* [ ] Lock funds
* [ ] Release funds
* [ ] Refund buyer

### Contract Safety

* [ ] Input validation
* [ ] State transition validation
* [ ] Error handling
* [ ] Validation helpers

### Events

* [ ] EscrowCreated
* [ ] FundsLocked
* [ ] FundsReleased
* [ ] EscrowRefunded

### Testing

* [ ] Happy path tests
* [ ] Failure-path tests
* [ ] Authorization tests
* [ ] Shared test utilities

### Developer Experience

* [x] Improved README
* [x] GitHub Actions CI
* [x] Testnet deployment
* [ ] Release notes

---

# v0.2.0 — Contract Hardening

Focus on making the contract safer, more resilient, and easier to maintain.

## Planned Features

* Escrow expiration
* Cancellation improvements
* Storage optimization
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
* Dispute resolution
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

