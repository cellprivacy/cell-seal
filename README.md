# Cell Seal

Confidential settlement for tokenized fund shares on Stellar.

This repository contains the technical design and a subset of the existing testnet prototype. It does not implement the full target design.

## Prototype source

- `contracts/`: five product contracts (`seal-token`, `seal-verifier`, `seal-auditor`, `fund-policy`, `settlement`), plus the sample `share-token`, existing tests, proof fixtures and six verification keys.
- `circuits/`: six Noir operation circuits and their shared library, based on OpenZeppelin `stellar-contracts` at commit `5ffda91`.
- [Technical design](docs/README.md) and [Technical Architecture Document](docs/technical-architecture.md).

The source is copied from `cell-seal-mvp`. The SDK, backend, apps, witness helpers and deployment scripts are outside this subset. Verification keys and proof fixtures are copied from the prototype; they have not been regenerated here.

## Current scope

Fund registration remains inside `settlement` and `fund-policy`. At window close, `settlement` still mints or burns the sample fund token and wraps or unwraps the net. Transfers use `submit_leg`, with the settlement account at one end.

A separate `fund-registry`, settlement without mint/burn, holder self-wrap, sealed cash and atomic DvP, and 2-of-3 viewing-key disclosure are target-design work, not implemented by this prototype subset.

## Checks

```sh
cd contracts
cargo test --workspace
```

```sh
cd circuits
nargo test
```

The prototype pins OpenZeppelin dependencies to `5ffda91` and uses Noir `1.0.0-beta.11` with Barretenberg `0.87.0`.
