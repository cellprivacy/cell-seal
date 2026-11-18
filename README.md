# Cell Seal: Technical Design

Cell Seal is the settlement layer for tokenized RWA funds on Stellar. The amount of every
transaction is sealed on the public ledger, while the issuer, the transfer agent and the regulator
can still read everything. It is sold to tokenization platforms as a contract set plus an SDK. The
platform keeps its fund token, its mint and burn rights, KYC and the order lifecycle.

This repository is the technical design document (TDD). It holds no application code.

Published at **[docs.cellprivate.com/seal/architecture](https://docs.cellprivate.com/seal/architecture)**:
the Technical Architecture Document written for the Stellar Community Fund Build Award, with every
document below as a page under `/seal/`. The site is built from `docs/` with VitePress and deployed
by Vercel from `vercel.json`. One folder per product: `docs/seal/` is this design; a second
product gets its own folder and URL prefix on the same site. Mermaid blocks are rendered to SVG at build time, so the pages carry
no diagram JavaScript. `pnpm docs:dev` serves it locally, `pnpm docs:build` fails on a dead link or
on a diagram that does not parse.

## Reading order

| # | Document | Scope | For | Answers |
|---|---|---|---|---|
| 0 | [`docs/technical-architecture.md`](docs/seal/technical-architecture.md) | Both | Everyone | Start here. Components, Stellar integration, order flow, security and monitoring, infrastructure, stack, licensing, what exists today, testnet addresses |
| 1 | [`docs/overview.md`](docs/seal/overview.md) | Both | Everyone | What the product does, for whom, what it does not do |
| 2 | [`docs/glossary.md`](docs/seal/glossary.md) | Both | Everyone | What each term means, and where it appears in the contracts and the SDK |
| 3 | [`docs/flows.md`](docs/seal/flows.md) | Product | Everyone | Each business flow: who does what, what goes on chain, what the market sees |
| 4 | [`docs/integration.md`](docs/seal/integration.md) | Product | Integrators | What the platform owns, what Cell Seal provides. Fund token conditions. Who mints, who burns. Reconciliation |
| 5 | [`docs/onboarding.md`](docs/seal/onboarding.md) | Product | Integrators | Bringing a fund onto the rail: Case 1 (no token yet) and Case 2 (existing token), step by step |
| 6 | [`docs/primitives.md`](docs/seal/primitives.md) | Product | Engineering | The OpenZeppelin Confidential Token: what is reused, what is built |
| 7 | [`docs/architecture.md`](docs/seal/architecture.md) | Both | Engineering | Components, what is product and what is sandbox, trust boundaries, data classification, data flows |
| 8 | [`docs/contracts.md`](docs/seal/contracts.md) | Product | Soroban | Each contract: functions, events, errors, permissions, invariants |
| 9 | [`docs/cryptography.md`](docs/seal/cryptography.md) | Product | Cryptography, SDK | Anatomy of a transfer, sealed box, DvP, threshold viewing key, openings |
| 10 | [`docs/keys.md`](docs/seal/keys.md) | Product | All engineering | Every key: generation, storage, recovery, rotation, loss |
| 11 | [`docs/sdk.md`](docs/seal/sdk.md) | Product | Integrators, frontend | The `@cell-seal/sdk` API and the host services it needs |
| 12 | [`docs/security.md`](docs/seal/security.md) | Both | Everyone, auditors | Threat model, leak catalogue, invariants, assurance plan, test strategy |
| 13 | [`docs/backend.md`](docs/seal/backend.md) | Sandbox | Backend | The sandbox backend: services, API, state machine, reference indexer, data model |
| 14 | [`docs/frontend.md`](docs/seal/frontend.md) | Sandbox | Frontend | The sandbox apps: routes, data handling, amounts, browser security |
| 15 | [`docs/design/`](docs/seal/design/README.md) | Sandbox | Product, frontend | The sandbox screens: fund console (11) and investor app (7) |

**Scope** says what each document describes. *Product* is what a tokenization platform receives:
the contracts, the SDK and the onboarding procedure. *Sandbox* is the reference tokenization
platform that ships with Cell Seal integrated, for evaluation. Nothing in it is sold or required.
*Both* covers the two.

## Four principles

1. **Cell Seal reads no amount.** Nothing Cell Seal ships holds a key that can read an amount.
   encryption, decryption and proof generation happen only in the browser of an authorized party.
   Any backend the SDK talks to, the sandbox's included, is amount-blind.
2. **The chain is the source of truth.** State changes only on final events or on signed
   attestations.
3. **The rail knows nothing about fund business.** NAV, cut-off and dealing days live in the
   tokenization platform's system. In the sandbox, in the backend and the fund console.
4. **Compose, do not rewrite.** The underlying cryptography belongs to the primitive. Cell Seal
   writes the parts nobody has built.

## Product boundary

| Layer | What | Form |
|---|---|---|
| Settlement contracts | `seal-token` wrapper, `settlement`, `fund-policy`, `seal-verifier`, `seal-auditor`, `fund-registry` | **Product.** One contract set per platform, one wrapper per fund |
| SDK | `@cell-seal/sdk`: keys, proofs, decryption, sealed box, contract clients, register | **Product.** Integrated into the platform's own apps and backend |
| Onboarding | The procedure and the checks for bringing a fund onto the rail, Case 1 and Case 2 | **Product.** Delivered with the SDK, [`docs/onboarding.md`](docs/seal/onboarding.md) |
| Host services | Stellar RPC, a permanent event archive, an order store for ciphertext | **Platform's.** The SDK needs them. The platform provides them in its own infrastructure, [`docs/sdk.md`](docs/seal/sdk.md) section 2.1 |
| Sandbox | A reference tokenization platform with Cell Seal integrated: backend (`gateway`, `orders`, `indexer`, Postgres), fund console, investor app | **Not the product.** For platforms to try the rail and to read as a worked integration. Not sold, not operated for real funds. Its `indexer` is a reference implementation of the event archive that a platform may run as is |

`settlement` never mints or burns the fund token. It wraps the net of a window when `net > 0` and
unwraps it when `net < 0`. The issuer mints and burns on the platform's own rail. Two onboarding
cases share the same contracts: a fund with no token yet (Case 1) and a fund with an existing
token and public holders (Case 2). See [`docs/onboarding.md`](docs/seal/onboarding.md).

## Status

| Part | Status |
|---|---|
| Product model | Decided 2026-10-24: contracts plus SDK sold to tokenization platforms. `settlement` does not mint or burn. The testnet prototype predates this decision. The required changes are in [`docs/integration.md`](docs/seal/integration.md) section 8 |
| Technical Architecture Document for SCF | Written 2026-11-09, published at `docs.cellprivate.com/seal/architecture` |
| Overview, glossary, flows, onboarding, design | Written |
| Primitive survey | Read from the upstream `v0.9.0` code on 2026-09-20 and measured on testnet on 2026-10-09 ([`docs/primitives.md`](docs/seal/primitives.md) section 8) |
| Technical design | Version 1, the basis for implementation |
| Code | [Contracts and circuits](https://github.com/cellprivacy/cell-seal). |

## Conventions

- English throughout. Technical terms follow [`docs/glossary.md`](docs/seal/glossary.md).
- **L-n**: an amount leak path, master list in `docs/security.md` section 3. **R-n**: an item in
  the scope of the independent cryptographic review, `docs/cryptography.md` section 10.
- Primitive signatures are copied from the upstream spec at the pinned commit. When the commit is
  bumped, re-check `docs/contracts.md` sections 3 to 4 and `docs/cryptography.md` sections 1 to 3.
- Diagrams are Mermaid fenced blocks. Stellar accounts are `G...`, contracts are `C...`.
- Amounts in prose are illustrative. The only public figure the design produces per window is
  `net`.
