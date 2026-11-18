---
layout: home
title: Cell Seal
hero:
  name: Cell Seal
  image:
    src: /logo.svg
    alt: Cell
  text: Confidential settlement for tokenized fund shares on Stellar
  tagline: The amount of every transaction is sealed on the public ledger. The issuer, the transfer agent and the regulator can still read everything.
  actions:
    - theme: brand
      text: Technical Architecture
      link: /seal/architecture
    - theme: alt
      text: Overview
      link: /seal/overview
    - theme: alt
      text: Contracts
      link: /seal/contracts
features:
  - title: Sealed
    details: Every amount on chain is a Pedersen commitment, not a readable number. Addresses, timestamps and order references stay public.
  - title: Proven
    details: Every transaction carries an UltraHonk proof of conservation, range and policy, verified on Soroban. The chain rejects a wrong number without reading it.
  - title: Disclosed
    details: The viewing key is split k-of-n between issuer, transfer agent and regulator. They rebuild 100% of the register. Cell Seal holds no key.
---
