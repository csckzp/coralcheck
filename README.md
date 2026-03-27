
# Coral

This is an implementation of Coral, a system for generating zero-knowledge proofs that a public document is consistent with a committed Context Free Grammar.
The details of Coral are described in our paper: [Coral: Fast Succinct Non-Interactive Zero-Knowledge CFG Proofs](https://eprint.iacr.org/2025/1420).

## Compile

```
cargo build --release
```

With metrics:
```
cargo build --release --features metrics
```

With pipelined proving and witness generation:
```
cargo build --release --features para
```


## Usage
```
Usage: coral [OPTIONS] <--commit|--prove|--verify|--e2e>

Options:
      --commit              Commit the grammar
      --prove               Prove public document against committed grammar
      --verify              Verify proof against public document and grammar commitment
      --e2e                 End-to-end: commit, prove, and verify
      --cmt-name <FILE>     Optional name for grammar commitment file
      --proof-name <FILE>   Optional name for .proof file
  -d, --doc <FILE>          Public document file (required for prove/verify/e2e)
  -g, --grammar <FILE>      Grammar .pest file (required for commit/prove/e2e)
  -m, --metrics <FILE>      Metrics and other output information
  -b, --batch-size <USIZE>  Batch size [default: 1]
  -h, --help                Print help
  -V, --version             Print version
```

### Workflow

1. **Commit** the grammar (no document needed):
   ```
   ./target/release/coral --commit -g ./grammars/json.pest
   ```
   This writes `grammar.cmt` (or the name given via `--cmt-name`).

2. **Prove** that a public document matches the committed grammar:
   ```
   ./target/release/coral --prove -g ./grammars/json.pest -d ./tests/test_docs/json/test_json_64.txt
   ```
   This reads `grammar.cmt` and writes `to_verify.proof`.

3. **Verify** the proof against the public document and grammar commitment:
   ```
   ./target/release/coral --verify -d ./tests/test_docs/json/test_json_64.txt
   ```
   This reads `grammar.cmt` and `to_verify.proof`.

Or do everything at once:
```
./target/release/coral --e2e -g ./grammars/json.pest -d ./tests/test_docs/json/test_json_64.txt -b 100
```

Coral has the ability to process multiple nodes in the parse tree per folding, this is controlled by the `--batch-size` parameter. A larger batch size will require fewer total proving steps, but each step will have more constraints. In our experience, between 5 and 10 total steps is usually optimal. Depending on your tree this will be a batch size between 150 and 1,000. Performance will significantly degrade as the batch size increases beyond 2,500.

You can use `--cmt-name` and `--proof-name` to choose names for your
commitment and proof files. This is optional - Coral will choose a name for the
commitment/proof if you do not. 

## Perpetual Powers of Tau 
You will need a local copy of the [Perpetual Powers of Tau](https://github.com/privacy-scaling-explorations/perpetualpowersoftau) to run Coral. Coral is hardcoded to use **./ppot_0080_23.ptau**. However, you can use whichever one you prefer by changing the specified file in `src/solver.rs` and `src/util.rs`.

## Sample Grammars
The grammars directory contains sample grammars for JSON, TOML, and a subset of C. You can run Coral for JSON with the following:
```
./target/release/coral --e2e -d ./tests/test_docs/json/test_json_64.txt -g ./grammars/json.pest -b 100 -m ./tests/results/timings/scale/json_64_coral.txt
```

## Reproducing Baseline Results
If you're interested in reproducing our baseline results, you can run the corresponding scripts in the **tests/scripts** directory. We have also provided a python notebook **DataCleaning** to help reproduce our analysis. 

Thank you for using Coral,
Happy proving!
