use futures::executor::block_on;
use ibc_client_tendermint::types::Header;
use ibc_core::client::types::Height;
use tendermint::{
    account::Id,
    block::{
        self,
        signed_header::{self, SignedHeader},
    },
    proposal,
};
use tendermint_rpc::{Client, HttpClient, Paging, Url};
use tendermint_testgen::ValidatorSet;

pub struct LightClientProvider {
    provider: HttpClient,
}

impl LightClientProvider {
    pub fn new(url: Url) -> Self {
        Self {
            provider: HttpClient::new(url.clone()).unwrap(),
        }
    }

    pub async fn light_header(&self, height: u32) -> Header {
        let signed_header = self.get_signed_header(height).await;

        Header {
            signed_header: signed_header.clone(),
            trusted_height: Height::new(0, 12).unwrap(),
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
    use tendermint_rpc::Url;

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
