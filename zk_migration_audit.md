# ZK Migration Audit: "public document + committed grammar"

Date: 2026-03-28

## Scope

This audit checks whether Coral currently enforces the intended statement:

> There exists a (private) grammar consistent with a **public grammar commitment** such that the **public document** parses under that grammar, with proof soundness/zero-knowledge coming from the SNARK.

## What is implemented

1. A deterministic SHA-256 grammar commitment exists over canonicalized grammar tables (`commit_grammar`).
2. Prove mode checks that a supplied grammar file matches the supplied commitment before proving.
3. Verify mode checks the SNARK plus memory checks and public-document running-eval consistency.

## What is not yet cryptographically bound in verification

1. The verifier function currently accepts `grammar_commit` but does not use it (`_grammar_commit`).
2. The proof object (`ProverOutput`) contains no grammar commitment digest or grammar-opening witness.
3. The migration checklist itself marks key commitment-backed lookup/opening tasks as deferred.

## Analytical conclusion

This repo currently provides **sound ZK proving for document-consistency against the private grammar used by the prover**, plus an **out-of-circuit grammar-commitment check in prove mode**.

However, it does **not yet fully realize commit-and-prove binding at verifier time** for "public document + committed grammar" in the strong cryptographic sense, because the verifier is not checking that the proof is tied to a specific commitment digest.

So:
- ZK computation is present for the parse relation itself.
- Full CP-ZKP binding to a public grammar commitment is **partially implemented**, not complete.

## Minimal acceptance criterion for "done"

To consider the migration complete, verifier-time checks should include at least one of:

- commitment digest included in SNARK public IO (`z_0` or equivalent) and explicitly checked against the supplied commitment; and/or
- in-circuit opening verification (Merkle or KZG) for grammar table rows used by lookups.

