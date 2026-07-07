# Getting Started: External Verification for Curtailment Bodies

If you have built (or are building) a COSA-style workload scheduler that can curtail compute load, you can prove it understands the EMILIA grid.curtailment profile using an independent verifier.

## 1. Implement your verifier
Use only the published vectors and the spec. No dependency on the main EMILIA packages/verify.

See the clean-room Node example.

## 2. Run against vectors
Produce `*.results.json` files (array of {id, valid}).

## 3. Sign with fresh key
Use the reference signer with a fresh key generated for independence.

## 4. Self-enroll your body
Use your local key to produce a signed enrollment.

## 5. Gate a receipt
When you receive a curtailment receipt:
- Verify it
- Check envelope / gated spending
- Produce signed refusal or posture ack + telemetry

See examples/cosa/INDEPENDENT_VERIFIER_TO_BODY.md for the full flow.

Share your statement + public key in the coalition. Independence is proven when a second key (yours) agrees on the vectors.
