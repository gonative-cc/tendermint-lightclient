use std::{
    error::Error,
    fs::{self},
    time::Duration,
};

use api::TendermintClient;
use clap::Parser;
use context::Ctx;
use ibc_client_tendermint::{
    client_state::ClientState,
    types::{AllowUpdate, ClientState as ClientStateType, ConsensusState, Header, TrustThreshold},
};

use ibc_core::{
    client::context::client_state::{ClientStateExecution, ClientStateValidation, ClientStateCommon},
    host::types::identifiers::ClientId,
};
use ibc_core::{
    client::types::Height, commitment_types::specs::ProofSpecs, host::types::identifiers::ChainId,
};

mod api;
mod context;
mod provider;
mod storage;
mod utils;

#[derive(Parser, Debug)]
enum LCCLi {
    Verify {
        cs_path: String,
        header_path: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = LCCLi::parse();
    let five_year = 5 * 365 * 24 * 60 * 60;

    // TODO: READ it from json file also
    let client = ClientStateType::new(
        ChainId::new("ibc-0").unwrap(),
        TrustThreshold::ONE_THIRD,
        Duration::new(five_year, 0),
        Duration::new(five_year + 1, 0),
        Duration::new(40, 0),
        Height::new(0, 6).expect("Never fails"),
        ProofSpecs::cosmos(),
        vec!["upgrade".to_string(), "upgradedIBCState".to_string()],
        AllowUpdate {
            after_expiry: true,
            after_misbehaviour: true,
        },
    )?;

    let client: ClientState = ClientState::from(client);
    let mut ctx: Ctx<TendermintClient> = Ctx::default();
    let client_id = ClientId::new("stand-alone", 0)?;

    match cli {
        LCCLi::Verify {
            cs_path,
            header_path,
        } => {
            // we cannot init and verify in separate command 
            // b/c we need storage the consensus state hand latest trusted height.
            // so I do both here. However when we can separate 2 action if we have data base to store context i.e blockchain. 
            let cs_content = fs::read_to_string(cs_path)?;
            let cs: ConsensusState = serde_json::from_str(&cs_content)?;
            client.initialise(&mut ctx, &client_id, cs.into())?;
            let header_content = fs::read_to_string(header_path)?;
            let lc_header: Header = serde_json::from_str(&header_content)?;
            client.verify_client_message(&ctx, &client_id, lc_header.into())?;

            // client.verify_membership(prefix, proof, root, path, value)
        }

    }

    Ok(())
}
