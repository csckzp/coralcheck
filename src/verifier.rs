use crate::{prover::{GrammarCommitment, ProverOutput}, solver::*, util::*};
use ark_ff::PrimeField as arkPrimeField;
use nova_snark::nova::CompressedSNARK;
use nova_snark::{
    errors::NovaError,
    nova::{PublicParams, VerifierKey},
    traits::ROConstants,
};
use segmented_circuit_memory::memory::nebula::RunningMem;

use std::usize;

#[cfg(feature = "metrics")]
use metrics::metrics::{log, log::Component};

pub struct VerifierInfo<ArkF: arkPrimeField> {
    pub tree_size: usize,
    pub pp: PublicParams<E1, E2, C1>,
    pub num_steps: usize,
    pub mem: RunningMem<AF>,
    pub snark_vk: VerifierKey<E1, E2, C1, S1, S2>,
    pub perm_chal: Vec<ArkF>,
    pub r0_consts: ROConstants<E1>
}

pub fn setup(empty_circuit: &mut CoralStepCircuit<AF>) -> VerifierInfo<AF> {
    #[cfg(feature = "metrics")]
    log::tic(Component::Generator, "nova_pp_gen_v");
    let pp = gen_pp(empty_circuit);
    #[cfg(feature = "metrics")]
    log::stop(Component::Generator, "nova_pp_gen_v");

    #[cfg(feature = "metrics")]
    log::tic(Component::Verifier, "snark_params_v");
    let (_, vk) = CompressedSNARK::<_, _, _, S1, S2>::setup(&pp).unwrap();

    #[cfg(feature = "metrics")]
    log::stop(Component::Verifier, "snark_params_v");

    VerifierInfo {
        tree_size: empty_circuit.tree_size_usize,
        pp,
        num_steps: usize::div_ceil(empty_circuit.tree_size_usize, empty_circuit.batch_size),
        mem: empty_circuit.mem.clone().unwrap(),
        snark_vk: vk,
        perm_chal: empty_circuit.mem.as_ref().unwrap().perm_chal.clone(),
        r0_consts: ROConstants::<E1>::default()
    }
}

/// Compute the expected running evaluation from a public document.
/// This replaces the KZG check: the verifier recomputes the product
/// polynomial evaluation from the known document and compares with
/// the value exposed by the circuit.
fn expected_running_eval(doc: &[char], perm_chal: AF) -> AF {
    let shift = AF::from(2_u64.pow(32));
    let epsilon_val: AF = coral_hash("");
    let mut eval = AF::ONE;
    let mut doc_ctr = 0u64;
    for c in doc {
        let char_hash: AF = coral_hash(&c.to_string());
        if char_hash != epsilon_val {
            let root = char_hash * shift + AF::from(doc_ctr);
            eval *= perm_chal - root;
            doc_ctr += 1;
        }
    }
    eval
}

pub fn verify(
    p_o: &mut ProverOutput,
    v_i: VerifierInfo<AF>,
    doc: &[char],
    _grammar_commit: &GrammarCommitment,
) -> Result<(), NovaError> {
    #[cfg(feature = "metrics")]
    log::tic(Component::Verifier, "full_verify");

    #[cfg(feature = "metrics")]
    log::tic(Component::Verifier, "snark_verify");

    let comp_snark_result = p_o
        .compressed_snark
        .verify(&v_i.snark_vk, v_i.num_steps, &p_o.z_0);
    assert!(comp_snark_result.is_ok());

    #[cfg(feature = "metrics")]
    log::stop(Component::Verifier, "snark_verify");

    // check final cmt outputs
    let (zn, ci) = comp_snark_result.unwrap();

    #[cfg(feature = "metrics")]
    log::tic(Component::Verifier, "eq_checks");

    v_i.mem.verifier_checks(&zn, &ci, v_i.r0_consts);
    #[cfg(feature = "metrics")]
    {
        log::stop(Component::Verifier, "eq_checks");
        log::tic(Component::Verifier, "doc_eval_check");
    }

    // Check the public document against the running evaluation
    let eval_offset = 13;
    let claimed_eval = zn[eval_offset];
    let claimed_eval_ark: AF =
        segmented_circuit_memory::bellpepper::nova_to_ark_field(&claimed_eval);
    let expected = expected_running_eval(doc, v_i.perm_chal[0]);
    assert_eq!(
        claimed_eval_ark, expected,
        "Document running-eval mismatch: proof does not match public document"
    );

    println!("Verified Successfully!");

    #[cfg(feature = "metrics")]
    log::stop(Component::Verifier, "doc_eval_check");

    #[cfg(feature = "metrics")]
    log::stop(Component::Verifier, "full_verify");

    Ok(())
}
