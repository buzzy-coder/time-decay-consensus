use chrono::prelude::*;
use sha2::{Sha256, Digest};
use hex::encode;
use rand::Rng;

const DIFFICULTY: usize = 2; // For Proof of Work

pub struct Block {
    pub id: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: i64,
    pub data: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(id: u64, previous_hash: String, data: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            id,
            hash: String::new(),
            previous_hash,
            timestamp,
            data,
            nonce: 0,
        };
        block.mine_block();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.id.to_string());
        hasher.update(&self.previous_hash);
        hasher.update(self.timestamp.to_string());
        hasher.update(&self.data);
        hasher.update(self.nonce.to_string());
        encode(hasher.finalize())
    }

    pub fn mine_block(&mut self) {
        let target = "0".repeat(DIFFICULTY);
        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        println!("Block {} mined: {}", self.id, self.hash);
    }
}

pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            blocks: vec![],
        };
        blockchain.create_genesis_block();
        blockchain
    }

    fn create_genesis_block(&mut self) {
        let genesis_block = Block {
            id: 0,
            hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(), // A fixed hash for the genesis block
            previous_hash: String::new(),
            timestamp: Utc::now().timestamp(),
            data: "Genesis Block".to_string(),
            nonce: 0,
        };
        self.blocks.push(genesis_block);
    }

    pub fn add_block(&mut self, data: String) {
        let previous_block = self.blocks.last().expect("Blockchain should have at least one block");
        let new_block = Block::new(previous_block.id + 1, previous_block.hash.clone(), data);
        self.blocks.push(new_block);
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current_block = &self.blocks[i];
            let previous_block = &self.blocks[i - 1];

            if current_block.hash != current_block.calculate_hash() {
                println!("Invalid hash for block {}", current_block.id);
                return false;
            }

            if current_block.previous_hash != previous_block.hash {
                println!("Invalid previous hash for block {}", current_block.id);
                return false;
            }

            let target = "0".repeat(DIFFICULTY);
            if !current_block.hash.starts_with(&target) {
                println!("Block {} does not meet difficulty target", current_block.id);
                return false;
            }
        }
        true
    }
}