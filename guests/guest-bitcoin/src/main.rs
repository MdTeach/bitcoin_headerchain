#![no_main]
#![no_std] // std support is experimental

use bitcoin_header_chain;
use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);

use bincode;
use bitcoin_header_chain::{ Header, PrevBlockContext, verify_genesis_block, verify_other_block,BlockOut};

fn main() {
    let (header, id, prev_context): (Header, [u32; 8], PrevBlockContext) = env::read();

    let res = match prev_context {
        PrevBlockContext::Block  => {verify_genesis_block(header)},
        PrevBlockContext::Proof { journal } => {
            env::verify(id, &journal).expect("Failed to verify recursive journal");
            let block_out: BlockOut = borsh::from_slice(&journal).expect("Invalid journal format");
            verify_other_block(header, block_out)
        }
    };
    
    env::commit(&res); 
}
