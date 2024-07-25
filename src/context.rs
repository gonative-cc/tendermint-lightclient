use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core::client::context::ExtClientValidationContext;

use ibc_core::client::types::error::ClientError;
use ibc_core::handler::types::error::ContextError;

use ibc_core::{
    client::{
        context::{
            client_state::{ClientStateCommon, ClientStateExecution},
            ClientExecutionContext, ClientValidationContext,
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

    use crate::{api::TendermintClient, utils::base64_to_bytes};

    use ibc_client_tendermint::{
        client_state::ClientState,
        types::{
            AllowUpdate, ClientState as ClientStateType, ConsensusState as ConsensusStateType,
            Header, TrustThreshold,
        },
    };

    use ibc_core::{
        channel::types::{
            commitment::compute_packet_commitment,
            packet::{self, Packet, PacketState},
            timeout::TimeoutHeight,
        },
        client::context::client_state::ClientStateValidation,
        commitment_types::commitment::{CommitmentPrefix, CommitmentProofBytes, CommitmentRoot},
        host::types::{
            identifiers::{ChannelId, PortId, Sequence},
            path::{CommitmentPath, Path},
        },
        primitives::Timestamp,
    };

    use ibc_core::{commitment_types::specs::ProofSpecs, host::types::identifiers::ChainId};

    use tendermint::{serializers::timestamp, time::Time, Hash};

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

        // Data for this test from this tx:
        //  - https://www.mintscan.io/cosmos/tx/A0E69441FB46C5797C1193D6EAA7EB5A59A809F0433ECA6CE29D7CD3DEFED679?height=21413592&sector=json
        // This transaction transfer token from Osmosis to Cosmos Hub
        // ibc module prefix ons
        let ibc_prefix = CommitmentPrefix::try_from("ibc".as_bytes().to_vec()).unwrap();
        // The proof of store on Osmosis. This is proof_commitment field.
        let proof_bytes = base64_to_bytes("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");

        let proof: CommitmentProofBytes = CommitmentProofBytes::try_from(proof_bytes).unwrap();

        // This is the root of multistore or app_hash/root of Osmosis client on CosmosHub
        let root = CommitmentRoot::from_bytes(&base64_to_bytes(
            "QNvnS2A//laHEjW9qVB5hya4zOCyugOIXU4vi0vnBww=",
        ));
        // Those data help us get the path of Commitment Path. You can check packet field in MsgRecvPacket msg.
        let port_id = PortId::new("transfer".to_owned()).unwrap();
        let channel_id = ChannelId::new(0);
        let sequence = Sequence::from(3514632);

        // data, timeout_height and timeout_timestamp from packet field in MsgRecvPacket msg.
        let data =  base64_to_bytes("eyJhbW91bnQiOiIyMDE5NzM2NCIsImRlbm9tIjoidHJhbnNmZXIvY2hhbm5lbC0wL3VhdG9tIiwicmVjZWl2ZXIiOiJjb3Ntb3MxaDUyamF6bHBtZTJkNWwybWo1bHlhcnlhZmNrbGRqMjBlbHNreGwiLCJzZW5kZXIiOiJvc21vMWg1MmphemxwbWUyZDVsMm1qNWx5YXJ5YWZja2xkajIwM3lyeHNkIn0=");
        let timeout_height = TimeoutHeight::At(Height::new(4, 21413739).unwrap());
        let timeout_timestamp = Timestamp::from_nanoseconds(0).unwrap();

        // hash those data together.
        let value = compute_packet_commitment(&data, &timeout_height, &timeout_timestamp);
        let value = value.into_vec();

        // This is the path we save value on cosmos Store i.e Store[path] = value
        let path = Path::Commitment(CommitmentPath::new(&port_id, &channel_id, sequence));

        // we prove MultiStore[prefix == ibc_prefix, Store[path] == value] with proof.
        client
            .verify_membership(&ibc_prefix, &proof, &root, path, value)
            .expect("pass validate");
    }
}
