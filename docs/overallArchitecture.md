# Smart Contract Architecture & State Flow

> [!NOTE]
> **Long-Term Vision**
> This document outlines the long-term architectural vision and ultimate state machine for the PadiPay platform. It includes features (like delivery confirmation and dispute resolution) that are beyond the scope of the current MVP. For the current MVP implementation details, please see [Architecture & State Flow](architecture.md).

This document outlines the core architecture and state machine for the PadiPay Soroban escrow contracts.

## Escrow State Machine

```text
                     +-------------------+
                     |                   |
                     |      LOCKED       | <--- Funds Deposited by Buyer
                     |                   |
                     +-------------------+
                               |
                        (Goods Delivered)
                               |
                               v
                     +-------------------+
                     |                   |
                     |     DELIVERED     | <--- Waiting for Buyer Confirmation
                     |                   |
                     +-------------------+
                               |
             +-----------------+-----------------+
             |                                   |
      (Buyer Confirms)                   (Issue Raised)
             |                                   |
             v                                   v
   +-------------------+               +-------------------+
   |                   |               |                   |
   |     RELEASED      |               |     DISPUTED      |
   |   (Terminal)      |               |                   |
   +-------------------+               +-------------------+
     Seller gets XLM                             |
                                         (Oracle Resolves)
                                                 |
                                 +---------------+---------------+
                                 |                               |
                           (Refund Buyer)                  (Pay Seller)
                                 |                               |
                                 v                               v
                       [ TERMINAL: REFUNDED ]         [ TERMINAL: RELEASED ]
```

The escrow contract transitions through the following states to ensure funds are securely managed between the Buyer and Seller.

1. **Locked**: 
   - Initial state when the Buyer deposits funds.
   - Funds are held in the contract.
   
2. **Delivered**:
   - The Seller has fulfilled the service or delivered the goods.
   - Waiting for the Buyer to confirm receipt.
   
3. **Released**:
   - The Buyer confirms receipt.
   - Funds are transferred to the Seller.
   - Terminal state.

4. **Disputed**:
   - Either the Buyer or Seller raises an issue before funds are released.
   - A Mediator (PadiPay admin or decentralized arbiter) steps in.
   - Terminal state resolution depends on the Mediator's decision.
   
## Contract Methods (Overview)
- `lock_funds`: Transitions state to `Locked`.
- `release_funds`: Transitions state from `Delivered` to `Released`.
- `resolve_dispute`: Transitions state from `Disputed` to a resolved outcome.
