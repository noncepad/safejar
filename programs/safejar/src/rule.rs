use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::solana_program::hash::{hash, Hash, HASH_BYTES};

use crate::errors::TreasuryError;
use crate::spend::{SpendState, TransferContext};
use crate::{nplog, tree, CreateRuleAccumulator};
use anchor_lang;
use anchor_lang::solana_program::instruction::Instruction;

pub(crate) const RULE_RATE_LIMITER: u8 = 1;
pub(crate) const RULE_PROGRAM_CONSTRAINT: u8 = 2;
pub(crate) const RULE_AUTHORIZATION_CONSTRAINT: u8 = 3;
pub(crate) const RULE_BALANCE_CONSTRAINT: u8 = 4;
pub(crate) const RULE_SWEEP: u8 = 5;

#[account]
pub struct RuleAccumulator {
    pub controller: Pubkey,
    pub index: u8,
    pub count: u8,
    pub hash: [u8; 32],
}

impl RuleAccumulator {
    // use this for unit tests
    pub fn new(controller: &Pubkey, tree: &[u8]) -> Result<Self> {
        nplog!("ra - 1");
        let (_x, count) = tree::deserialize(&tree.to_vec())?;
        nplog!("ra - 2");
        let mut ra = Self {
            controller: controller.clone(),
            index: 0,
            count,
            hash: RuleAccumulator::hash_init(),
        };
        nplog!("ra - 3");
        ra.hash_tree(&tree.to_vec());
        nplog!("ra - 4");
        return Ok(ra);
    }
    pub fn hash_init() -> [u8; 32] {
        return [0u8; 32];
    }

    // use this in anchor entrypoints
    pub fn init(&mut self, controller: &Pubkey, tree: &[u8]) -> Result<()> {
        let (_x, count) = tree::deserialize(&tree.to_vec())?;

        self.controller = controller.clone();
        self.index = 0;
        self.count = count;
        self.hash = RuleAccumulator::hash_init();
        self.hash_tree(&tree.to_vec());

        Ok(())
    }

    pub fn hash_tree(&mut self, tree: &Vec<u8>) {
        let mut a = Vec::new();
        a.append(self.hash.to_vec().as_mut());
        a.append(tree.clone().as_mut());
        self.hash = hash(&a).to_bytes();
        ////msg!("pre i=-1 hash={:X?}",&self.hash);
    }

    pub fn add(&mut self, rule: &dyn Rule) -> Result<()> {
        self.hash = rule.hash(self.index, &self.hash)?;
        ////msg!("add i={} hash={:X?}",self.index,&self.hash);
        self.index += 1;
        if self.count < self.index {
            ////msg!("rule range problem: count={} index={}",self.count,self.index);
            return Err(TreasuryError::RuleOutOfRange.into());
        }
        Ok(())
    }
}

impl<'info> CreateRuleAccumulator<'info> {
    // rule accumulator holds SOL to pay rent, including delegation account
    pub fn process(&mut self, tree: Vec<u8>) -> ProgramResult {
        self.accumulator.init(&self.controller.key(), &tree)?;
        Ok(())
    }
}

pub fn hash_rule_set(set: &[u8]) -> Hash {
    let mut first_hash: [u8; HASH_BYTES * 2] = [0u8; HASH_BYTES * 2];
    if set.len() == 0 {
        for i in 0..HASH_BYTES {
            first_hash[i] = ZERO_HASH[i];
        }
    } else {
        let h = hash(set);
        let mut i = 0;
        for x in h.as_ref() {
            first_hash[i] = *x;
            i += 1;
        }
    }
    for i in 0..HASH_BYTES {
        first_hash[HASH_BYTES + i] = HASH_RULE_SET[i];
    }

    //HASH_RULE_SET
    return hash(&first_hash);
}

pub trait Rule<'b> {
    fn id(&self) -> u8;
    fn hash<'a>(&'a self, index: u8, prev_hash: &'a [u8]) -> Result<[u8; 32]>;
    fn process<'a>(&'a self, state: &mut SpendState, context: &TransferContext) -> Result<()>;
}

pub(crate) fn generic_hash<'a>(index: &u8, serialized_rule: &[u8], prev_hash: &[u8]) -> [u8; 32] {
    ////msg!("hashing rule={:X?}",serialized_rule);
    let mut all = Vec::new();
    ////msg!("hashing - 1");
    for i in 0..(1 + prev_hash.len() + serialized_rule.len()) {
        ////msg!("hashing - 2 - i={}",i);
        if i == 0 {
            all.push(*index);
        } else if i < prev_hash.len() + 1 {
            all.push(prev_hash[i - 1]);
        } else {
            all.push(serialized_rule[i - prev_hash.len() - 1]);
        }
    }
    //msg!("resulting hash={:X?}",hash(&all).to_bytes());
    return hash(&all).to_bytes();
}

pub const ZERO_HASH: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const HASH_RULE_SET: &[u8] = b"hashing_rule_set";
