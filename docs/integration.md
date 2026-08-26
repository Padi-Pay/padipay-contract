# PadiPay Escrow — Integration Guide

This guide documents the Soroban ABI/XDR shapes for the PadiPay escrow contract.
It is intended for backend Relayer API teams building transaction submissions.

---

## Contract Address

Deploy the compiled `soroban_escrow_contracts_optimized.wasm` and note the
 resulting contract address. All invocations target this address.

---

## XDR Type Reference

| Soroban type | XDR representation | Notes |
|---|---|---|
| `Address` | `PublicKey` (ed25519 or contract) | 32-byte key or contract hash |
| `i128` | `Int128Parts { hi: i64, lo: i64 }` | Signed 128-bit integer |
| `u64` | `Uint64` | Unsigned 64-bit (used for deadlines) |
| `u32` | `Uint32` | Unsigned 32-bit (used for ledger sequence) |
| `Option<u32>` | `Option<Uint32>` | XDR optional — `None` encoded as void |
| `Symbol` | `ScSymbol` | Short string literal (max 9 chars) |
| `EscrowId` (`u64`) | `Uint64` | Returned by `create_escrow` |

---

## Entrypoint Signatures

### Admin Operations

#### `initialize(admin: Address)`

One-time setup. Sets the global administrator.

| Parameter | Type | Description |
|---|---|---|
| `admin` | `Address` | Public key that will control admin operations |

> Fails with `AlreadyInitialized` if called more than once.

#### `update_admin(new_admin: Address)`

Transfers admin role. Requires current admin authorization.

| Parameter | Type | Description |
|---|---|---|
| `new_admin` | `Address` | The public key of the incoming admin |

#### `pause()` / `unpause()`

Halts or resumes escrow creation. Requires admin authorization.
No parameters beyond `env`.

---

### Escrow Operations

#### `create_escrow(…) → EscrowId`

Creates a new escrow agreement. This is the primary Phase 2 entrypoint with
the full parameter set.

| Parameter | Type | XDR shape | Description |
|---|---|---|---|
| `buyer` | `Address` | `PublicKey` | The buyer's public key. **Must authorize.** |
| `seller` | `Address` | `PublicKey` | The seller's public key. Must differ from `buyer`. |
| `token` | `Address` | `ContractAddress` | The Soroban token contract address to escrow. |
| `amount` | `i128` | `Int128Parts` | Amount in token base units. Must be > 0. |
| `deadline` | `u64` | `Uint64` | Unix timestamp after which `execute_timeout` may be called. |
| `mediator` | `Address` | `PublicKey` | Third-party mediator for dispute resolution. |
| `timeout_ledger` | `Option<u32>` | `Option<Uint32>` | **New in Phase 2.** Optional ledger sequence after which `refund_expired` may be called. Pass `None` to omit. |

**Returns:** `EscrowId` (`u64` / `Uint64`) — a monotonically increasing nonce
identifying this escrow.

**Authorization required:** `buyer` must sign the transaction.

**Phase 2 change:** The `mediator` and `timeout_ledger` parameters are new.
Phase 1 `create_escrow` did not include these fields.

```text
XDR illustration (simplified):

struct {
    Address buyer;        // ScAddress -> PublicKey
    Address seller;       // ScAddress -> PublicKey
    Address token;        // ScAddress -> ContractAddress
    i128     amount;      // { hi, lo }
    u64      deadline;    // Uint64
    Address  mediator;    // ScAddress -> PublicKey
    Option<u32> timeout_ledger;  // void | Uint32
}
```

---

#### `lock_funds(escrow_id: EscrowId)`

Transfers `amount` from the buyer's token balance into the contract.
Escrow must be in `Created` status.

| Parameter | Type | Description |
|---|---|---|
| `escrow_id` | `u64` | The escrow ID returned by `create_escrow` |

**Authorization required:** `buyer` must sign.

---

#### `release_funds(escrow_id: EscrowId)`

Transfers the locked amount from the contract to the seller.
Escrow must be in `Locked` status.

| Parameter | Type | Description |
|---|---|---|
| `escrow_id` | `u64` | The escrow ID |

**Authorization required:** `buyer` must sign.

---

#### `refund(escrow_id: EscrowId)`

Returns the locked amount to the buyer.
Escrow must be in `Locked` status.

| Parameter | Type | Description |
|---|---|---|
| `escrow_id` | `u64` | The escrow ID |

**Authorization required:** `seller` must sign.

---

#### `refund_expired(escrow_id: EscrowId)`

Returns funds to the buyer after the ledger-based timeout has elapsed.
The escrow must have been created with a `timeout_ledger` value, and the
current ledger sequence must exceed it.

| Parameter | Type | Description |
|---|---|---|
| `escrow_id` | `u64` | The escrow ID |

**Authorization required:** `buyer` must sign.

**Phase 2 new entrypoint.** Not available in Phase 1.

---

#### `execute_timeout(escrow_id: EscrowId)`

Returns funds to the buyer after the timestamp-based deadline has passed.
The current ledger timestamp must exceed the escrow's `deadline` field.

| Parameter | Type | Description |
|---|---|---|
| `escrow_id` | `u64` | The escrow ID |

**Authorization required:** None (permissionless after deadline).

---

#### `resolve_dispute(escrow_id: EscrowId, outcome: Symbol)`

Mediator-only dispute resolution. The mediator decides whether to refund the
buyer or pay the seller.

| Parameter | Type | XDR shape | Description |
|---|---|---|---|
| `escrow_id` | `u64` | `Uint64` | The escrow ID |
| `outcome` | `Symbol` | `ScSymbol` | `"refund_buyer"` or `"pay_seller"` |

**Authorization required:** `mediator` must sign.

**Accepted `outcome` values:**

| Symbol | Effect | Required escrow status |
|---|---|---|
| `"refund_buyer"` | Transfers amount back to buyer | `Locked` |
| `"pay_seller"` | Transfers amount to seller | `Locked` |

Any other symbol value returns `InvalidState`.

```text
XDR illustration (simplified):

struct {
    Uint64  escrow_id;
    Symbol  outcome;  // ScSymbol: "refund_buyer" | "pay_seller"
}
```

---

## Escrow Lifecycle & Status Transitions

```
Created ──► Locked ──► Released
                  │
                  └──► Refunded
```

| From | To | Triggered by |
|---|---|---|
| `Created` | `Locked` | `lock_funds` |
| `Locked` | `Released` | `release_funds` or `resolve_dispute("pay_seller")` |
| `Locked` | `Refunded` | `refund`, `refund_expired`, `execute_timeout`, or `resolve_dispute("refund_buyer")` |

---

## Error Codes

| Code | Name | Meaning |
|---|---|---|
| 1 | `Unauthorized` | Caller did not authorize the transaction |
| 2 | `InvalidState` | Operation not valid in current escrow status, or unknown dispute outcome |
| 3 | `EscrowNotFound` | No escrow exists for the given ID |
| 4 | `InvalidAmount` | Amount is zero or negative |
| 5 | `EscrowAlreadyFunded` | Attempted to lock funds on an already-funded escrow |
| 6 | `InvalidAddresses` | Buyer and seller are the same address |
| 7 | `DeadlineNotReached` | Deadline/timeout has not yet elapsed |
| 8 | `ContractPaused` | Contract is paused; new escrows cannot be created |
| 9 | `AlreadyInitialized` | `initialize` called more than once |
| 10 | `NotInitialized` | Contract has not been initialized |
| 11 | `InvalidTimeout` | `timeout_ledger` is in the past at creation time |

---

## Phase 1 vs Phase 2 Entrypoint Differences

| Aspect | Phase 1 | Phase 2 |
|---|---|---|
| `create_escrow` params | `buyer, seller, token, amount, deadline` | Adds `mediator: Address` and `timeout_ledger: Option<u32>` |
| Dispute resolution | Not supported | `resolve_dispute(escrow_id, outcome)` |
| Ledger-based timeout | Not supported | `refund_expired(escrow_id)` |
| Mediator role | Not present | Required in `create_escrow`; authorizes `resolve_dispute` |
| `timeout_ledger` | N/A | `Option<u32>` — `None` disables ledger timeout |

---

## Events

All events are published with a topic tuple:

| Event | Topics | Data |
|---|---|---|
| `EscrowCreated` | `(Symbol, escrow_id, buyer, seller)` | `amount: i128` |
| `FundsLocked` | `(Symbol, escrow_id, buyer, seller)` | `amount: i128` |
| `FundsReleased` | `(Symbol, escrow_id, buyer, seller)` | `amount: i128` |
| `EscrowRefunded` | `(Symbol, escrow_id, buyer, seller)` | `amount: i128` |
