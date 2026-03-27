use crate::{circuit::multi_node_step, parser::GrammarGraph, solver::*, util::*};
use ark_relations::gr1cs::{ConstraintSystem, OptimizationGoal, SynthesisError, SynthesisMode};
use ark_serialize::CanonicalSerialize;

use ark_serialize::CompressedChecked;
use nova_snark::{
    errors::NovaError,
    frontend::LinearCombination,
    nova::{CompressedSNARK, ProverKey, PublicParams, RandomLayer, RecursiveSNARK},
    traits::ROConstants,
};
use segmented_circuit_memory::bellpepper::FCircuit;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::usize;
use std::{
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};

#[cfg(feature = "metrics")]
use metrics::metrics::{log, log::Component};

pub struct ProverInfo {
    pub ic_key_lengths: Vec<usize>,
    pub ic_blinds: Vec<Vec<N1>>,
    pub ic_hints: Vec<Vec<N1>>,
    pub snark_pk: ProverKey<E1, E2, C1, S1, S2>,
    pub random_layer: RandomLayer<E1, E2>,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize)]
pub struct ProverOutput {
    pub compressed_snark: CompressedSNARK<E1, E2, C1, S1, S2>,
    #[serde_as(as = "CompressedChecked<Option<CoralStepCircuit<AF>>>")]
    pub empty: Option<CoralStepCircuit<AF>>,
    pub z_0: Vec<N1>,
}

/// Commitment to a normalized grammar. Produced in commit mode and
/// consumed by prove/verify modes.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GrammarCommitment {
    /// SHA-256 digest of the canonical grammar tables.
    pub digest: [u8; 32],
    /// Number of unique rules (including the ANY sentinel).
    pub rule_count: usize,
    /// Maximum number of symbols in a single rule production.
    pub max_rule_size: usize,
    /// Number of negative-predicate rules.
    pub np_count: usize,
    /// Maximum size of a negative-predicate polynomial.
    pub max_np_rule_size: usize,
    /// Number of whitespace entries.
    pub ws_count: usize,
}

/// Build a canonical SHA-256 commitment over the normalized grammar tables.
pub fn commit_grammar(g: &GrammarGraph) -> GrammarCommitment {
    let rule_vec = make_rule_vector::<AF>(g);
    let np_vec = make_np_vector::<AF>(g);
    let ws_vec = make_whitespace_vec::<AF>(g);

    let mut hasher = Sha256::new();
    for row in &rule_vec {
        for val in row {
            let mut buf = Vec::new();
            val.serialize_compressed(&mut buf).unwrap();
            hasher.update(&buf);
        }
    }
    for row in &np_vec {
        for val in row {
            let mut buf = Vec::new();
            val.serialize_compressed(&mut buf).unwrap();
            hasher.update(&buf);
        }
    }
    for val in &ws_vec {
        let mut buf = Vec::new();
        val.serialize_compressed(&mut buf).unwrap();
        hasher.update(&buf);
    }
    let digest: [u8; 32] = hasher.finalize().into();

    GrammarCommitment {
        digest,
        rule_count: g.rule_count,
        max_rule_size: g.max_rule_size,
        np_count: g.np.len(),
        max_np_rule_size: g.max_np_rule_size,
        ws_count: ws_vec.len(),
    }
}

/// Returns `true` when the grammar produces the same commitment.
pub fn verify_grammar_commitment(g: &GrammarGraph, commitment: &GrammarCommitment) -> bool {
    let computed = commit_grammar(g);
    computed.digest == commitment.digest
}

pub fn setup<ArkF: ArkPrimeField>(
    grammar_graph: &GrammarGraph,
    batch_size: usize,
) -> Result<
    (
        ProverInfo,
        CoralStepCircuit<ArkF>,
        CoralStepCircuit<ArkF>,
        PublicParams<E1, E2, C1>,
    ),
    SynthesisError,
> {
    let mut base = CoralStepCircuit::new(grammar_graph, batch_size);

    let r0_consts = ROConstants::<E1>::default();

    let (ic_blinds, ram_hints, mut empty) = base.solve(grammar_graph, r0_consts)?;

    #[cfg(feature = "metrics")]
    log::tic(Component::Generator, "nova_pp_gen_p");
    let pp = gen_pp(&mut empty);
    #[cfg(feature = "metrics")]
    log::stop(Component::Generator, "nova_pp_gen_p");

    #[cfg(feature = "metrics")]
    log::tic(Component::Prover, "sample_random_layer");

    let random_layer = CompressedSNARK::<_, _, _, S1, S2>::sample_random_layer(&pp).unwrap();

    #[cfg(feature = "metrics")]
    log::stop(Component::Prover, "sample_random_layer");

    #[cfg(feature = "metrics")]
    log::tic(Component::Prover, "snark_params_p");
    let (pk, _) = CompressedSNARK::<_, _, _, S1, S2>::setup(&pp).unwrap();

    #[cfg(feature = "metrics")]
    log::stop(Component::Prover, "snark_params_p");

    let p_i = ProverInfo {
        ic_key_lengths: base.key_lengths.clone(),
        ic_blinds,
        ic_hints: ram_hints,
        snark_pk: pk,
        random_layer,
    };

    Ok((p_i, base, empty, pp))
}

type Constraint<F> = (
    LinearCombination<F>,
    LinearCombination<F>,
    LinearCombination<F>,
);

pub fn make_coral_circuit<ArkF: ArkPrimeField>(
    csc: &mut CoralStepCircuit<ArkF>,
    irw: &mut InterRoundWires<ArkF>,
    i: usize,
    saved_matrix: Option<Arc<Vec<Constraint<N1>>>>,
) -> FCircuit<N1> {
    #[cfg(feature = "metrics")]
    {
        log::tic(Component::Solver, format!("witness_synthesis_{}", i));
        // log::tic(Component::Solver, format!("witness_synthesis_ark_{}", i));
    }

    let cs = ConstraintSystem::<ArkF>::new_ref();
    cs.set_optimization_goal(OptimizationGoal::Constraints);

    if i != 0 {
        cs.set_mode(SynthesisMode::Prove {
            construct_matrices: false,
            generate_lc_assignments: false,
        });
        let num_constraints = saved_matrix.as_ref().unwrap().len();
        cs.borrow_mut()
            .unwrap()
            .assignments
            .witness_assignment
            .reserve(num_constraints * 2);
    }

    let mut wires = CoralWires::wires_from_irw(irw, cs.clone(), csc, i);

    let mut memory = csc
        .mem
        .as_mut()
        .unwrap()
        .begin_new_circuit(cs.clone())
        .unwrap();

    let wires_res = multi_node_step(csc, &mut wires, &mut memory, cs.clone());

    assert!(wires_res.is_ok(), "Wires failed at {:?}", i);

    irw.update(wires_res.unwrap());

    // #[cfg(feature = "metrics")]
    // {
    //     log::stop(Component::Solver, format!("witness_synthesis_ark_{}", i));
    // }

    let f = FCircuit::<N1>::new(cs.clone(), saved_matrix);

    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Solver, format!("witness_synthesis_{}", i));
    }

    f
}

pub fn run_wit_synth<'a>(
    sender: Sender<Option<FCircuit<N1>>>,
    saved_nova_matrices: Arc<Vec<Constraint<N1>>>,
    base: CoralStepCircuit<AF>,
    irw: InterRoundWires<AF>,
    n_rounds: usize,
) {
    println!("Solving thread starting...");
    let mut base = base;
    let mut irw = irw;

    for i in 0..n_rounds {
        if i + 1 < n_rounds {
            let circuit_primary = make_coral_circuit(
                &mut base,
                &mut irw,
                i + 1,
                Some(saved_nova_matrices.clone()),
            );
            sender.send(Some(circuit_primary)).unwrap();
        }
    }
}

pub fn run_prove(
    recv: Receiver<Option<FCircuit<N1>>>,
    recursive_snark: &mut RecursiveSNARK<E1, E2, C1>,
    p_i: &mut ProverInfo,
    pp: &PublicParams<E1, E2, C1>,
    circuit_primary: FCircuit<N1>,
    z0_primary: Vec<N1>,
    n_rounds: usize,
) -> Result<ProverOutput, NovaError> {
    let mut circuit_primary = circuit_primary;

    #[cfg(feature = "metrics")]
    log::tic(Component::Prover, "folding_proof");

    for i in 0..n_rounds {
        println!("Proving round {:?}", i);
        #[cfg(feature = "metrics")]
        log::tic(Component::Prover, format!("prove_{i}"));

        let res = recursive_snark.prove_step(
            pp,
            &mut circuit_primary,
            Some(p_i.ic_blinds[i].clone()),
            p_i.ic_hints[i].clone(),
            p_i.ic_key_lengths.clone(),
        );
        assert!(res.is_ok());

        #[cfg(feature = "metrics")]
        log::stop(Component::Prover, format!("prove_{i}"));

        if i + 1 < n_rounds {
            circuit_primary = recv.recv().unwrap().unwrap();
        }
    }

    // produce a compressed SNARK
    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Prover, "folding_proof");
        log::tic(Component::Prover, "compressed_snark");
    }

    println!("Compressed");
    let compressed_snark = CompressedSNARK::<_, _, _, S1, S2>::prove(
        pp,
        &p_i.snark_pk,
        recursive_snark,
        p_i.random_layer.clone(),
    );
    assert!(compressed_snark.is_ok());

    #[cfg(feature = "metrics")]
    log::stop(Component::Prover, "compressed_snark");

    Ok(ProverOutput {
        compressed_snark: compressed_snark.unwrap(),
        z_0: z0_primary,
        empty: None,
    })
}

pub fn run_para_prover<ArkF: ArkPrimeField>(
    grammar_graph: &GrammarGraph,
    base: CoralStepCircuit<AF>,
    p_i: &mut ProverInfo,
    pp: &PublicParams<E1, E2, C1>,
) -> Result<ProverOutput, NovaError> {
    let n_rounds = u32::div_ceil(
        grammar_graph.lcrs_tree.node_count() as u32,
        base.batch_size as u32,
    ) as usize;

    let mut base = base;

    let mut irw = InterRoundWires::new();

    let (sender_main, recv_main) = mpsc::channel();

    #[cfg(feature = "metrics")]
    log::tic(Component::Prover, "constraint_gen");

    let mut circuit_primary = make_coral_circuit(&mut base, &mut irw, 0, None);

    let z0_primary_full = circuit_primary.get_zi();
    let z0_offset = p_i.ic_key_lengths.iter().sum();
    let z0_primary = z0_primary_full[z0_offset..].to_vec();

    // produce a recursive SNARK
    let mut recursive_snark = RecursiveSNARK::<E1, E2, C1>::new(
        pp,
        &mut circuit_primary,
        &z0_primary,
        Some(p_i.ic_blinds[0].clone()),
        p_i.ic_hints[0].clone(),
        p_i.ic_key_lengths.clone(),
    )
    .unwrap();

    let saved_nova_matrices = circuit_primary.lcs.as_ref().right().unwrap().clone();

    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Prover, "constraint_gen");
        log::r1cs(Component::Prover, "Num Constraints", pp.num_constraints().0);
        log::tic(Component::Prover, "prove_e2e");
    }

    let now = Instant::now();

    let prover_output = thread::scope(|s| {
        s.spawn(move || {
            run_wit_synth(sender_main, saved_nova_matrices, base, irw, n_rounds);
        });
        let handle = s.spawn(move || {
            run_prove(
                recv_main,
                &mut recursive_snark,
                p_i,
                pp,
                circuit_primary,
                z0_primary,
                n_rounds,
            )
        });
        handle.join().expect("Proving thread panicked")
    })
    .unwrap();

    println!("Proving time: {:?}", now.elapsed());

    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Prover, "prove_e2e");
        log::space(
            Component::Prover,
            "compressed_snark",
            bincode::serialize(&prover_output.compressed_snark)
                .unwrap()
                .len(),
        );
    }

    Ok(prover_output)
}

pub fn run_prover<ArkF: ArkPrimeField>(
    grammar_graph: &GrammarGraph,
    base: &mut CoralStepCircuit<AF>,
    p_i: &mut ProverInfo,
    pp: &PublicParams<E1, E2, C1>,
) -> Result<ProverOutput, NovaError> {
    let n_rounds = u32::div_ceil(
        grammar_graph.lcrs_tree.node_count() as u32,
        base.batch_size as u32,
    ) as usize;

    println!("n rounds {:?}", n_rounds);

    //Actually prove things now
    let mut irw = InterRoundWires::new();

    let mut circuit_primary = make_coral_circuit(base, &mut irw, 0, None);

    #[cfg(feature = "metrics")]
    log::r1cs(Component::Prover, "Num Constraints", pp.num_constraints().0);

    let z0_primary_full = circuit_primary.get_zi().clone();
    let z0_offset = p_i.ic_key_lengths.iter().sum();
    let z0_primary = z0_primary_full[z0_offset..].to_vec();

    // produce a recursive SNARK
    let mut recursive_snark = RecursiveSNARK::<E1, E2, C1>::new(
        pp,
        &mut circuit_primary,
        &z0_primary,
        Some(p_i.ic_blinds[0].clone()),
        p_i.ic_hints[0].clone(),
        p_i.ic_key_lengths.clone(),
    )
    .unwrap();

    let saved_nova_matrices = circuit_primary.lcs.as_ref().right().unwrap().clone();

    #[cfg(feature = "metrics")]
    log::tic(Component::Prover, "prove_e2e");

    for i in 0..n_rounds {
        println!("Proving round {:?}", i);
        #[cfg(feature = "metrics")]
        log::tic(Component::Prover, format!("prove_{}", i));

        let res = recursive_snark.prove_step(
            pp,
            &mut circuit_primary,
            Some(p_i.ic_blinds[i].clone()),
            p_i.ic_hints[i].clone(),
            p_i.ic_key_lengths.clone(),
        );
        assert!(res.is_ok());

        #[cfg(feature = "metrics")]
        {
            log::stop(Component::Prover, format!("prove_{}", i));
        }

        if i + 1 < n_rounds {
            println!("gen round {:?}", i + 1);
            circuit_primary =
                make_coral_circuit(base, &mut irw, i + 1, Some(saved_nova_matrices.clone()));
        }
    }

    // produce a compressed SNARK
    #[cfg(feature = "metrics")]
    log::tic(Component::Prover, "compressed_snark");

    let compressed_snark = CompressedSNARK::<_, _, _, S1, S2>::prove(
        pp,
        &p_i.snark_pk,
        &recursive_snark,
        p_i.random_layer.clone(),
    );
    assert!(compressed_snark.is_ok());

    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Prover, "compressed_snark");
        log::stop(Component::Prover, "prove_e2e");
    }

    Ok(ProverOutput {
        compressed_snark: compressed_snark.unwrap(),
        z_0: z0_primary,
        empty: None,
    })
}
