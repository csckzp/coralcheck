#![allow(missing_docs, non_snake_case)]

mod parser;
use anyhow::Result;
use clap::Parser;
use coral::{
    config::*,
    prover::{self, *},
    util::*,
    verifier::{self, verify},
};
use std::fs;

#[cfg(feature = "metrics")]
use metrics::metrics::{log, log::Component};
fn main() -> Result<()> {
    let opt = Options::parse();

    let grammar_path = opt.grammar.clone();
    let input_text_path = opt.doc.clone();
    let batch_size = opt.batch_size;

    // ------------------------------------------------------------------
    // Commit mode: grammar only → grammar commitment artifact
    // ------------------------------------------------------------------
    if opt.commit {
        let grammar_path = grammar_path
            .as_ref()
            .expect("Grammar file (-g) is required for --commit");

        let grammar_graph = read_grammar(grammar_path.clone());

        #[cfg(feature = "metrics")]
        log::tic(Component::Generator, "grammar_commit");

        let grammar_commit = commit_grammar(&grammar_graph);

        #[cfg(feature = "metrics")]
        log::stop(Component::Generator, "grammar_commit");

        let commit_bytes = bincode::serialize(&grammar_commit).unwrap();
        fs::write(get_commit_name(opt.cmt_name.clone()), commit_bytes)
            .expect("Unable to write grammar commitment file");

        println!(
            "Grammar commitment written to {}",
            get_commit_name(opt.cmt_name.clone())
        );
    }

    // ------------------------------------------------------------------
    // E2E mode: commit + prove + verify in one shot
    // ------------------------------------------------------------------
    if opt.e2e {
        let grammar_path = grammar_path
            .as_ref()
            .expect("Grammar file (-g) is required for --e2e");
        let doc_path = input_text_path
            .as_ref()
            .expect("Document file (-d) is required for --e2e");

        // 1. Build grammar commitment
        let (grammar_graph, doc) = read_graph(grammar_path.clone(), doc_path.clone());

        #[cfg(feature = "metrics")]
        log::tic(Component::Generator, "grammar_commit");

        let grammar_commit = commit_grammar(&grammar_graph);

        #[cfg(feature = "metrics")]
        log::stop(Component::Generator, "grammar_commit");

        let commit_bytes = bincode::serialize(&grammar_commit).unwrap();
        fs::write(get_commit_name(opt.cmt_name.clone()), commit_bytes)
            .expect("Unable to write grammar commitment file");

        // 2. Prove
        #[allow(unused_mut)]
        let (mut p_i, mut base, mut empty, pp) =
            prover::setup(&grammar_graph, batch_size).unwrap();

        #[cfg(feature = "para")]
        let prover_output_res =
            run_para_prover::<AF>(&grammar_graph, base, &mut p_i, &pp);

        #[cfg(not(feature = "para"))]
        let prover_output_res =
            run_prover::<AF>(&grammar_graph, &mut base, &mut p_i, &pp);

        assert!(prover_output_res.is_ok());

        let mut prover_output = prover_output_res.unwrap();
        prover_output.empty = Some(empty);

        let prover_output_data = bincode::serialize(&prover_output).unwrap();
        fs::write(
            get_proof_name(opt.proof_name.clone()),
            prover_output_data,
        )
        .expect("Unable to write proof file");

        // 3. Verify
        let data_from_prover =
            fs::read(get_proof_name(opt.proof_name.clone())).expect("Unable to read proof file");
        let mut prover_output_v =
            bincode::deserialize::<ProverOutput>(&data_from_prover).unwrap();

        let mut empty_v = prover_output_v.empty.take().unwrap();
        let v_i = verifier::setup(&mut empty_v);

        let commit_data =
            fs::read(get_commit_name(opt.cmt_name.clone())).expect("Unable to read commitment file");
        let grammar_commit_v: GrammarCommitment =
            bincode::deserialize(&commit_data).unwrap();

        let verifier_output = verify(&mut prover_output_v, v_i, &doc, &grammar_commit_v);
        assert!(verifier_output.is_ok());

        #[cfg(feature = "metrics")]
        metrics_file(
            opt.metrics.clone(),
            grammar_path,
            doc_path,
            doc.len(),
            grammar_graph.lcrs_tree.node_count(),
            opt.batch_size,
            grammar_graph.rule_count,
        );
    }

    // ------------------------------------------------------------------
    // Prove mode: grammar commitment + grammar + document → proof
    // ------------------------------------------------------------------
    if opt.prove && !opt.e2e {
        let grammar_path = grammar_path
            .as_ref()
            .expect("Grammar file (-g) is required for --prove");
        let doc_path = input_text_path
            .as_ref()
            .expect("Document file (-d) is required for --prove");

        let (grammar_graph, _doc) = read_graph(grammar_path.clone(), doc_path.clone());

        // Optionally verify the grammar matches its commitment
        let commit_data =
            fs::read(get_commit_name(opt.cmt_name.clone())).expect("Unable to read commitment file");
        let grammar_commit: GrammarCommitment =
            bincode::deserialize(&commit_data).unwrap();
        assert!(
            verify_grammar_commitment(&grammar_graph, &grammar_commit),
            "Grammar does not match commitment"
        );

        #[allow(unused_mut)]
        let (mut p_i, mut base, mut empty, pp) =
            prover::setup(&grammar_graph, batch_size).unwrap();

        #[cfg(feature = "para")]
        let prover_output_res =
            run_para_prover::<AF>(&grammar_graph, base, &mut p_i, &pp);

        #[cfg(not(feature = "para"))]
        let prover_output_res =
            run_prover::<AF>(&grammar_graph, &mut base, &mut p_i, &pp);

        assert!(prover_output_res.is_ok());

        let mut prover_output = prover_output_res.unwrap();
        prover_output.empty = Some(empty);

        let prover_output_data = bincode::serialize(&prover_output).unwrap();
        fs::write(
            get_proof_name(opt.proof_name.clone()),
            prover_output_data,
        )
        .expect("Unable to write proof file");
    }

    // ------------------------------------------------------------------
    // Verify mode: grammar commitment + document + proof → result
    // ------------------------------------------------------------------
    if opt.verify && !opt.e2e {
        let doc_path = input_text_path
            .as_ref()
            .expect("Document file (-d) is required for --verify");

        let doc = read_doc(doc_path.clone());

        let data_from_prover =
            fs::read(get_proof_name(opt.proof_name.clone())).expect("Unable to read proof file");
        let mut prover_output =
            bincode::deserialize::<ProverOutput>(&data_from_prover).unwrap();

        let mut empty = prover_output.empty.take().unwrap();
        let v_i = verifier::setup(&mut empty);

        let commit_data =
            fs::read(get_commit_name(opt.cmt_name.clone())).expect("Unable to read commitment file");
        let grammar_commit: GrammarCommitment =
            bincode::deserialize(&commit_data).unwrap();

        let verifier_output = verify(&mut prover_output, v_i, &doc, &grammar_commit);
        assert!(verifier_output.is_ok());
    }

    Ok(())
}
