use std::marker::PhantomData;
use std::str::FromStr;

use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core::client::context::ExtClientValidationContext;

use ibc_core::{
    client::{
        context::{
            client_state::ClientStateExecution, ClientExecutionContext, ClientValidationContext,
        },
        types::Height,
    },
    host::types::identifiers::ClientId,
};
use tendermint::Time;

#[derive(Debug)]
pub struct Ctx<C: ClientType> {
    client: Option<C::ClientState>,
    consensus_state: Option<C::ConsensusState>,
    _marker: PhantomData<C>,
}


impl<C: ClientType> Default for Ctx<C> {
    fn default() -> Self {
        Self {
            client: None,
            consensus_state:None,
            _marker: PhantomData
        }
    }
}
pub trait ClientType: Sized {
    type ClientState: ClientStateExecution<Ctx<Self>> + Clone;
    type ConsensusState: ConsensusStateTrait + Clone;
}

impl<C: ClientType> ClientValidationContext for Ctx<C> {
    type ClientStateRef = C::ClientState;

    type ConsensusStateRef = C::ConsensusState;

    fn client_state(
        &self,
        _client_id: &ClientId,
    ) -> Result<Self::ClientStateRef, ibc_core::handler::types::error::ContextError> {
        Ok(self.client.clone().unwrap())
    }

    fn consensus_state(
        &self,
        _client_cons_state_path: &ibc_core::host::types::path::ClientConsensusStatePath,
    ) -> Result<Self::ConsensusStateRef, ibc_core::handler::types::error::ContextError> {
        Ok(self.consensus_state.clone().unwrap())
    }

    fn client_update_meta(
        &self,
        _client_id: &ibc_core::host::types::identifiers::ClientId,
        _height: &ibc_core::client::types::Height,
    ) -> Result<
        (ibc_core::primitives::Timestamp, Height),
        ibc_core::handler::types::error::ContextError,
    > {
        todo!()
    }
}
impl<C: ClientType> ClientExecutionContext for Ctx<C> {
    fn client_state_mut(
        &self,
        _client_id: &ibc_core::host::types::identifiers::ClientId,
    ) -> Result<Self::ClientStateMut, ibc_core::handler::types::error::ContextError> {
        todo!()
    }

    type ClientStateMut = C::ClientState;

    fn store_client_state(
        &mut self,
        _client_state_path: ibc_core::host::types::path::ClientStatePath,
        client_state: Self::ClientStateRef,
    ) -> Result<(), ibc_core::handler::types::error::ContextError> {
        self.client = Some(client_state);
        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        _consensus_state_path: ibc_core::host::types::path::ClientConsensusStatePath,
        consensus_state: Self::ConsensusStateRef,
    ) -> Result<(), ibc_core::handler::types::error::ContextError> {
        self.consensus_state = Some(consensus_state);
        Ok(())
    }

    fn delete_consensus_state(
        &mut self,
        _consensus_state_path: ibc_core::host::types::path::ClientConsensusStatePath,
    ) -> Result<(), ibc_core::handler::types::error::ContextError> {
        todo!()
    }

    fn store_update_meta(
        &mut self,
        _client_id: ibc_core::host::types::identifiers::ClientId,
        _height: Height,
        _host_timestamp: ibc_core::primitives::Timestamp,
        _host_height: Height,
    ) -> Result<(), ibc_core::handler::types::error::ContextError> {
        Ok(())
    }

    fn delete_update_meta(
        &mut self,
        _client_id: ibc_core::host::types::identifiers::ClientId,
        _height: Height,
    ) -> Result<(), ibc_core::handler::types::error::ContextError> {
        todo!()
    }
}

impl<C: ClientType> ExtClientValidationContext for Ctx<C> {
    fn host_timestamp(
        &self,
    ) -> Result<ibc_core::primitives::Timestamp, ibc_core::handler::types::error::ContextError>
    {
        // TODO: mock it
        Ok(Time::from_str("2023-03-10T13:59:35.188345Z")
            .unwrap()
            .into())
    }

    fn host_height(&self) -> Result<Height, ibc_core::handler::types::error::ContextError> {
        let h = Height::new(0, 12)?;
        Ok(h)
    }

    fn consensus_state_heights(
        &self,
        _client_id: &ClientId,
    ) -> Result<Vec<Height>, ibc_core::handler::types::error::ContextError> {
        todo!()
    }

    fn next_consensus_state(
        &self,
        _client_id: &ClientId,
        _height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ibc_core::handler::types::error::ContextError>
    {
        todo!()
    }

    fn prev_consensus_state(
        &self,
        _client_id: &ClientId,
        _height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ibc_core::handler::types::error::ContextError>
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use crate::api::TendermintClient;


    use base64::Engine;
    use ibc_client_tendermint::{client_state::ClientState, types::{
        AllowUpdate, ClientState as ClientStateType, ConsensusState as ConsensusStateType, Header,
        TrustThreshold,
    }};

    use ibc_core::client::context::client_state::ClientStateValidation;

    use ibc_core::{commitment_types::specs::ProofSpecs, host::types::identifiers::ChainId};
    use tendermint::{time::Time, Hash};

   
    fn get_header() -> Header {
        serde_json::from_str::<Header>(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/data/header.json"
        )))
        .unwrap()
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct ClientStateParams {
        pub id: ChainId,
        pub trust_level: TrustThreshold,
        pub trusting_period: Duration,
        pub unbonding_period: Duration,
        pub max_clock_drift: Duration,
        pub latest_height: Height,
        pub proof_specs: ProofSpecs,
        pub upgrade_path: Vec<String>,
        pub allow_update: AllowUpdate,
    }

    pub fn base64_to_bytes(base64_str: &str) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(base64_str)
            .unwrap()
    }
    pub fn dummy_consensus_state() -> ConsensusStateType {
        ConsensusStateType::new(
            base64_to_bytes("EIP4I6oX9Nf8icn2zA11HBeAwjEfabYIUsw9TDd/2iI=").into(),
            Time::from_str("2023-03-10T11:56:35.188345Z").expect("not failed"),
            // Hash of default validator set
            Hash::from_str("46DED613D8C7893433B18818CF0FF8D2E918F9A3CE824CAD76FDDAC1F1BAFAF5")
                .expect("Never fails"),
        )
    }

    #[test]
    fn it_works() {
        let params: ClientStateParams = ClientStateParams {
            id: ChainId::new("chain2").unwrap(),
            trust_level: TrustThreshold::ONE_THIRD,
            trusting_period: Duration::new(1209600, 0),
            unbonding_period: Duration::new(1814400, 0),
            max_clock_drift: Duration::new(40, 0),
            latest_height: Height::new(0, 6).expect("Never fails"),
            proof_specs: ProofSpecs::cosmos(),
            upgrade_path: vec!["upgrade".to_string(), "upgradedIBCState".to_string()],
            allow_update: AllowUpdate {
                after_expiry: true,
                after_misbehaviour: true,
            },
        };

        let client = ClientStateType::new(
            params.id,
            params.trust_level,
            params.trusting_period,
            params.unbonding_period,
            params.max_clock_drift,
            params.latest_height,
            params.proof_specs,
            params.upgrade_path,
            params.allow_update,
        )
        .unwrap();

        let client = ClientState::from(client);

        let mut ctx: Ctx<TendermintClient> = Ctx::default();
        let client_id = ClientId::new("my_client", 10).unwrap();

        let consensus_state = dummy_consensus_state();
        client
            .initialise(&mut ctx, &client_id, consensus_state.into())
            .expect("Not fails");

        let header = get_header();
        client
            .verify_client_message(&ctx, &client_id, header.into())
        .expect("Not fails")
    }
}
