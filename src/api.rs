use ibc_client_tendermint::{client_state::ClientState, consensus_state::ConsensusState};

use crate::context::ClientType;

pub struct TendermintClient;

impl ClientType for TendermintClient {
    type ClientState = ClientState;
    type ConsensusState = ConsensusState;
}

