use std::time::Duration;

use api::TendermintClient;
use client::LightClientProvider;
use context::Ctx;
use ibc_client_tendermint::{
    client_state::ClientState,
    types::{AllowUpdate, ClientState as ClientStateType, TrustThreshold},
};
use ibc_core::{
    client::context::{
        client_state::{ClientStateExecution, ClientStateValidation},
        ClientExecutionContext, ClientValidationContext,
    },
    host::types::identifiers::ClientId,
};
use ibc_core::{
    client::types::Height,
    commitment_types::{commitment::CommitmentRoot, proto::ics23::Hash, specs::ProofSpecs},
    host::types::identifiers::ChainId,
};

mod api;
mod client;
mod context;
mod storage;

#[tokio::main]
async fn main() {
    let url = "http://127.0.0.1:27010".parse().unwrap();
    let provider = LightClientProvider::new(url);

    let cs = provider.consensus_state(6).await;

    let five_year = 5 * 365 * 24 * 60 * 60;

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
    )
    .unwrap();

    let client = ClientState::from(client);

    let mut ctx: Ctx<TendermintClient> = Ctx::default();

    let client_id = ClientId::new("stand-alone", 0).unwrap();
    client.initialise(&mut ctx, &client_id, cs.into()).unwrap();

    let header = provider.light_header(500).await;

    // client.verify_client_message(&ctx, &client_id, header.into()).unwrap();
    client
        .update_state(&mut ctx, &client_id, header.into())
        .unwrap();

    let header = provider.light_header(1000).await;

    client
        .verify_client_message(&ctx, &client_id, header.into())
        .unwrap();
}
