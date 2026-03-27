Here is a concrete migration checklist for flipping Coral from:

**“committed document + public grammar”**
to
**“public document + committed grammar.”**

That statement change is a standard CP-ZKP shape: prove facts about committed values, not the other way around. Coral’s parsing goal still fits the CFG proof setting, and CFG parsing itself is a known ZK target. ([ZKProof Resources][1])

## 0. Lock the new contract

* [ ] Define the public inputs as: `document`, `grammar_commitment`, and any grammar metadata needed to interpret the commitment.
* [ ] Define the witness as: the grammar opening data, parse witness, and any internal parsing auxiliaries.
* [ ] Decide whether the public document is raw bytes/chars or a public hash of the document.
* [ ] Decide the commitment format for the grammar:

  * easiest: Merkle commitment to normalized grammar tables
  * smaller proofs: polynomial commitment / KZG-style openings

## 1. Remove the document-commit pipeline

Current code path:

* `run_doc_committer(...)`
* `CoralDocCommitment`
* `VerifierDocCommit`
* `doc_commit_proof` in `ProverOutput`
* KZG opening check in `verifier::verify(...)`

Checklist:

* [ ] Delete `run_doc_committer` from `src/prover.rs`.
* [ ] Delete `CoralDocCommitment` from `src/prover.rs`.
* [ ] Delete `VerifierDocCommit` from `src/verifier.rs`.
* [ ] Delete `doc_commit_proof` from `ProverOutput`.
* [ ] Remove `gen_ark_pp(doc_len)` from the prove/verify flow, unless you reuse KZG for grammar commitments.
* [ ] Remove the verifier’s `ArkKZG::check(...)` call from `verify()`.

## 2. Replace document commitment with grammar commitment

You want a new artifact that represents the committed grammar.

* [ ] Add a new type, for example:

  * `CoralGrammarCommitment`
  * `VerifierGrammarCommit`
  * `GrammarCommitWitness`
* [ ] Build a canonical serialization of the normalized grammar.
* [ ] Commit to the canonical grammar payload, not the raw `.pest` source.
* [ ] Store the commitment artifact separately from the proof artifact.
* [ ] Make the commitment artifact include enough metadata to interpret openings later:

  * rule table shape
  * NP table shape
  * whitespace table shape
  * rule ordering / normalization version

## 3. Split grammar loading from document loading

Current `read_graph(...)` loads both grammar and document and returns `(GrammarGraph, Vec<char>)`.

* [ ] Split `read_graph` into:

  * `read_grammar(...) -> GrammarGraph + normalized tables`
  * `read_doc(...) -> Vec<char>`
* [ ] Make grammar normalization happen exactly once, before commitment.
* [ ] Make the document path purely public-input handling.
* [ ] Stop passing the document into any grammar-commit construction.

## 4. Refactor CLI modes

Current CLI is centered on `--commit`, `--prove`, `--verify`, `--e2e` with `--doc` required for commit/prove.

* [ ] Rename CLI semantics so `--commit` means “commit the grammar.”
* [ ] Keep `--prove` as “prove public document against committed grammar.”
* [ ] Keep `--verify` as “verify proof against public document and grammar commitment.”
* [ ] Update help text and README examples.
* [ ] Remove the assumption that commit mode needs the document file.

## 5. Change `main.rs` control flow

Current `main.rs`:

* reads grammar + doc together
* commits the doc
* serializes prover-side and verifier-side commit artifacts
* proves using the doc commitment
* verifies with `VerifierDocCommit`

Checklist:

* [ ] In commit mode, read grammar only.
* [ ] Build grammar commitment artifact and write it to disk.
* [ ] In prove mode, read grammar commitment + public document.
* [ ] In verify mode, read grammar commitment + public document + proof.
* [ ] Remove all `opt_doc` handling from commit mode.
* [ ] Update output filenames:

  * grammar commitment file
  * proof file
  * optional public document hash file if you choose to bind the document that way

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

* [ ] Remove `blind` from `CoralStepCircuit`.
* [ ] Remove any fields only needed for document commitment hashing.
* [ ] Add fields for grammar commitment metadata:

  * commitment root / digest
  * table lengths
  * table layout identifiers
* [ ] Keep the parsing-state fields that are still needed:

  * batch size
  * tree size
  * stack sizes
  * memory tags and offsets
* [ ] Make all grammar-table access go through commitment-backed lookups or openings.

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
* [ ] For each lookup used inside the circuit, generate a witness opening.
* [ ] In-circuit, verify that the opened row matches the committed grammar table.
* [ ] Keep the “rule shape” and “NP shape” invariants, but derive them from the committed grammar metadata.
* [ ] Preserve the current padding / filler behavior so the memory layout stays stable.

## 8. Refactor `setup()` and proving inputs

Current `setup(grammar_graph, batch_size, doc_blind)` builds the circuit from the public grammar and doc commitment blind.

Checklist:

* [ ] Change `setup(...)` to accept:

  * committed grammar metadata
  * batch size
  * public document binding
* [ ] Remove `doc_blind` from the setup signature.
* [ ] Make `base.solve(...)` consume committed grammar openings instead of a public `GrammarGraph`.
* [ ] Make `ProverInfo` carry grammar-opening witness info rather than document-commit info.
* [ ] Keep Nova recursion if you want the same proof architecture; only the statement changes.

## 9. Refactor `run_prover` / `run_para_prover`

Current prover flow:

* create recursive SNARK
* spawn witness synthesis thread
* generate KZG opening proof for the committed document
* attach `doc_commit_proof` to `ProverOutput`

Checklist:

* [ ] Delete the KZG open on the document path.
* [ ] Replace it with grammar opening generation.
* [ ] Add a “grammar witness bundle” to the prover output or side-channel artifact.
* [ ] Make the recursive proof consume the public document directly.
* [ ] Make the prover attach the grammar commitment root to the transcript/public input.
* [ ] Keep the witness-synthesis threading model if it still helps performance.

## 10. Refactor `verify()`

Current verifier checks:

1. compressed SNARK
2. memory consistency
3. doc commitment opening

Checklist:

* [ ] Keep the compressed SNARK verification.
* [ ] Keep the memory-consistency checks.
* [ ] Remove document commitment verification.
* [ ] Add grammar commitment verification instead.
* [ ] Ensure the verifier can check that the proof statement binds to the same grammar commitment artifact produced in commit mode.
* [ ] If you use Merkle commitments, verify the root and the openings.
* [ ] If you use KZG, verify the grammar openings instead of the document opening.

## 11. Rewrite the data model in `ProverOutput`

Current `ProverOutput` contains:

* `compressed_snark`
* `empty`
* `doc_commit_proof`
* `z_0`

Checklist:

* [ ] Rename or replace `doc_commit_proof` with `grammar_commit_proof` or `grammar_openings`.
* [ ] Keep `empty` if it is still needed for verifier setup.
* [ ] Audit whether `z_0` still needs any document-dependent component.
* [ ] Make the proof artifact self-contained for the new statement.

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
* [ ] Remove any assumption that the grammar is publicly readable at proving time.
* [ ] Keep the same memory layout if possible, so the circuit constraints do not need a full rewrite.
* [ ] Preserve `tree_ram_tag`, `rule_ram_tag`, `np_ram_tag`, and stack tags unless the commitment design forces changes.

## 13. Update tests in this order

Checklist:

* [ ] First, add unit tests for grammar normalization and canonical serialization.
* [ ] Next, add tests that the grammar commitment artifact is deterministic.
* [ ] Next, add a prover/verifier roundtrip for:

  * valid public document + valid committed grammar
* [ ] Then add negative tests for:

  * valid document + wrong grammar commitment
  * invalid document + valid grammar commitment
  * tampered opening witness
* [ ] Update the existing end-to-end tests in `src/circuit.rs` to use the new grammar-commit path.
* [ ] Keep the sample grammars (`json`, `toml`, `c_simple`) as regression fixtures.

## 14. Delete or quarantine dead code

After the refactor compiles, remove these if unused:

* [ ] `gen_ark_pp(doc_len)`
* [ ] `run_doc_committer`
* [ ] `CoralDocCommitment`
* [ ] `VerifierDocCommit`
* [ ] `doc_commit_proof` serialization logic
* [ ] doc-commit-specific README text
* [ ] doc-commit-specific metrics labels

## 15. Final validation checklist

* [ ] `cargo test` passes
* [ ] `cargo build --release` passes
* [ ] `--commit` emits a grammar commitment, not a document commitment
* [ ] `--prove` accepts a public document and a grammar commitment
* [ ] `--verify` rejects mismatched grammar commitments
* [ ] end-to-end proof still succeeds on the sample grammars
* [ ] README examples match the new workflow


[1]: https://docs.zkproof.org/pages/standards/accepted-workshop4/proposal-commit.pdf?utm_source=chatgpt.com "Proposal: Commit-and-Prove Zero-Knowledge Proof Systems and Extensions"
