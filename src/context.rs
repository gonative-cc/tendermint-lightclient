use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core::client::context::ExtClientValidationContext;

use ibc_core::client::types::error::ClientError;
use ibc_core::handler::types::error::ContextError;

use ibc_core::{
    client::{
        context::{
            client_state::{ClientStateExecution, ClientStateCommon}, ClientExecutionContext, ClientValidationContext,
        },
        types::Height,
    },
    host::types::identifiers::ClientId,
};
use tendermint::Time;

use crate::storage::{Direction, Storage};

pub struct Ctx<C: ClientType> {
    storage: Storage<C>,
}

impl<C: ClientType> Default for Ctx<C> {
    fn default() -> Self {
        Self {
            storage: Storage::default(),
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

    fn client_state(&self, _client_id: &ClientId) -> Result<Self::ClientStateRef, ContextError> {
        Ok(self.storage.client_state.clone().unwrap())
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ibc_core::host::types::path::ClientConsensusStatePath,
    ) -> Result<Self::ConsensusStateRef, ContextError> {
        let cons_state = self
            .storage
            .consensus_state
            .get(&client_cons_state_path.leaf());
        match cons_state {
            Some(state) => Ok(state.to_owned()),
            None => Err(ContextError::ClientError(
                ibc_core::client::types::error::ClientError::ConsensusStateNotFound {
                    client_id: client_cons_state_path.clone().client_id,
                    height: Height::new(
                        client_cons_state_path.revision_number,
                        client_cons_state_path.revision_height,
                    )
                    .unwrap(),
                },
            )),
        }
    }

    fn client_update_meta(
        &self,
        client_id: &ibc_core::host::types::identifiers::ClientId,
        height: &ibc_core::client::types::Height,
    ) -> Result<(ibc_core::primitives::Timestamp, Height), ContextError> {
        match self.storage.update_meta.get(height) {
            Some(meta) => Ok(meta.to_owned()),
            None => Err(ClientError::UpdateMetaDataNotFound {
                client_id: client_id.clone(),
                height: *height,
            }
            .into()),
        }
    }
}

impl<C: ClientType> ClientExecutionContext for Ctx<C> {
    fn client_state_mut(
        &self,
        client_id: &ibc_core::host::types::identifiers::ClientId,
    ) -> Result<Self::ClientStateMut, ContextError> {
        //TODO: check client mut actual "mut"
        //If we only listen and verify message this api doesn't require
        self.client_state(client_id)
    }

    type ClientStateMut = C::ClientState;

    fn store_client_state(
        &mut self,
        _client_state_path: ibc_core::host::types::path::ClientStatePath,
        client_state: Self::ClientStateRef,
    ) -> Result<(), ContextError> {
        self.storage.client_state = Some(client_state);
        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        consensus_state_path: ibc_core::host::types::path::ClientConsensusStatePath,
        consensus_state: Self::ConsensusStateRef,
    ) -> Result<(), ContextError> {
        self.storage
            .consensus_state
            .insert(consensus_state_path.leaf(), consensus_state);
        Ok(())
    }

    fn delete_consensus_state(
        &mut self,
        consensus_state_path: ibc_core::host::types::path::ClientConsensusStatePath,
    ) -> Result<(), ContextError> {
        self.storage
            .consensus_state
            .remove(&consensus_state_path.leaf());
        Ok(())
    }

    fn store_update_meta(
        &mut self,
        _client_id: ibc_core::host::types::identifiers::ClientId,
        height: Height,
        host_timestamp: ibc_core::primitives::Timestamp,
        host_height: Height,
    ) -> Result<(), ContextError> {
        self.storage
            .update_meta
            .insert(height, (host_timestamp, host_height));
        Ok(())
    }

    fn delete_update_meta(
        &mut self,
        _client_id: ibc_core::host::types::identifiers::ClientId,
        height: Height,
    ) -> Result<(), ContextError> {
        self.storage.update_meta.remove(&height);
        Ok(())
    }
}

impl<C: ClientType> ExtClientValidationContext for Ctx<C> {
    fn host_timestamp(&self) -> Result<ibc_core::primitives::Timestamp, ContextError> {
        Ok(Time::now().into())
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        let h = Height::new(0, 1)?;
        Ok(h)
    }

    fn consensus_state_heights(&self, _client_id: &ClientId) -> Result<Vec<Height>, ContextError> {
        Ok(self.storage.get_heights())
    }

    fn next_consensus_state(
        &self,
        _client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ContextError> {
        Ok(self.storage.get_adjacent_height(height, Direction::Next))
    }

    fn prev_consensus_state(
        &self,
        _client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ContextError> {
        Ok(self
            .storage
            .get_adjacent_height(height, Direction::Previous))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{str::FromStr, time::Duration};

    use crate::api::TendermintClient;

    use base64::Engine;
    use ibc_client_tendermint::{
        client_state::ClientState,
        types::{
            AllowUpdate, ClientState as ClientStateType, ConsensusState as ConsensusStateType,
            Header, TrustThreshold,
        },
    };

    use ibc_core::{client::context::client_state::ClientStateValidation, commitment_types::commitment::{CommitmentPrefix, CommitmentProofBytes, CommitmentRoot}, host::types::{identifiers::{ChannelId, PortId, Sequence}, path::{CommitmentPath, Path}}};

    use ibc_core::{commitment_types::specs::ProofSpecs, host::types::identifiers::ChainId};

    use tendermint::{time::Time, Hash};

    // TODO: Get msg from protobuf
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
    fn verify_client_message() {
        let five_year = 5 * 365 * 24 * 60 * 60;

        let params: ClientStateParams = ClientStateParams {
            id: ChainId::new("chain2").unwrap(),
            trust_level: TrustThreshold::ONE_THIRD,
            trusting_period: Duration::new(five_year, 0),
            unbonding_period: Duration::new(five_year + 1, 0),
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
            .verify_client_message(&ctx, &client_id, header.clone().into())
            .expect("Not fails");

        // update don't check status of header. We need verify it first in logic.
        client
            .update_state(&mut ctx, &client_id, header.into())
            .expect("Not fails");


    }

    #[test] 
    fn verify_membership_test() {
        let five_year = 5 * 365 * 24 * 60 * 60;

        let params: ClientStateParams = ClientStateParams {
            id: ChainId::new("chain2").unwrap(),
            trust_level: TrustThreshold::ONE_THIRD,
            trusting_period: Duration::new(five_year, 0),
            unbonding_period: Duration::new(five_year + 1, 0),
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

        let prefix = CommitmentPrefix::try_from("transfer".as_bytes().to_vec()).unwrap();
        let proof_bytes = base64_to_bytes("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");

        let proof = CommitmentProofBytes::try_from(proof_bytes).unwrap();
        let root= CommitmentRoot::from_bytes(&base64_to_bytes("OrwyiNobhGhuFDpKXxy1qZWfkUpaTiTxvz+dSOqhfrs="));
        let port_id = PortId::new("transfer".to_owned()).unwrap();
        let channel_id = ChannelId::new(0);
        let sequence = Sequence::from(3512624);

        let path = Path::Commitment(CommitmentPath::new(&port_id, &channel_id, sequence));
        // value = 
        // client.verify_membership(&prefix, &proof, &root, path, value); 
    }


}
