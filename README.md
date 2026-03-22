# ZUS Protocol

Private reward distribution on Avalanche.

ZUS lets a campaign creator fund a reward drop without exposing the recipient list onchain. A claimant proves eligibility with a zk proof, the circuit derives a one-time stealth address, and `ZusProtocol` pays that stealth address.

![ZUS TUI](https://i.ibb.co/q3SLCCwk/Screenshot-2026-03-22-at-5-10-34-PM.png)

## Flow

1. The Rust API in `userslist/` creates a campaign and builds the Merkle tree.
2. The protocol stores the campaign root, verifier, message, and payout onchain.
3. The TUI fetches the claimant's Merkle path from the API.
4. Noir + Barretenberg generate a proof and public inputs.
5. `ZusProtocol` verifies the proof and pays the derived stealth address.

## TUI

The TUI in `tui/` is the offchain tool that drives the claim flow. It uses encrypted Foundry keystores plus `cast`, `nargo`, and `bb` to:

- resolve the wallet
- fetch claim inputs from the Rust API
- write `Prover.toml`
- generate the witness and proof
- call `previewClaim(...)`
- send `claim(...)` to `ZusProtocol`

For the current MVP, the TUI uses fixed demo values for `message` and `stealth_tweak`.

## Main Components

- `zus_addy/` - Noir circuit for Merkle membership, nullifier derivation, and stealth address derivation
- `userslist/` - Rust API for campaign creation and claim payloads
- `tui/` - Rust terminal app for proof generation and claiming
- `verifier/` - UltraHonk Solidity verifier
- `zusprotocol/` - protocol contract for campaigns, funding, and claims
- `frontend/` - React frontend

## Contracts

- `ZusProtocol.sol` manages campaigns, verifies claims, prevents nullifier reuse, and sends payouts
- `HonkVerifier.sol` verifies the UltraHonk proof generated from the Noir circuit

## Live Fuji Deployments

- Verifier: [0x2Ab7e6Bc7A69d0D37B43ea2f7374a12aC3f04CAB](https://testnet.snowtrace.io/address/0x2Ab7e6Bc7A69d0D37B43ea2f7374a12aC3f04CAB/contract/43113/code)
- ZusProtocol: [0x19b2d6A4D21078A215406eeF1F71731AEE84F7b4](https://testnet.snowtrace.io/address/0x19b2d6A4D21078A215406eeF1F71731AEE84F7b4/contract/43113/code)

## Notes

- Merkle paths stay offchain; only the root is committed onchain
- payouts are currently flat per campaign
- the shared verifier is reused across campaigns

See also:

- `verifier/README.md`
- `zusprotocol/README.md`
