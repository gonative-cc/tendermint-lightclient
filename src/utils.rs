#[cfg(test)]
mod gen_test {
    use std::{error::Error, fs::File};
    #[tokio::test]
    async fn create_test_data() -> Result<(), Box<dyn Error>> {
        use std::io::Write;

        use serde::{Deserialize, Serialize};
        use tendermint::{Hash, Time};

        use crate::provider::LightClientProvider;

        let provider = LightClientProvider::new("http://127.0.0.1:27010".parse().unwrap());

        let mut file = File::create("./src/data/cs.json")?;
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
            next_validators_hash: cs.next_validators_hash.into(),
        };

        file.write_all(serde_json::to_string(&tmp)?.as_bytes())?;

        let mut file = File::create("./src/data/header_500.json")?;
        let cs = provider.light_header(500).await;
        file.write_all(serde_json::to_string(&cs)?.as_bytes())?;

        let mut file = File::create("./src/data/header_1000.json")?;
        let cs = provider.light_header(1000).await;
        file.write_all(serde_json::to_string(&cs)?.as_bytes())?;
        Ok(())
    }

}
