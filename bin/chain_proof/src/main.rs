use anyhow::Ok;
use bincode;
use bitcoin_header_chain::{read_users_from_file, BlockOut, PrevBlockContext};
use guest_code_builder::{GUEST_BITCOIN_ELF as ELF, GUEST_BITCOIN_ID as ID};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use std::time::Instant;

fn add_hint<T: serde::Serialize>(env: &mut Vec<u32>, item: T) {
    let mut serializer = risc0_zkvm::serde::Serializer::new(env);
    item.serialize(&mut serializer)
        .expect("Risc0 hint serialization is infallible");
}

fn main() -> anyhow::Result<()> {
    let block_headers = read_users_from_file("data/data-5.json").unwrap();
    let mut prev_proof: Option<Receipt> = None;

    let mut prev_context = PrevBlockContext::Block;

    for block_header in block_headers {
        println!(
            "\n\nProving for the block header {:?}",
            block_header.block_hash()
        );
        let mut builder = ExecutorEnv::builder();

        let mut env: Vec<u32> = Default::default();
        let env_data = (block_header, ID, prev_context);
        add_hint(&mut env, env_data);

        if let Some(ref receipt) = prev_proof {
            // Verifying a proof recursively requires adding the previous proof as an assumption.
            builder.add_assumption(receipt.clone());
        }

        // Obtain the default prover.
        let prover = default_prover();

        // Produce a receipt by proving the specified ELF binary.
        let env = builder.write_slice(&env).build().unwrap();
        let receipt = prover.prove(env, ELF)?;

        // Update the previous context to verify off the last proof.
        prev_context = PrevBlockContext::Proof {
            journal: receipt.journal.bytes.clone(),
        };
        prev_proof = Some(receipt)
    }

    let res: BlockOut = prev_proof.unwrap().journal.decode().unwrap();
    println!("Got the res {:#?} ", res);

    Ok(())
}
