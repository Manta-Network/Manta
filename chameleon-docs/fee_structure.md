Since we’re working with a **fixed-supply tokenomics (100M total, with 65M reserved as a reward pool)**, the fee structure must supplement rewards and fund treasury operations so the network can remain sustainable long-term. Below is a **detailed, final fee structure proposal** — with explanations for why each decision is made.

# **Chameleon Network – Final Fee Structure (with Rationale)**

## **A) Shielding & Unshielding Fees**

**Hybrid Fee Model:**

* **Shielding (Public → Private):** `0.02% of amount OR 0.1 CHML (whichever is higher)`  
* **Unshielding (Private → Public):** `0.05% of amount OR 0.1 CHML (whichever is higher)`

**Allocation:**

* **Non-EVM Bridges (custodial, e.g., BTC, Solana):**  
  * 70% → Custodians (infra \+ security ops)  
  * 30% → Treasury

* **EVM Bridges (trustless contracts, e.g., Ethereum, BSC):**  
  * 70% → Bridge Ops Reserve (Treasury, used for audits/upgrades)  
  * 30% → Treasury

✅ **Why:**

* Hybrid (percentage \+ minimum) ensures small txns aren’t free and large txns still scale fairly.  
* Higher fee on unshielding discourages rapid in/out churn.  
* Custodians get steady compensation, while Treasury builds sustainable revenue.  
* Clear separation between custodial vs. trustless bridges ensures fairness.

## **B) pDEX Trading Fees**

* **Fee per Swap:** 0.25% (in line with Uniswap, PancakeSwap).  
* **Allocation:**

  * 90% → Liquidity Providers (LPs)  
  * 10% → Treasury

✅ **Why:**

* LPs earn real yield from fees (not just inflationary CHML rewards).  
* Treasury builds recurring revenue (unlike Incognito, which gave 100% to LPs).  
* Keeps fees competitive with public DEXes while offering **privacy as a premium feature**.

## **C) Network Gas Fees**

* **Flat Fee:** \~0.1 CHML per transaction **\[governance adjustable ie The community (through DAO/governance) can later vote to increase or decrease this fee.This is important because CHML’s price will fluctuate over time\]**

* **Applied To:** every on-chain action (send, swap, stake, provide liquidity, shield/unshield).  
* **Allocation:**  
  * 80% → Validators  
  * 20% → Treasury

✅ **Why:**

* Predictable flat fee prevents spam and covers validator costs.  
* Governance can adjust fee as CHML price changes (ensuring affordability \+ sustainability).  
* Treasury earns a share for infra/security.

## **D) Treasury Allocation & Sustainability**

* **Reward Pool (Fixed Supply):** 65M CHML → split between Validator Rewards \+ LP Rewards (already defined in tokenomics).

* **Treasury Revenue Streams:**  
  * 30% of Shield/Unshield fees  
  * 10% of pDEX fees  
  * 20% of Gas fees

✅ **Why:**

* Instead of relying solely on the reward pool distribution, the Treasury also builds self-sustaining revenue streams from fees (shield/unshield, pDEX, gas). Over time, this reduces reliance on the finite reward pool.  
* Ensures continuous funding for audits, development, ecosystem growth, and marketing.

## **E) Long-Term Sustainability Path**

1. **Early Stage (Mainnet Launch):**  
   * Rewards from the 65M CHML pool are the primary driver (validators \+ LPs).  
   * Fees supplement but are small.  
2. **Growth Stage (Increased Usage):**  
   * Shield/unshield \+ pDEX volume grows → Treasury revenue grows.  
   * CHML reward emissions can be tapered down (slower burn of 65M pool).  
3. **Maturity Stage (5–10 years):**  
   * Network runs on **fees primarily**, with reward pool nearly exhausted.  
   * Validators \+ LPs are incentivized by real activity (fees), not just reward pool  
   * Treasury becomes **financially independent** of token emissions.

## **✅ Final Summary (To Include in PRD)**

* **Shielding:** 0.02% or 0.1 CHML (whichever is higher)  
* **Unshielding:** 0.05% or 0.1 CHML (whichever is higher)  
* **pDEX Swaps:** 0.25% (90% LPs, 10% Treasury)  
* **Gas Fees:** \~0.1 CHML per transaction (80% Validators, 20% Treasury)  
* **Custodial Split:** Custodians 70%, Treasury 30%  
* **Trustless Split:** Bridge Ops Reserve 70%, Treasury 30%