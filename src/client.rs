use ibc_client_tendermint::types::Header;
use tendermint::block::{self, signed_header::SignedHeader, Height};
use tendermint_rpc::{Client, HttpClient, Url};
use tendermint_testgen::ValidatorSet;

pub struct LightClientProvider {
    url: Url,
}

impl LightClientProvider {
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    pub async fn get_signed_header(height: u32) -> SignedHeader {
        todo!()
    }

    pub async fn get_validator_set() -> ValidatorSet {
        todo!()
    }

    pub async fn get_trusted_next_validator_set() -> ValidatorSet {
        todo!()
    }
    
    pub async fn fetch_block(&self) {
        let client = HttpClient::new(self.url.clone()).unwrap();
        let ans = client.latest_block().await.unwrap();
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
