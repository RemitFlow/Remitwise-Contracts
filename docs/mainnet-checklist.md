# Mainnet Checklist

This document provides a pre-launch and post-launch verification checklist for deploying the RemitFlow contract to Stellar Mainnet.

---

## Pre-Launch Requirements

- [ ] **Security Audit**: All smart contract code has undergone a formal security audit and all high/medium severity findings are addressed.
- [ ] **Testnet Verification**: The contract version has been deployed to Testnet and passed end-to-end smoke testing.
- [ ] **Admin Key Management**: The administrator key uses a hardware wallet or multisig setup (e.g., Gnosis Safe/Stellar multisig).
- [ ] **WASM Optimization**: The production WASM binary is compiled under release profile and does not exceed the 64 KB budget limit.
- [ ] **Deployment Identity Funding**: The deploying identity has sufficient XLM balance for transaction and storage rent reserves.

---

## Post-Launch Verification

- [ ] Follow the complete [Post-Deploy Verification Checklist](post-deploy-verification.md) to confirm:
  1. WASM bytecode hash matches release build.
  2. `initialize` was called with correct admin and token addresses.
  3. `counter` starts at 0 and contract is unpaused.
  4. Allowlist configuration for operational callers.
  5. Off-chain indexers and event monitoring system are active.
