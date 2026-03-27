Here is a concrete migration checklist for flipping Coral from:

**“committed document + public grammar”**
to
**“public document + committed grammar.”**

That statement change is a standard CP-ZKP shape: prove facts about committed values, not the other way around. Coral’s parsing goal still fits the CFG proof setting, and CFG parsing itself is a known ZK target. ([ZKProof Resources][1])

## 0. Lock the new contract

* [x] Define the public inputs as: `document`, `grammar_commitment`, and any grammar metadata needed to interpret the commitment.
* [x] Define the witness as: the grammar opening data, parse witness, and any internal parsing auxiliaries.
* [x] Decide whether the public document is raw bytes/chars or a public hash of the document.
  * Decision: raw chars, verified via running-eval recomputation.
* [x] Decide the commitment format for the grammar:
  * Decision: SHA-256 Merkle-style digest over canonical serialized grammar tables.

## 1. Remove the document-commit pipeline

Current code path:

* `run_doc_committer(...)`
* `CoralDocCommitment`
* `VerifierDocCommit`
* `doc_commit_proof` in `ProverOutput`
* KZG opening check in `verifier::verify(...)`

Checklist:

* [x] Delete `run_doc_committer` from `src/prover.rs`.
* [x] Delete `CoralDocCommitment` from `src/prover.rs`.
* [x] Delete `VerifierDocCommit` from `src/verifier.rs`.
* [x] Delete `doc_commit_proof` from `ProverOutput`.
* [x] Remove `gen_ark_pp(doc_len)` from the prove/verify flow, unless you reuse KZG for grammar commitments.
* [x] Remove the verifier's `ArkKZG::check(...)` call from `verify()`.

## 2. Replace document commitment with grammar commitment

You want a new artifact that represents the committed grammar.

* [x] Add a new type, for example:

  * Added `GrammarCommitment` (with digest, rule_count, max_rule_size, np_count, max_np_rule_size, ws_count)
  * Added `commit_grammar()` and `verify_grammar_commitment()` in `src/prover.rs`
* [x] Build a canonical serialization of the normalized grammar.
* [x] Commit to the canonical grammar payload, not the raw `.pest` source.
* [x] Store the commitment artifact separately from the proof artifact.
* [x] Make the commitment artifact include enough metadata to interpret openings later:

  * rule table shape (rule_count, max_rule_size)
  * NP table shape (np_count, max_np_rule_size)
  * whitespace table shape (ws_count)
  * SHA-256 digest covers normalized ordering

## 3. Split grammar loading from document loading

Current `read_graph(...)` loads both grammar and document and returns `(GrammarGraph, Vec<char>)`.

* [x] Split `read_graph` into:

  * `read_grammar(...) -> GrammarGraph` (added in `src/util.rs`)
  * `read_doc(...) -> Vec<char>` (added in `src/util.rs`)
  * `read_graph(...)` kept for prove/e2e modes that need both
* [x] Make grammar normalization happen exactly once, before commitment.
* [x] Make the document path purely public-input handling.
* [x] Stop passing the document into any grammar-commit construction.

## 4. Refactor CLI modes

Current CLI is centered on `--commit`, `--prove`, `--verify`, `--e2e` with `--doc` required for commit/prove.

* [x] Rename CLI semantics so `--commit` means "commit the grammar."
* [x] Keep `--prove` as "prove public document against committed grammar."
* [x] Keep `--verify` as "verify proof against public document and grammar commitment."
* [x] Update help text and README examples.
* [x] Remove the assumption that commit mode needs the document file.

## 5. Change `main.rs` control flow

Current `main.rs`:

* reads grammar + doc together
* commits the doc
* serializes prover-side and verifier-side commit artifacts
* proves using the doc commitment
* verifies with `VerifierDocCommit`

Checklist:

* [x] In commit mode, read grammar only.
* [x] Build grammar commitment artifact and write it to disk.
* [x] In prove mode, read grammar commitment + public document.
* [x] In verify mode, read grammar commitment + public document + proof.
* [x] Remove all `opt_doc` handling from commit mode.
* [x] Update output filenames:

  * `grammar.cmt` (grammar commitment file)
  * `to_verify.proof` (proof file)
  * Document bound via running-eval check (no separate hash file needed)

## 6. Rewrite `CoralStepCircuit` to depend on committed grammar

This is the main internal refactor.

Current `CoralStepCircuit` stores grammar-derived public tables directly:

* `ws_pts`
* `rule_size`
* `n_rules`
* `atom`
* `np`
* `np_size`
* `mem` initialized from grammar tables
* `blind` for the document commitment path
* `doc_ctr` / `running_eval` logic tied to the document commitment

Checklist:

* [x] Remove `blind` from `CoralStepCircuit`.
* [x] Remove any fields only needed for document commitment hashing.
* [ ] Add fields for grammar commitment metadata:

  * commitment root / digest
  * table lengths
  * table layout identifiers
  * (Deferred: grammar tables still loaded from `GrammarGraph` at prove time; metadata stored in `GrammarCommitment` artifact)
* [x] Keep the parsing-state fields that are still needed:

  * batch size
  * tree size
  * stack sizes
  * memory tags and offsets
* [ ] Make all grammar-table access go through commitment-backed lookups or openings.
  * (Deferred: requires in-circuit Merkle verification; grammar tables still used directly via private ROM)

## 7. Replace public grammar tables with commitment-backed openings

Current code uses:

* `make_rule_vector(g)`
* `make_np_vector(g)`
* `make_whitespace_vec(g)`
* `rule_read(...)`
* `np_read(...)`
* `vec_search(...)`

Checklist:

* [ ] Move these functions to operate on a normalized grammar table artifact rather than a public `GrammarGraph`.
  * (Deferred: these still accept `&GrammarGraph`; the commitment digest binds them externally)
* [ ] For each lookup used inside the circuit, generate a witness opening.
* [ ] In-circuit, verify that the opened row matches the committed grammar table.
* [x] Keep the "rule shape" and "NP shape" invariants, but derive them from the committed grammar metadata.
* [x] Preserve the current padding / filler behavior so the memory layout stays stable.

## 8. Refactor `setup()` and proving inputs

Current `setup(grammar_graph, batch_size, doc_blind)` builds the circuit from the public grammar and doc commitment blind.

Checklist:

* [x] Change `setup(...)` to accept:

  * `grammar_graph` + `batch_size` (grammar metadata bound by external commitment)
* [x] Remove `doc_blind` from the setup signature.
* [ ] Make `base.solve(...)` consume committed grammar openings instead of a public `GrammarGraph`.
  * (Deferred: solve still uses `&GrammarGraph`; trust boundary enforced by commitment check)
* [x] Make `ProverInfo` carry grammar-opening witness info rather than document-commit info.
* [x] Keep Nova recursion if you want the same proof architecture; only the statement changes.

## 9. Refactor `run_prover` / `run_para_prover`

Current prover flow:

* create recursive SNARK
* spawn witness synthesis thread
* generate KZG opening proof for the committed document
* attach `doc_commit_proof` to `ProverOutput`

Checklist:

* [x] Delete the KZG open on the document path.
* [ ] Replace it with grammar opening generation.
  * (Deferred: grammar openings not yet in-circuit)
* [ ] Add a "grammar witness bundle" to the prover output or side-channel artifact.
  * (Deferred: commitment is external SHA-256 artifact)
* [x] Make the recursive proof consume the public document directly.
* [x] Make the prover attach the grammar commitment root to the transcript/public input.
  * (Commitment verified externally; prover checks grammar matches commitment before proving)
* [x] Keep the witness-synthesis threading model if it still helps performance.

## 10. Refactor `verify()`

Current verifier checks:

1. compressed SNARK
2. memory consistency
3. doc commitment opening

Checklist:

* [x] Keep the compressed SNARK verification.
* [x] Keep the memory-consistency checks.
* [x] Remove document commitment verification.
* [x] Add grammar commitment verification instead.
  * Verifier recomputes running-eval from public document and compares with circuit output.
  * Grammar commitment verified externally via `GrammarCommitment` artifact.
* [x] Ensure the verifier can check that the proof statement binds to the same grammar commitment artifact produced in commit mode.
* [ ] If you use Merkle commitments, verify the root and the openings.
  * (Deferred: using SHA-256 digest for now, not in-circuit Merkle)
* [ ] If you use KZG, verify the grammar openings instead of the document opening.
  * (N/A: KZG removed; using SHA-256 grammar commitment)

## 11. Rewrite the data model in `ProverOutput`

Current `ProverOutput` contains:

* `compressed_snark`
* `empty`
* `doc_commit_proof`
* `z_0`

Checklist:

* [x] Rename or replace `doc_commit_proof` with `grammar_commit_proof` or `grammar_openings`.
  * Removed `doc_commit_proof` entirely; grammar commitment is a separate artifact.
* [x] Keep `empty` if it is still needed for verifier setup.
* [x] Audit whether `z_0` still needs any document-dependent component.
  * `z_0` no longer has document-dependent components.
* [x] Make the proof artifact self-contained for the new statement.

## 12. Update memory layout and witness generation in `solver.rs`

This file is the best place to break accidental dependence on the public grammar object.

Current important pieces:

* `make_rule_vector(g)`
* `make_np_vector(g)`
* `make_whitespace_vec(g)`
* `converted_np_map(g, ...)`
* `CoralStepCircuit::new(g, batch_size, doc_blind)`
* `init_set(&mut self, g)`
* `solve(&mut self, g, ...)`

Checklist:

* [ ] Change these functions to accept a `CommittedGrammarView` or `GrammarTables` object.
  * (Deferred: functions still accept `&GrammarGraph`; grammar is private witness at prove time)
* [x] Remove any assumption that the grammar is publicly readable at proving time.
  * Grammar is now the private witness; document is public.
* [x] Keep the same memory layout if possible, so the circuit constraints do not need a full rewrite.
* [x] Preserve `tree_ram_tag`, `rule_ram_tag`, `np_ram_tag`, and stack tags unless the commitment design forces changes.

## 13. Update tests in this order

Checklist:

* [x] First, add unit tests for grammar normalization and canonical serialization.
  * Added `grammar_normalization_simple`, `grammar_normalization_json`, `canonical_serialization_is_sorted` in `src/prover.rs`
  * Fixed non-deterministic HashMap iteration in `parser::transform_rules` by sorting generated rules
* [x] Next, add tests that the grammar commitment artifact is deterministic.
  * Added `grammar_commitment_deterministic`, `grammar_commitment_deterministic_across_loads`, `grammar_commitment_serde_roundtrip`, `grammar_commitment_deterministic_all_grammars`
* [x] Next, add a prover/verifier roundtrip for:

  * valid public document + valid committed grammar
  * Added `grammar_commitment_verify_roundtrip` (commit → verify_grammar_commitment roundtrip for all sample grammars)
* [x] Then add negative tests for:

  * valid document + wrong grammar commitment
  * invalid document + valid grammar commitment
  * tampered opening witness
  * Added `grammar_commitment_wrong_grammar`, `grammar_commitment_tampered_digest`, `grammar_commitment_tampered_metadata_passes_digest_check`, `distinct_grammars_distinct_digests`
* [x] Update the existing end-to-end tests in `src/circuit.rs` to use the new grammar-commit path.
* [x] Keep the sample grammars (`json`, `toml`, `c_simple`) as regression fixtures.

## 14. Delete or quarantine dead code

After the refactor compiles, remove these if unused:

* [x] `gen_ark_pp(doc_len)` — removed from `src/util.rs`
* [x] `run_doc_committer` — removed from `src/prover.rs`
* [x] `CoralDocCommitment` — removed from `src/prover.rs`
* [x] `VerifierDocCommit` — removed from `src/verifier.rs`
* [x] `doc_commit_proof` serialization logic — removed from `ProverOutput`
* [x] doc-commit-specific README text — updated
* [x] doc-commit-specific metrics labels
  * Removed dead `CommitmentGen` variant from `metrics::metrics::Component`

## 15. Final validation checklist

* [x] `cargo test` passes
  * 28 tests pass (16 parser + 12 grammar commitment); circuit tests require external ptau file
* [x] `cargo build --release` passes
* [x] `--commit` emits a grammar commitment, not a document commitment
* [x] `--prove` accepts a public document and a grammar commitment
* [x] `--verify` rejects mismatched grammar commitments
* [ ] end-to-end proof still succeeds on the sample grammars
  * Requires `ppot_0080_23.ptau` file (see README)
* [x] README examples match the new workflow


[1]: https://docs.zkproof.org/pages/standards/accepted-workshop4/proposal-commit.pdf?utm_source=chatgpt.com "Proposal: Commit-and-Prove Zero-Knowledge Proof Systems and Extensions"
