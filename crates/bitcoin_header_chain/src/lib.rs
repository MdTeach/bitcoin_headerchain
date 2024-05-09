pub use bitcoin::block::Header;
use bitcoin::{Target, Work};
use ethnum::U256;
use std::fs::File;

pub mod block_time_tracker;
pub mod genesis_config;

use borsh::{BorshDeserialize, BorshSerialize};

use genesis_config::{EXPECTED_BLOCK_TIME, GENESIS_HASH, MIN_TARGET};
use serde::{Deserialize, Serialize};

use crate::{
    block_time_tracker::RecentBlockTimeStamp,
    genesis_config::{DIFFICULTY_ADJUSTMENT_INTERVAL, EPOCH_END_INTERVAL, EPOCH_START_INTERVAL},
};

pub type Headers = Vec<Header>;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Serialize, Deserialize)]
pub struct BlockOut {
    pub block_number: u32,
    pub target_compact: u32,
    pub block_hash: String,
    pub epoch_start_timestamp: u32,
    pub epoch_end_timestamp: u32,
    pub total_pow: f64,
    pub recent_block_timestamp: [u32; 11],
}

pub fn read_users_from_file(file_path: &str) -> anyhow::Result<Vec<Header>> {
    // Open the file in read-only mode.
    let file = File::open(file_path).unwrap();

    let u = serde_json::from_reader(file).unwrap();
    Ok(u)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PrevBlockContext {
    Proof { journal: Vec<u8> },
    Block,
}

pub fn verify_genesis_block(header: Header) -> BlockOut {
    let block_hash = header.block_hash();
    assert_eq!(block_hash.to_string(), GENESIS_HASH);

    let block_out = BlockOut {
        block_number: 0,
        target_compact: header.target().to_compact_lossy().to_consensus(),
        block_hash: block_hash.to_string(),
        epoch_start_timestamp: header.time,
        epoch_end_timestamp: header.time,
        total_pow: header.difficulty_float(),
        recent_block_timestamp: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, header.time],
    };

    return block_out;
}

pub fn verify_other_block(header: Header, prev_block: BlockOut) -> BlockOut {
    let block_hash = header.block_hash().to_string();

    // Block Hash Check
    assert_eq!(prev_block.block_hash, header.prev_blockhash.to_string());

    // Pow Check
    assert!(header.validate_pow(header.target()).is_ok());

    // Difficulty Adjustment Check
    let mut epoch_start_timestamp = prev_block.epoch_start_timestamp;
    let mut epoch_end_timestamp = prev_block.epoch_end_timestamp;

    let current_target_compact = header.target().to_compact_lossy().to_consensus();
    let block_number = prev_block.block_number + 1;

    if block_number % DIFFICULTY_ADJUSTMENT_INTERVAL == 0 {
        println!(
            "Doing for the block_number {:?} {:?}",
            block_number,
            header.block_hash()
        );
        // Check if new difficulty was calculated properly
        let current_target = header.target().to_be_bytes();
        let expected_target =
            get_new_target(current_target, epoch_start_timestamp, epoch_end_timestamp);

        assert_eq!(current_target_compact, expected_target)
    } else {
        // The target should be constant
        assert_eq!(current_target_compact, prev_block.target_compact)
    }

    // Epoch Headers are tracked and processed properly
    if block_number % EPOCH_START_INTERVAL == 0 {
        epoch_start_timestamp = header.time;
    } else if block_number % EPOCH_END_INTERVAL == 0 {
        epoch_end_timestamp = header.time;
    }

    // Timestamp check
    let mut recent_block_timestamp = RecentBlockTimeStamp::new(&prev_block.recent_block_timestamp);
    let median_time = recent_block_timestamp.get_median_time();
    assert!(header.time > median_time);
    recent_block_timestamp.insert_timestamp(header.time);

    let block_out = BlockOut {
        block_number,
        target_compact: current_target_compact,
        block_hash: block_hash,
        epoch_end_timestamp,
        epoch_start_timestamp,
        total_pow: prev_block.total_pow + header.difficulty_float(),
        recent_block_timestamp: recent_block_timestamp.output(),
    };

    return block_out;
}

fn get_new_target(current_target: [u8; 32], first_block: u32, last_block: u32) -> u32 {
    let mut current_target = U256::from_be_bytes(current_target);
    let mut diff = last_block - first_block;

    // max decrease by 1/4th
    if diff < EXPECTED_BLOCK_TIME / 4 {
        diff = EXPECTED_BLOCK_TIME / 4
    }

    // max increase by 4x
    if diff > 4 * EXPECTED_BLOCK_TIME {
        diff = 4 * EXPECTED_BLOCK_TIME
    }

    current_target *= U256::from(diff);
    current_target /= U256::from(EXPECTED_BLOCK_TIME);

    let bytes = current_target.to_be_bytes();
    let target = Target::from_be_bytes(bytes)
        .to_compact_lossy()
        .to_consensus();

    println!("target: {:?} min-target: {:?}",target,MIN_TARGET);
    if target > MIN_TARGET {
        return MIN_TARGET;
    }

    target
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_new_target() {
        let first: u32 = 1457133956; //block 401_184
        let last: u32 = 1458291885; // block 403_199
        let prev_target: [u8; 32] = [
            0, 0, 0, 0, 0, 0, 0, 0, 6, 240, 168, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];

        // New Target in block 403_200
        // https://blockstream.info/block/000000000000000000c4272a5c68b4f55e5af734e88ceab09abf73e9ac3b6d01
        // // Expected target: 0x1806a4c3 -> 403088579
        let expected_target = 403088579;
        let new_target = get_new_target(prev_target, first, last);

        assert_eq!(new_target, expected_target)
    }

    #[test]
    fn test_headers() {
        let block_headers = read_users_from_file("../../data/data-5000.json").unwrap();

        // verify block
        let block = block_headers.first().unwrap().clone();
        let mut block_out = verify_genesis_block(block.clone());

        for header_idx in 1..block_headers.len() {
            // println!("Veriying {:?} {:?}",header_idx,block_headers[header_idx]);
            block_out = verify_other_block(block_headers[header_idx], block_out)
        }
    }
}
