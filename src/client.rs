use futures::executor::block_on;
use ibc_client_tendermint::{
    consensus_state::ConsensusState,
    types::{ConsensusState as ConsensusStateType, Header},
};
use ibc_core::{client::types::Height, commitment_types::commitment::CommitmentRoot};
use tendermint::{
    account::Id,
    block::{
        self,
        signed_header::{self, SignedHeader},
    },
    proposal,
};
use tendermint_rpc::{Client, HttpClient, Paging, Url};

pub struct LightClientProvider {
    provider: HttpClient,
}

impl LightClientProvider {
    pub fn new(url: Url) -> Self {
        Self {
            provider: HttpClient::new(url.clone()).unwrap(),
        }
    }

    pub async fn consensus_state(&self, height: u32) -> ConsensusState {
        let block = self.provider.block(height).await.unwrap();

        let timestamp = block.block.header.time;
        let next_validators_hash = block.block.header.next_validators_hash;
        let root = block.block.header.app_hash;

        ConsensusState::from(ConsensusStateType {
            next_validators_hash,
            root: CommitmentRoot::from_bytes(root.as_bytes()),
            timestamp,
        })
    }

    pub async fn light_header(&self, height: u32) -> Header {
        let signed_header = self.get_signed_header(height).await;

        Header {
            signed_header: signed_header.clone(),
            trusted_height: Height::new(0, 6).unwrap(),
            trusted_next_validator_set: self.get_validator_set(height + 1, None).await,
            validator_set: self
                .get_validator_set(height, Some(signed_header.header.proposer_address))
                .await,
        }
    }

    pub async fn latest_height(&self) -> u64 {
        let block = self.provider.latest_block_results().await;
        block.unwrap().height.into()
    }

    pub async fn get_signed_header(&self, height: u32) -> SignedHeader {
        let commit = self.provider.commit(height).await;
        commit.unwrap().signed_header
    }

    pub async fn get_validator_set(
        &self,
        height: u32,
        proposer: Option<Id>,
    ) -> tendermint::validator::Set {
        let validators = self.provider.validators(height, Paging::All).await.unwrap();
        match proposer {
            Some(proposer_id) => {
                tendermint::validator::Set::with_proposer(validators.validators, proposer_id)
                    .unwrap()
            }
            None => tendermint::validator::Set::without_proposer(validators.validators),
        }
    }

    pub async fn fetch_block(&self) {
        let ans = self.provider.latest_block().await.unwrap();
        println!("{:#?}", ans.block);
    }
}

#[cfg(test)]
mod provider_test {
    use super::LightClientProvider;

    #[tokio::test]
    async fn fetch_block_test() {
        let url_str = "http://127.0.0.1:27010".to_string();
        let url = url_str.parse().unwrap();
        let provider = LightClientProvider::new(url);
        let ans = provider.fetch_block().await;
        println!("ans {:?}", ans);
    }
}
