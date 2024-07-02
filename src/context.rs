use std::marker::PhantomData;

use ibc_client_tendermint::{client_state::ClientState, consensus_state::ConsensusState};
use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core::{
    client::{
        context::{
            client_state::ClientStateExecution, ClientExecutionContext, ClientValidationContext,
        },
        types::{error::ClientError, Height},
    },
    host::types::identifiers::ClientId,
    primitives::proto::Any,
};

pub struct Ctx<C: ClientType> {
    _marker: PhantomData<C>,
}

pub trait ClientType: Sized
// where
//     <Self::ClientState as TryFrom<Any>>::Error: Into<ClientError>,
//     <Self::ConsensusState as TryFrom<Any>>::Error: Into<ClientError>,
{
    type ClientState: ClientStateExecution<Ctx<Self>>;
    type ConsensusState: ConsensusStateTrait;
}

impl<C: ClientType> ClientValidationContext for Ctx<C> {
    type ClientStateRef = C::ClientState;

    type ConsensusStateRef = C::ConsensusState;

    fn client_state(
        &self,
        client_id: &ClientId,
    ) -> Result<Self::ClientStateRef, ibc_core::handler::types::error::ContextError> {
        todo!()
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ibc_core::host::types::path::ClientConsensusStatePath,
    ) -> Result<Self::ConsensusStateRef, ibc_core::handler::types::error::ContextError> {
        todo!()
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
        todo!()
    }

    fn store_consensus_state(
        &mut self,
        consensus_state_path: ibc_core::host::types::path::ClientConsensusStatePath,
        consensus_state: Self::ConsensusStateRef,
    ) -> Result<(), ibc_core::handler::types::error::ContextError> {
        todo!()
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
        todo!()
    }

    fn delete_update_meta(
        &mut self,
        client_id: ibc_core::host::types::identifiers::ClientId,
        height: Height,
    ) -> Result<(), ibc_core::handler::types::error::ContextError> {
        todo!()
    }
}

// impl ExtClientExecutionContext for Ctx {}
