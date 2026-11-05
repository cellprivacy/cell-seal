# Cell Seal: Keys

> Every key in the system: how it is generated, who holds it, where it is stored, how it is recovered
> and rotated, and what happens when it is lost or exposed. The primitive's key derivation is copied
> from OpenZeppelin's `docs/sdk/key-derivation.md` (`v0.9.0`). Its MUST requirements are binding on
> `@cell-seal/sdk`.

## 1. Key inventory

| # | Key | Algorithm | Holder | Purpose |
|---|---|---|---|---|
| K1 | **Stellar key** | ed25519 | Each user, in their wallet | Signs transactions and authorization entries. Signs the messages that derive K2 and K4 |
| K2 | **Privacy key** of one account in one wrapper: `sk`, `vk`, `dvk_i` | Grumpkin | The account owner, in memory only | `sk`: build proofs. `vk`: decrypt. `dvk_i`: key of one delegation |
| K3 | **Ops root** of a fund | 32 random bytes | Fund ops with the `signer` role, via envelope | Root of the settlement account's privacy key and of the fund's box key |
| K4 | **Box key** | X25519 | Each user. Each fund | Sealed boxes for off-chain data |
| K5 | **Viewing key share** `k_j` | Grumpkin scalar | Each key holder | Participates in threshold decryption |
| K6 | **Platform admin key** | ed25519 multisig | The tokenization platform. In the sandbox, Cell Seal | Deploy, upgrade, write verification keys, `set_pilot_cap` |
| K7 | **Session token** | JWT, signed by the sandbox backend. A platform uses its own session mechanism | The browser | Calls to the order store |

The backend never holds K1-K5 in plaintext, at any point in time. "Backend" in this document means
the order store the SDK talks to: the sandbox backend, or the platform's own
([`sdk.md`](sdk.md) section 2.1).

## 2. Investor privacy key (K2)

### 2.1 Key tree

```
root --HKDF-SHA-512--> sk --Poseidon2(delta_vk, sk, addr_f)--> vk --Poseidon2(delta_dvk, vk, op_i)--> dvk_i
                       |                                      |
                       +--> Y = sk*H                          +--> PVK = vk*H
```

```
sk = RS( HKDF-SHA-512( IKM  = root,
                       salt = "openzeppelin/confidential-token/v1/sk",
                       info = be32(addr_f) || be32(acct_f) || le4(j) ) )
```

`addr_f` is the wrapper address, `acct_f` the account address, `RS` is rejection sampling into
`[1, r)` with counter `j`. A candidate is also rejected if the derived `vk` is 0. `sk` differs for
every (wrapper, account) pair, so an investor has one privacy key for the share wrapper of each fund
and one for the cash wrapper.

### 2.2 Signer root

For an account controlled by one ed25519 key, `root` is the SEP-0053 signature over the message:

```
"openzeppelin/confidential-token/v1/sk" || 0x0a || strkey(wrapper) || 0x0a || strkey(account)
```

| Spec requirement | What `@cell-seal/sdk` does |
|---|---|
| The message names both the wrapper and the account | A signature obtained by deception only unlocks that account on that wrapper |
| The signature must be checked against the expected public key | A wallet with the wrong account selected still returns a valid signature. Without the check, registration succeeds with an `sk` that can never be reproduced |
| The signature must be requested twice and compared | RFC 8032 ed25519 is deterministic. MPC or threshold wallets are not: the SDK detects this, refuses, and switches to a raw root |
| Never store the signature | The signature is equivalent to `sk`. Re-derive every session |
| Record the enrolled signer | An account can have several signers. `sk` is bound to the address, not to a signer. The backend stores the public key of the enrolled signer |
| Warn when the signer is rotated | A `set_options` that removes the enrolled signer leaves the sealed account alive but the root lost. The SDK compares the enrolled signer with the current signer set on every sync |
| State the single point of failure | The account set-up step of the platform's app must say: anyone who holds your wallet can read and spend your sealed balance |

The SDK checks the wallet at enrolment: it requests the root signature twice and compares. A wallet
that cannot sign messages, or signs non-deterministically, gets a raw root instead
(`E_NONDETERMINISTIC_SIGNER`, section 2.3).

### 2.3 Raw root

32 random bytes from a CSPRNG, used when the SEP-0053 path is unavailable or when the account is a
contract (smart account). A raw root cannot be reproduced from anything the user already has, so:

- The SDK forces a backup at creation time and does not proceed until it is confirmed.
- The backend records the **root form** of each account (`signer` | `raw` | `imported`), and the
  app only shows the recovery options that form supports.

### 2.4 During a session

| Item | Where | How long |
|---|---|---|
| Root signature | Local variable in the Web Worker | Until `sk` is derived, then overwritten with zeros |
| `sk`, `vk` | Web Worker memory | End of session, or after 15 minutes of inactivity |
| Wallet openings | IndexedDB, AES-GCM encrypted with a key derived from `vk` | Until logout. If exposed, only balances leak. Nothing can be spent |
| Page main thread | Never receives `sk`, `vk` or the root | n/a |

## 3. Settlement account and ops root (K3)

The settlement account has several operators who may change over time, so a signer root is not
usable.

```
ops_root (32 bytes, generated in the browser of whoever creates the fund)
  +--> privacy key of the settlement account in the share wrapper   (formula of section 2.1, root = ops_root)
  +--> privacy key of the settlement account in each cash wrapper
  +--> the fund's box key                                            (section 4)
```

| Role | Receives | Can do |
|---|---|---|
| `signer` | An envelope containing `ops_root` | Read every order of the fund. Build proofs for the settlement account |
| `viewer` | An envelope containing the `vk` of each wrapper and the private box key | Read every order. Cannot build proofs |

An envelope is a sealed box to the recipient's personal box key, stored in the order store (sandbox:
the `key_envelopes` table).
Adding a person: an existing `signer` opens their own envelope and re-seals it for the newcomer. The
backend only relays.

**Two layers of protection on spending.** A transfer from the settlement account needs both: a proof
(needs `sk`, hence `ops_root`) and the `require_auth` of the Stellar account itself (needs enough
signer weight). Exposure of `ops_root` allows *reading*, not *moving*. Conversely, capturing a signer
without `ops_root` cannot produce a proof.

**Removing a `signer` is a big deal.** That person already knows `ops_root`, and since `register` runs
only once, `sk` cannot be rotated in place. To revoke read access the fund must: create a new
settlement account with a new `ops_root`, move the whole sealed balance across, and update
`fund-registry`. The sandbox therefore gives the `viewer` role to most fund ops and keeps the number
of `signer`s minimal. In production `ops_root` lives in a signing service hosted by the platform or
by the fund, and fund ops never see it.

## 4. Box key (K4)

| Subject | Derived from |
|---|---|
| A user whose wallet can sign messages | SEP-0053 signature over `"cell-seal/v1/box" || 0x0a || strkey(fund-registry) || 0x0a || strkey(account)`, through `HKDF-SHA-512(salt = "cell-seal/v1/box")`, 32 bytes taken as the X25519 private key. The registry address identifies the platform, so one wallet has one box key per platform |
| A user on the raw-root path | `HKDF-SHA-512(IKM = raw root, salt = "cell-seal/v1/box")` |
| A fund | `HKDF-SHA-512(IKM = ops_root, salt = "cell-seal/v1/box", info = be4(fund_id))` |

The box-key message differs from the privacy-key message, so a signature leaked for one purpose does
not unlock the other. The box public key is registered with the backend at account creation together
with a wallet signature proving ownership. The client always verifies that signature before encrypting
to a box key.

## 5. Viewing key share (K5)

| Phase | Content |
|---|---|
| Generation | DKG at fund creation ([`cryptography.md`](cryptography.md) section 6.2). No party, Cell Seal included, ever sees `k` |
| Storage | The share `k_j` is sealed with a sealed box to the key holder's own box key and stored on the backend as a backup. The working copy lives in the memory of the key-holder client |
| Use | Only inside the key-holder client, only for `R_e` values that belong to an approved disclosure request |
| Key epoch | Every `rotate_key` creates a new *epoch*. Shares of old epochs must be kept to read that epoch's history |
| Replacing a key holder | Re-run the DKG with the new set, `rotate_key`. The replaced party's old share can still read the old history. This is a property of the primitive and cannot be undone |
| Losing a share | As long as `k_t` shares remain, no impact. Losing more than `n - k_t` makes the auditor channel of that epoch unreadable. The register is unaffected |

The default threshold is 2 of 3 (issuer, transfer agent, regulator). The regulator can join later: the
fund starts with 2 of 2 and re-runs the DKG when the regulator is ready.

## 6. Platform key (K6) and session (K7)

| Key | Rule |
|---|---|
| Platform admin | Multisig of at least 2 of 3, keys on hardware wallets. Every upgrade and every verification-key change goes through an on-chain timelock |
| Session (sandbox) | Login via a challenge signed by the wallet. JWT valid 15 minutes, refresh 24 hours. The token carries no key and no amount. A platform's own session must keep the last rule |

## 7. Loss and exposure

| Situation | Consequence | Handling |
|---|---|---|
| Investor loses the wallet (signer root) | Cannot read, cannot spend. The shares still sit in the on-chain commitment | The fund freezes the account, runs clawback ([`cryptography.md`](cryptography.md) section 6.4), and reissues to a new address. The clawback amount is public |
| Investor loses a raw root | As above | As above |
| Investor's wallet is compromised | The attacker can read and can build proofs. But every transfer must go through `settlement` to the settlement account or to a whitelisted investor, and redemption proceeds go to the registered bank account | Freeze immediately. Then as for a lost wallet |
| The enrolled signer is removed from the account | The root is lost, the sealed account is orphaned | The SDK warns beforehand (`E_SIGNER_ROTATED`). Afterwards, handle as a lost wallet |
| `ops_root` exposed | The attacker can read every order of the fund, past and future. Cannot move assets without a settlement-account signer | Create a new settlement account, move the balance, update the registry. Notify investors |
| A settlement-account signer is captured, `ops_root` safe | No proof can be built, so no sealed asset can move | Rotate the signer with `set_options` |
| One viewing-key share exposed | Harmless below the threshold | Re-run the DKG, `rotate_key` |
| `k_t` or more shares exposed | Every amount of that epoch is readable, forever | `rotate_key` stops the future. The past cannot be saved. This is why the shares must sit with different legal entities |
| Backend compromised | Metadata and ciphertexts leak. No amount, no key | Rotate the session secret. No cryptographic key needs rotation |
| The fund's box key exposed | Every off-chain order secret of the fund is readable | The box key derives from `ops_root`, so handle as an `ops_root` exposure |
