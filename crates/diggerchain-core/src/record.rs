//! Versioned expedition history. The private expedition secret is not a wallet key.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::vault::{Action, Expedition, MAX_ACTIONS, RULES_VERSION};

pub const FORMAT_VERSION: u32 = 1;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub format_version: u32,
    pub rules_version: u64,
    pub seed: u64,
    pub secret: Option<[u8; 32]>,
    pub actions: Vec<u8>,
    pub finished: bool,
}

pub fn expedition_id(secret: &[u8; 32]) -> [u8; 32] {
    Sha256::digest(secret).into()
}

pub fn seed_from_id(id: &[u8; 32]) -> u64 {
    u64::from_be_bytes(id[..8].try_into().expect("eight seed bytes"))
}

pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(result, "{byte:02x}").expect("writing a string cannot fail");
    }
    result
}

impl Record {
    pub fn new(secret: [u8; 32]) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            rules_version: RULES_VERSION,
            seed: seed_from_id(&expedition_id(&secret)),
            secret: Some(secret),
            actions: Vec::new(),
            finished: false,
        }
    }

    pub fn practice(seed: u64) -> Self {
        Self {
            seed,
            secret: None,
            ..Self::new([0; 32])
        }
    }

    pub fn id(&self) -> Option<[u8; 32]> {
        self.secret.as_ref().map(expedition_id)
    }

    pub fn replay(&self) -> Result<Expedition, String> {
        if self.format_version != FORMAT_VERSION || self.rules_version != RULES_VERSION {
            return Err("Unsupported save or rules version; this save was not changed.".into());
        }
        if self.id().is_some_and(|id| seed_from_id(&id) != self.seed) {
            return Err("The expedition secret does not match its seed.".into());
        }
        if self.actions.len() > MAX_ACTIONS {
            return Err("The save exceeds the expedition action limit.".into());
        }
        let mut game = Expedition::new(self.seed);
        for (index, &code) in self.actions.iter().enumerate() {
            let action = Action::from_code(code)
                .ok_or_else(|| format!("Unknown saved action at step {}.", index + 1))?;
            game.apply(action)
                .map_err(|error| format!("Invalid saved action at step {}: {error}", index + 1))?;
        }
        Ok(game)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::demonstration;

    #[test]
    fn history_restores_a_generated_expedition_and_rejects_corruption() {
        let mut record = Record::new([7; 32]);
        let actions = demonstration(record.seed, MAX_ACTIONS, true).unwrap();
        let mut expected = Expedition::new(record.seed);
        for &action in &actions {
            expected.apply(action).unwrap();
        }
        record.actions = actions.into_iter().map(Action::code).collect();
        record.finished = true;
        assert_eq!(record.replay().unwrap(), expected);
        record.seed ^= 1;
        assert!(record.replay().is_err());
        record.seed ^= 1;
        record.actions[0] = 0;
        assert!(record.replay().is_err());
        record.actions.clear();
        record.rules_version += 1;
        assert!(record.replay().is_err());
    }

    #[test]
    fn identity_uses_sha256_and_actions_have_one_encoding() {
        assert_eq!(
            hex(&expedition_id(&[0; 32])),
            "66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925"
        );
        for code in 1..=11 {
            let action = Action::from_code(code).unwrap();
            assert_eq!(action.code(), code);
            assert_eq!(Action::parse(action.name()), Some(action));
        }
        assert!(Action::from_code(0).is_none());
        assert!(Action::from_code(12).is_none());
    }
}
