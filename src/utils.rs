use std::{error::Error, fs::File};

use base64::Engine;
pub fn base64_to_bytes(base64_str: &str) -> Vec<u8> {
    base64::engine::general_purpose::STANDARD
        .decode(base64_str)
        .unwrap()
}

pub async fn fetch_consensus_state(url_str: String, output_path: String) -> Result<(), Box<dyn Error>> {
    use std::io::Write;

    use serde::{Deserialize, Serialize};
    use tendermint::{Hash, Time};

    use crate::provider::LightClientProvider;

    let provider = LightClientProvider::new(url_str.parse().unwrap());

    let mut file = File::create(output_path)?;
    let cs = provider.consensus_state(6).await;

    #[derive(Serialize, Deserialize)]
    struct Tmp<'a> {
        root: &'a [u8],
        timestamp: Time,
        next_validators_hash: Hash,
    }

    let tmp = Tmp {
        root: cs.root.as_bytes(),
        timestamp: cs.timestamp(),
        next_validators_hash: cs.next_validators_hash,
    };

    file.write_all(serde_json::to_string(&tmp)?.as_bytes())?;
    Ok(())
}
