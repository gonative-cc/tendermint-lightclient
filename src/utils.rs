use std::io::Write;
use std::{error::Error, fs::File};

use serde::{Deserialize, Serialize};
use tendermint::{Hash, Time};

use base64::Engine;
use ibc_client_tendermint::types::ConsensusState;
pub fn base64_to_bytes(base64_str: &str) -> Vec<u8> {
    base64::engine::general_purpose::STANDARD
        .decode(base64_str)
        .unwrap()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CSReadable {
    root: Vec<u8>,
    timestamp: Time,
    next_validators_hash: Hash,
}

impl From<ConsensusState> for CSReadable{
    fn from(cs: ConsensusState) -> Self {
        let root = cs.root.clone();
        CSReadable {
            root: root.into_vec(),
            timestamp: cs.timestamp().clone(),
            next_validators_hash: cs.next_validators_hash,
        }
    }
}

pub async fn fetch_consensus_state(
    url_str: String,
    output_path: String,
) -> Result<(), Box<dyn Error>> {
    use crate::provider::LightClientProvider;

    let provider = LightClientProvider::new(url_str.parse().unwrap());

    let mut file = File::create(output_path)?;
    let cs = provider.consensus_state(6).await;

    file.write_all(serde_json::to_string(&CSReadable::from(cs))?.as_bytes())?;
    Ok(())
}


pub async fn fetch_header(url_str: String, output_path: String, height: u32) -> Result<(), Box<dyn Error>> {
    use crate::provider::LightClientProvider;

    let provider = LightClientProvider::new(url_str.parse().unwrap());  
    let mut file = File::create(output_path)?;
    let cs = provider.light_header(height).await;
    file.write_all(serde_json::to_string(&cs)?.as_bytes())?;
    Ok(())
}