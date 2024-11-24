use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::solana_program::hash::HASH_BYTES;
use anchor_lang::solana_program::instruction::Instruction;

use crate::errors::TreasuryError;
use crate::rule::{generic_hash, Rule, RULE_AUTHORIZATION_CONSTRAINT};
use crate::spend::{SpendState, TransferContext};
use crate::{
    nplog, safejar, RuleAddAuthorizationConstraint, SpendProcessAuthorizationConstraint,
    SpendProcessAuthorizationConstraintNoSigner,
};

impl<'info> RuleAddAuthorizationConstraint<'info> {
    pub fn process(&mut self) -> ProgramResult {
        let rule = AuthorizationConstraint::new(
            &self.authorizer.key(),
            Some(self.authorizer.key().clone()),
        );
        if self.accumulator.add(&rule).is_err() {
            return Err(ProgramError::Custom(TreasuryError::RuleAddFail.into()));
        }
        Ok(())
    }
}

impl<'info> SpendProcessAuthorizationConstraint<'info> {
    pub fn process(&mut self) -> ProgramResult {
        nplog!("rule process acs - 1 - {}", self.required_authorizer.key());
        let mut authorizer = None;
        for a in self.to_account_infos() {
            if a.is_signer && a.key() == self.required_authorizer.key() {
                authorizer = Some(self.required_authorizer.key());
                nplog!("rule process acs - 2");
                break;
            }
        }
        let rule = AuthorizationConstraint::new(&self.required_authorizer.key(), authorizer);
        self.request.process(&rule)?;

        Ok(())
    }
}

impl<'info> SpendProcessAuthorizationConstraintNoSigner<'info> {
    pub fn process(&mut self) -> ProgramResult {
        nplog!("rule process acns - 1 - {}", self.required_authorizer.key());
        // we must check if the authorizer has signed regardless.
        let mut authorizer = None;
        for a in self.to_account_infos() {
            if a.is_signer && a.key() == self.required_authorizer.key() {
                authorizer = Some(self.required_authorizer.key());
                nplog!("rule process acns - 2");
                break;
            }
        }
        let rule = AuthorizationConstraint::new(&self.required_authorizer.key(), authorizer);
        self.request.process(&rule)?;

        Ok(())
    }
}

// we only want to serialize required_authorizer to do the hash
#[derive(Clone)]
pub struct AuthorizationConstraint {
    pub required_authorizer: Pubkey,
    pub authorizer: Option<Pubkey>,
}

impl AnchorDeserialize for AuthorizationConstraint {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let ac = AuthorizationConstraintOnly::deserialize(buf)?;
        return Ok(Self::new(&ac.required_authorizer, None));
    }

    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let r = AuthorizationConstraintOnly::deserialize_reader(reader);
        if r.is_err() {
            return Err(r.err().unwrap());
        } else {
            let ac = r.unwrap();
            return Ok(Self::new(&ac.required_authorizer, None));
        }
    }
}

impl AnchorSerialize for AuthorizationConstraint {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let ac = AuthorizationConstraintOnly {
            required_authorizer: self.required_authorizer,
        };
        ac.serialize(writer)?;
        return Ok(());
    }
}

impl AuthorizationConstraint {
    pub fn new(required_authorizer: &Pubkey, authorizer: Option<Pubkey>) -> Self {
        Self {
            required_authorizer: required_authorizer.clone(),
            authorizer,
        }
    }
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct AuthorizationConstraintOnly {
    pub required_authorizer: Pubkey,
}

impl AuthorizationConstraintOnly {
    pub fn ac(&self, will_sign: bool) -> AuthorizationConstraint {
        let authorizer: Option<Pubkey>;
        if will_sign {
            authorizer = Some(self.required_authorizer.clone());
        } else {
            authorizer = None;
        }
        return AuthorizationConstraint {
            required_authorizer: self.required_authorizer.clone(),
            authorizer,
        };
    }
}

impl<'b> Rule<'b> for AuthorizationConstraint {
    fn id(&self) -> u8 {
        return RULE_AUTHORIZATION_CONSTRAINT;
    }

    fn process<'a>(&'a self, _state: &mut SpendState, _context: &TransferContext) -> Result<()> {
        if self.authorizer.is_none() {
            nplog!("authorizer has not signed");
            return Err(TreasuryError::RuleAuthorizerConstraintMustMatch.into());
        }
        nplog!("has signed");
        Ok(())
    }

    fn hash<'a>(&'a self, index: u8, prev_hash: &'a [u8]) -> Result<[u8; HASH_BYTES]> {
        let mut x = [0u8; std::mem::size_of::<AuthorizationConstraintOnly>()];
        let mut cursor = std::io::Cursor::new(x.as_mut());
        let ac = AuthorizationConstraintOnly {
            required_authorizer: self.required_authorizer.clone(),
        };
        ac.serialize(&mut cursor)?;
        //msg!("_______+++++rule({})={:X?}",x.len(),&x);
        return Ok(generic_hash(&index, &x, prev_hash));
    }
}
