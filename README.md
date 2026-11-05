# Cell Seal

**Confidential settlement for tokenized funds on Stellar.**

Cell Seal lets a tokenized fund keep investor balances and order sizes private while preserving the holder visibility a regulated fund needs. Holder addresses stay public. The product is designed to give the issuer, transfer agent and regulator controlled access to amounts through a shared viewing key.

This is the project repository for Cell Seal's **Stellar Community Fund #46 Open Track** submission. It brings together the Soroban prototype, Noir circuits and technical design supporting the application.

## How it works

A fund's SEP-41 token is wrapped 1:1 into sealed balances. Share transfers carry an order reference and a cryptographic proof, which a Soroban contract verifies without publishing the amount. Subscriptions and redemptions are netted over a settlement window, so the public wrap or unwrap reveals the window net rather than each order size.

Cell Seal builds on OpenZeppelin's Confidential Token primitive and Nethermind's UltraHonk verifier. The target product combines settlement contracts, a TypeScript SDK and onboarding tools for tokenization platforms and transfer agents. Platforms retain their fund token, mint and burn authority, KYC process and order lifecycle.

## For SCF reviewers

- [Technical Architecture Document](docs/technical-architecture.md): architecture, Stellar integration, security, testnet evidence and tranche roadmap. Appendix A lists the prototype contract IDs and cycle transactions.
- [Technical design index](docs/README.md): detailed specifications for contracts, cryptography, keys, SDK, integration and the evaluation sandbox.
- [Published documentation](https://docs.cellprivate.com/seal).

## Testnet prototype

The self-funded prototype completed a subscription, settlement and redemption cycle on Stellar testnet with real UltraHonk proofs on 9 October 2026. The cycle wrapped a net of 1,000 shares, delivered 250 shares sealed, redeemed 100 shares sealed and closed the next window with a net of −100 shares. The transaction references are recorded in the [TAD's testnet appendix](docs/technical-architecture.md#appendix-a-testnet-contracts-and-transactions).

The source here covers five product contracts and a sample fund token:

| Contract | Role |
|---|---|
| `seal-token` | Confidential share wrapper with fund-specific policy hooks |
| `seal-verifier` | UltraHonk proof verification and verification-key registry |
| `seal-auditor` | Auditor viewing-key registry |
| `fund-policy` | Holder whitelist and KYC expiry checks |
| `settlement` | Order-linked sealed transfers and window net settlement |
| `share-token` | Sample fund token used by the prototype |

The prototype reflects the starting point described in the SCF application: fund registration lives inside `settlement` and `fund-policy`, and window close still mints or burns the sample fund token. The product-model changes are part of Tranche 1.

## SCF roadmap

| Tranche | Planned delivery |
|---|---|
| 1 — Product-model contracts and SDK v0 | Separate `fund-registry`, settlement that only wraps or unwraps net, existing-token onboarding, holder self-wrap, SDK v0 and reference indexer |
| 2 — Hardened testnet and sandbox | Key/admin timelocks, monitoring, TTL management, sealed stablecoin settlement, atomic DvP and evaluation apps |
| 3 — Disclosure and mainnet release | 2-of-3 viewing key and audit access, reproducible builds, platform pilot, SDK 1.0 and an audit-gated mainnet launch |

These are planned deliverables; the prototype does not yet implement the full product design. The SDK, backend and apps are not included in this repository's current source tree.

## Repository layout

| Path | Contents |
|---|---|
| `contracts/` | Soroban contracts, existing tests, proof fixtures and six verification keys |
| `circuits/` | Register, transfer, withdraw, set-spender, spender-transfer and clawback circuits, plus their shared Noir library |
| `docs/` | Technical Architecture Document and detailed product specifications |

The contracts pin OpenZeppelin `stellar-contracts` to commit `5ffda91`. The operation circuits and shared library originate from that upstream implementation.

## Build and test

Contract tests:

```sh
cd contracts
cargo test --workspace
```

Circuit tests, using Noir `1.0.0-beta.11`:

```sh
cd circuits
nargo test
```

The prototype verification keys use Barretenberg `0.87.0`. Proof fixtures and verification keys are included for the contract tests; a deployment or end-to-end proving run requires additional tooling.
