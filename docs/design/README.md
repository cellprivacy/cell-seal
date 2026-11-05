# Cell Seal: Sandbox Design

Design of the two apps of the **sandbox**, the reference tokenization platform with the Cell Seal
rail integrated. They are a worked SDK integration and an evaluation environment, not the product
([`../integration.md`](../integration.md) section 1). Both live in one file.

- Source file: [`cell-seal.pen`](cell-seal.pen), open with Pen (pen.dev)
- Exported images: [`screens/fund/`](screens/fund/) and [`screens/investor/`](screens/investor/)
- Colors and fonts: background `#0A0B0D`, accent `#C8F94E`, Inter + JetBrains Mono
- Logo mark and the Cell Seal wordmark (Space Grotesk 700) follow the Cell Protocol brand kit

| App | Who uses it | What for |
|---|---|---|
| **Fund console** | The fund's ops staff | Create a fund, approve investors, process orders, set the price, settlement, register, audit |
| **Investor app** | KYC-approved investors | See the funds on the platform, invest, redeem, track the portfolio |

The sandbox implements Case 1, a fund that has no token yet (screens 01-03). Procedures for both
onboarding cases: [`../onboarding.md`](../onboarding.md).

## Fund console

Follows the order in which a fund works, from nothing to weekly operation.

| # | Screen | What the fund does | [`flows.md`](../flows.md) |
|---|---|---|---|
| 01 | [Set up: Fund details](screens/fund/01-setup-fund-details.png) | Create the fund and the share token, the dealing calendar, how investors pay | Section 2 |
| 02 | [Set up: Who can read amounts](screens/fund/02-setup-who-can-read-amounts.png) | Choose the viewing key holders and the k-of-n threshold | Section 2 |
| 03 | [Investors](screens/fund/03-investors.png) | Add, approve, freeze investors | Section 2 |
| 04 | [Dashboard](screens/fund/04-dashboard.png) | What needs doing today, this week's settlement | - |
| 05 | [Orders](screens/fund/05-orders.png) | Order list and status | Section 1 |
| 06 | [Subscription order](screens/fund/06-subscription-order.png) | Confirm money received (manual), price, issue shares | Section 3 |
| 07 | [NAV: Set price](screens/fund/07-nav-set-price.png) | Set the period's price, price the pending orders | Section 3, section 4 |
| 08 | [Redemption order](screens/fund/08-redemption-order.png) | Receive shares, price, pay out (manual) | Section 4 |
| 09 | [Settlement](screens/fund/09-settlement.png) | The period's net, the only number that becomes public | Section 6 |
| 10 | [Register](screens/fund/10-register.png) | The investor register rebuilt from decrypted data | Section 7 |
| 11 | [Audit access](screens/fund/11-audit-access.png) | Grant time-limited read access, needs enough approving parties | Section 7 |

## Investor app

| # | Screen | What the investor does | [`flows.md`](../flows.md) |
|---|---|---|---|
| 01 | [Set up account](screens/investor/01-set-up-account.png) | Connect the wallet, KYC, sign one message to create the privacy key (derived from the wallet signature, no separate seed phrase) | Section 2 |
| 02 | [Funds](screens/investor/02-funds.png) | See every fund on the platform. Invest or request access | - |
| 03 | [Fund detail](screens/investor/03-fund-detail.png) | Public information about the fund and the token, plus the investor's own position | - |
| 04 | [Invest](screens/investor/04-invest.png) | Enter the amount, choose how to pay, place the order | Section 3 |
| 05 | [Order](screens/investor/05-order.png) | Track the order. Get the bank transfer details | Section 3 |
| 06 | [Portfolio](screens/investor/06-portfolio.png) | Holdings and recent orders | - |
| 07 | [Redeem](screens/investor/07-redeem.png) | Enter the number of shares, choose how to receive the money, sign the order | Section 4 |

### What an investor sees about a fund

The Funds and Fund detail pages show **public** data only, what anyone can read from the chain
or what the fund publishes. The only non-public item on those pages is the investor's own
position, marked "Only you can see this" and decrypted on the device.

| Public | Source |
|---|---|
| Fund name, symbol, fund manager, asset class, currency | Published by the fund |
| Token contract, issuer account, network | Chain |
| Price per share and price history | Published by the fund every dealing day |
| Fund size, total shares outstanding | Chain: the wrapper's balance. Changes only at each settlement |
| Number of investors | Chain: holder addresses are public |
| Dealing calendar, cut-off, settlement, fees, minimums, payment methods | Published by the fund |

| Never shown to others | |
|---|---|
| Each investor's shares and value | Only that investor and the fund's viewing key holders |
| The amount of each order | As above |

## UI principles

- The user's language, not cryptography's: "Bank transfer" / "EURC" instead of DoP / DvP,
  "price per share" instead of commitment or proof.
- One primary action per screen. An order is shown as steps. The current step carries the button.
- Privacy is mentioned only where it changes the user's decision: the net figure will be public
  (fund console) and "Only you can see this" on private figures (investor app).

## Not in the sandbox

- Secondary transfer between two investors. The SDK supports it ([`../sdk.md`](../sdk.md)
  section 4.5). A platform builds its own screen.
- The flow to request access to a fund. In the sandbox the fund adds investors from the console.
