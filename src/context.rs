use std::marker::PhantomData;

use ibc_client_tendermint::client_state::ClientState;
use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core::client::context::{ExtClientExecutionContext, ExtClientValidationContext};
use ibc_core::primitives::Timestamp;
use ibc_core::{
    client::{
        context::{
            client_state::ClientStateExecution, ClientExecutionContext, ClientValidationContext,
        },
        types::Height,
    },
    host::types::identifiers::ClientId,
};

pub struct Ctx<C: ClientType> {
    client: C::ClientState,
    consensus_state: C::ConsensusState,
    _marker: PhantomData<C>,
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
        Ok(self.client.clone())
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ibc_core::host::types::path::ClientConsensusStatePath,
    ) -> Result<Self::ConsensusStateRef, ibc_core::handler::types::error::ContextError> {
        Ok(self.consensus_state.clone())
    }

    fn client_update_meta(
        &self,
        client_id: &ibc_core::host::types::identifiers::ClientId,
        height: &ibc_core::client::types::Height,
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
        client_id: &ibc_core::host::types::identifiers::ClientId,
    ) -> Result<Self::ClientStateMut, ibc_core::handler::types::error::ContextError> {
        todo!()
    }

    type ClientStateMut = C::ClientState;

    fn store_client_state(
        &mut self,
        client_state_path: ibc_core::host::types::path::ClientStatePath,
        client_state: Self::ClientStateRef,
    ) -> Result<(), ibc_core::handler::types::error::ContextError> {
        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        consensus_state_path: ibc_core::host::types::path::ClientConsensusStatePath,
        consensus_state: Self::ConsensusStateRef,
    ) -> Result<(), ibc_core::handler::types::error::ContextError> {
        Ok(())
    }

    fn delete_consensus_state(
        &mut self,
        consensus_state_path: ibc_core::host::types::path::ClientConsensusStatePath,
    ) -> Result<(), ibc_core::handler::types::error::ContextError> {
        todo!()
    }

    fn store_update_meta(
        &mut self,
        client_id: ibc_core::host::types::identifiers::ClientId,
        height: Height,
        host_timestamp: ibc_core::primitives::Timestamp,
        host_height: Height,
    ) -> Result<(), ibc_core::handler::types::error::ContextError> {
        Ok(())
    }

    fn delete_update_meta(
        &mut self,
        client_id: ibc_core::host::types::identifiers::ClientId,
        height: Height,
    ) -> Result<(), ibc_core::handler::types::error::ContextError> {
        todo!()
    }
}

impl<C: ClientType> ExtClientValidationContext for Ctx<C> {
    fn host_timestamp(
        &self,
    ) -> Result<ibc_core::primitives::Timestamp, ibc_core::handler::types::error::ContextError>
    {
        Ok(Timestamp::now())
    }

    fn host_height(&self) -> Result<Height, ibc_core::handler::types::error::ContextError> {
        let h = Height::new(0, 12)?;
        Ok(h)
    }

    fn consensus_state_heights(
        &self,
        client_id: &ClientId,
    ) -> Result<Vec<Height>, ibc_core::handler::types::error::ContextError> {
        todo!()
    }

    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ibc_core::handler::types::error::ContextError>
    {
        todo!()
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ibc_core::handler::types::error::ContextError>
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use std::{str::FromStr as _, time::Duration};

    use crate::api::TendermintClient;

    use super::{ClientType, *};
    use ibc_client_tendermint::{
        consensus_state::{self, ConsensusState},
        types::{
            AllowUpdate, ClientState as ClientStateType, ConsensusState as ConsensusStateType,
            TrustThreshold,
        },
    };

    use ibc_core::{
        commitment_types::{commitment::CommitmentRoot, specs::ProofSpecs},
        host::types::identifiers::ChainId,
        primitives::Timestamp,
    };
    use tendermint::{serializers::timestamp, Hash};

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

    pub fn dummy_sov_consensus_state(timestamp: Timestamp) -> ConsensusStateType {
        ConsensusStateType::new(
            vec![0].into(),
            timestamp.into_tm_time().expect("Time exists"),
            // Hash of default validator set
            Hash::from_str("D6B93922C33AAEBEC9043566CB4B1B48365B1358B67C7DEF986D9EE1861BC143")
                .expect("Never fails"),
        )
    }

    #[test]
    fn it_works() {
        let default_params: ClientStateParams = ClientStateParams {
            id: ChainId::new("ibc-1").unwrap(),
            trust_level: TrustThreshold::ONE_THIRD,
            trusting_period: Duration::new(64000, 0),
            unbonding_period: Duration::new(128_000, 0),
            max_clock_drift: Duration::new(3, 0),
            latest_height: Height::new(1, 10).expect("Never fails"),
            proof_specs: ProofSpecs::cosmos(),
            upgrade_path: Vec::new(),
            allow_update: AllowUpdate {
                after_expiry: false,
                after_misbehaviour: false,
            },
        };

        let p = default_params.clone();

        let client = ClientStateType::new(
            p.id,
            p.trust_level,
            p.trusting_period,
            p.unbonding_period,
            p.max_clock_drift,
            p.latest_height,
            p.proof_specs,
            p.upgrade_path,
            p.allow_update,
        )
        .unwrap();

        let client = ClientState::from(client);
        let consensus_state = dummy_sov_consensus_state(Timestamp::now());
        let mut ctx: Ctx<TendermintClient> = Ctx {
            client: client.clone(),
            consensus_state: consensus_state.into(),
            _marker: PhantomData,
        };

        client
            .initialise(
                &mut ctx,
                &ClientId::new("my_client", 10).unwrap(),
                dummy_sov_consensus_state(Timestamp::now()).into(),
            )
            .unwrap();
    }
}
