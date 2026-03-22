# Zus Protocol MVP

This package wraps the shared verifier with actual payout logic.

The verifier only answers `true` or `false`, but the proof already carries:

- the expected `eligible_root`
- the nullifier point bytes
- the derived `stealth_address`

So the protocol contract does this:

1. decode the public inputs
2. check the fixed message and expected root
3. call the shared verifier
4. mark the nullifier as spent
5. pay the decoded stealth address

## Current shape

This contract is intentionally one drop per deployment:

- one `eligible_root`
- one `expectedMessage`
- one fixed payout amount

That matches the current MVP TUI flow, which is also using a fixed Noir message.

## Important MVP constraint

The current Noir circuit does **not** bind an amount into the proof.

Because of that, [ZusProtocol.sol](./src/ZusProtocol.sol) pays a fixed native-token amount per valid claim.

If you want variable amounts per recipient later, the circuit needs to bind the leaf payload, not just address membership.

If you want one protocol contract to support many campaigns later, make the proof domain campaign-specific too.
Right now the nullifier uses `message || compressed_pubkey`, so reusing the same message across many campaigns would couple their nullifier space together.

## Run tests

```bash
cd zusprotocol
forge test
```
