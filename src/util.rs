use crate::{parser::GrammarGraph, prover::make_coral_circuit, solver::*};
use ark_ff::{BigInteger256, PrimeField};
use csv::Writer;
use nova_snark::{
    nova::PublicParams,
    provider::{Bn256EngineKZG, GrumpkinEngine},
    traits::{Engine, snark::default_ck_hint},
};
use segmented_circuit_memory::bellpepper::FCircuit;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::time::SystemTime;
use std::{fs, usize};

#[cfg(feature = "metrics")]
use metrics::metrics::{log, log::Component};

pub trait ArkPrimeField: PrimeField<BigInt = BigInteger256> {}

impl<F: PrimeField<BigInt = BigInteger256>> ArkPrimeField for F {}

pub type AF = ark_bn254::Fr;

pub type HashMap<K, V> = std::collections::HashMap<K, V>;
pub type HashSet<K> = std::collections::HashSet<K>;

pub fn new_hash_map<K, V>() -> HashMap<K, V> {
    HashMap::default()
}

pub type E1 = Bn256EngineKZG;
pub type E2 = GrumpkinEngine;
pub type EE1 = nova_snark::provider::hyperkzg::EvaluationEngine<E1>;
pub type EE2 = nova_snark::provider::ipa_pc::EvaluationEngine<E2>;
pub type S1 = nova_snark::spartan::snark::RelaxedR1CSSNARK<E1, EE1>;
pub type S2 = nova_snark::spartan::snark::RelaxedR1CSSNARK<E2, EE2>;
pub type C1 = FCircuit<<E1 as Engine>::Scalar>;
pub type N1 = <E1 as Engine>::Scalar;
pub type N2 = <E2 as Engine>::Scalar;

pub fn get_commit_name(opt_name: Option<String>) -> String {
    match opt_name {
        Some(name) => name,
        None => "grammar.cmt".to_string(),
    }
}

pub fn get_proof_name(opt_name: Option<String>) -> String {
    match opt_name {
        Some(name) => name,
        None => "to_verify.proof".to_string(),
    }
}

/// Read and compile a grammar file without a document.
/// Produces a `GrammarGraph` with populated rule tables suitable for
/// grammar commitment (but no LCRS parse tree).
pub fn read_grammar(pest_file: String) -> GrammarGraph {
    let grammar = fs::read_to_string(pest_file).expect("Failed to read grammar file");
    let mut grammar_graph = GrammarGraph::new();
    grammar_graph
        .compile_grammar(&grammar)
        .expect("Failed to compile grammar");
    grammar_graph
}

/// Read a document file and return its characters.
pub fn read_doc(input: String) -> Vec<char> {
    let input_text = fs::read_to_string(input).expect("Failed to read input file");
    input_text.chars().collect()
}

/// Read grammar + document, parse the document, and build the full
/// LCRS parse tree needed for proving.
pub fn read_graph(pest_file: String, input: String) -> (GrammarGraph, Vec<char>) {
    let grammar = fs::read_to_string(pest_file).expect("Failed to read grammar file");
    let input_text = fs::read_to_string(input).expect("Failed to read input file");

    let mut grammar_graph = GrammarGraph::new();
    grammar_graph
        .parse_text_and_build_graph(&grammar, &input_text)
        .expect("Failed to parse input");

    grammar_graph.parse_and_convert_lcrs();
    (grammar_graph, input_text.chars().collect())
}

pub fn gen_pp<AF: ArkPrimeField>(empty_csc: &mut CoralStepCircuit<AF>) -> PublicParams<E1, E2, C1> {
    let mut irw = InterRoundWires::new();

    let mut circuit_primary = make_coral_circuit(empty_csc, &mut irw, 0, None);

    let pp = PublicParams::<E1, E2, C1>::setup(
        &mut circuit_primary,
        &*default_ck_hint(),
        &*default_ck_hint(),
        empty_csc.key_lengths.clone(),
        Some("./ppot_0080_23.ptau"),
    )
    .unwrap();
    pp
}

pub fn metrics_file(
    metrics: Option<PathBuf>,
    grammar: &String,
    doc: &String,
    doc_len: usize,
    tree_size: usize,
    batch_size: usize,
    grammar_len: usize,
) {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(metrics.clone().unwrap())
        .unwrap();
    let mut wtr = Writer::from_writer(file);
    let _ = wtr.write_record(&[
        doc.to_string(),
        grammar.to_string(),
        time,
        grammar_len.to_string(),
        tree_size.to_string(),
        doc_len.to_string(),
        batch_size.to_string(),
    ]);
    let spacer = "---------";
    let _ = wtr.write_record([spacer, spacer, spacer, spacer, "\n"]);
    let _ = wtr.flush();
    #[cfg(feature = "metrics")]
    log::write_csv(metrics.unwrap().to_str().unwrap()).unwrap();
}
