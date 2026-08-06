# PadiPay Contracts: Security & Threat Model

This document outlines the security posture, trust assumptions, and threat model of the PadiPay Soroban smart contracts. It is intended for contributors, auditors, and users to understand the guarantees and limitations of the system.

## 1. Trust Assumptions

PadiPay operates as a trust-minimized escrow service. The core assumptions are:
- **Soroban Environment:** We assume the underlying Stellar network and the Soroban runtime execute instructions correctly, enforce authorization via `require_auth()`, and handle integer overflow/underflow safely (Rust panics on overflow by default in debug, but we rely on Soroban's safe math environment).
- **Mediator Neutrality:** When a mediator is assigned to an escrow, they are trusted to act impartially in the event of a dispute. The contract *does not* mathematically constrain the mediator's decision; it only enforces that *only* the authorized mediator can route funds during a dispute.
- **Ledger Time:** We assume `env.ledger().timestamp()` is a reliable source of truth for enforcing time-locks and expirations.

## 2. Authorization Model

The contract strictly enforces authorization for state-changing operations using Soroban's native `require_auth()`.

- **Buyer:**
  - Authorized to `create_escrow`.
  - Authorized to `lock_funds` (requires the buyer to sign the token transfer).
  - Authorized to trigger `execute_timeout` (refunds the buyer if the deadline has passed).
- **Seller:**
  - Cannot initiate or lock funds.
  - No direct authorization required in the happy path (the buyer releases funds to them).
- **Mediator:**
  - Authorized to `resolve_dispute`. Only the explicitly assigned mediator for a specific `EscrowId` can force a state transition to `Released` or `Refunded`.

## 3. State Machine Invariants

The escrow lifecycle is governed by a strict state machine defined in `EscrowStatus`.
- **Invariant 1 (Creation):** An escrow must begin in the `Created` state.
- **Invariant 2 (Funding):** Funds can only be locked if the state is exactly `Created`.
- **Invariant 3 (Terminal States):** `Released` and `Refunded` are terminal. No further state transitions are permitted once an escrow reaches these states.
- **Invariant 4 (Time-Locks):** A timeout (`execute_timeout`) can only be executed if the ledger timestamp strictly exceeds the `deadline` specified at creation, and the escrow is currently `Locked`.

## 4. Storage Invariants

- **Escrow ID Uniqueness:** Each escrow is assigned a unique `EscrowId` derived from a globally incrementing nonce (`DataKey::EscrowNonce`). ID collisions are impossible as long as the nonce does not overflow `u64`.
- **Data Integrity:** The `EscrowState` is written to persistent storage. It contains immutable parameters (`buyer`, `seller`, `token`, `amount`, `deadline`) and a mutable `status`. The immutable parameters cannot be modified after creation.

## 5. Threat Model & Attack Surface

### 5.1 Reentrancy
- **Vector:** A malicious token contract could attempt to reenter the `lock_funds` or `release_funds` functions during a `transfer` call.
- **Mitigation:** Soroban natively prevents reentrancy at the host level. Furthermore, the contract updates its state *before* invoking external token transfers (Checks-Effects-Interactions pattern), rendering reentrancy attacks ineffective.

### 5.2 Griefing / Permanent Fund Locking
- **Vector:** A seller refuses to deliver goods, and the buyer refuses to release funds. Without intervention, funds remain locked indefinitely.
- **Mitigation:** The `deadline` parameter introduces a time-lock. If the deadline passes, the `execute_timeout` function can be called to recover the funds, preventing indefinite locking.

### 5.3 Authorization Bypass
- **Vector:** An attacker attempts to release funds from an escrow they do not own.
- **Mitigation:** Every state transition function enforces either `require_auth()` from the appropriate party or strict role validation against the persisted `EscrowState`.

### 5.4 Nonce Predictability
- **Vector:** An attacker predicts the next `EscrowId` and attempts to pre-calculate or front-run creation.
- **Mitigation:** While the nonce is predictable, `EscrowId` is purely an internal identifier. Knowing it in advance provides no economic advantage or exploit vector, as all sensitive operations require cryptographic authorization from the buyer/seller.

## 6. Known Limitations & Future Security Improvements

- **Single Point of Failure (Mediator):** Currently, if the designated mediator loses their private key, disputes cannot be resolved. Future iterations will introduce a decentralized mediator registry and multi-mediator voting.
- **Emergency Circuit Breaker:** The contract currently lacks a global `pause` functionality. Adding an admin-controlled circuit breaker is a high-priority future enhancement to freeze new escrows in the event of a zero-day exploit.
- **Formal Verification:** The state machine has been tested extensively via integration tests, but mathematical formal verification of the transitions has not yet been performed.
