use std::collections::HashMap;
use std::marker::PhantomData;
use std::str::FromStr;

use ibc_client_tendermint::client_state::ClientState;
use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core::client::context::ExtClientValidationContext;
use ibc_core::primitives::Timestamp;
use ibc_core::{
    client::{
        context::{
            client_state::{ClientStateCommon, ClientStateExecution, ClientStateValidation},
            ClientExecutionContext, ClientValidationContext,
        },
        types::Height,
    },
    host::types::identifiers::ClientId,
};
use tendermint::Time;

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
        println!("{}", client_state_path);
        self.client = client_state;
        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        consensus_state_path: ibc_core::host::types::path::ClientConsensusStatePath,
        consensus_state: Self::ConsensusStateRef,
    ) -> Result<(), ibc_core::handler::types::error::ContextError> {
        println!("{}", consensus_state_path);
        self.consensus_state = consensus_state;
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
        Ok(Time::from_str(&"2023-03-10T13:59:35.188345Z").unwrap().into())
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

    use crate::{api::TendermintClient};

    use super::{ClientType, *};
    use base64::Engine;
    use ibc_client_tendermint::{
        consensus_state::{self, ConsensusState},
        types::{
            AllowUpdate, ClientState as ClientStateType, ConsensusState as ConsensusStateType,
            Header, TrustThreshold,
        },
    };

    use ibc_core::{
        client,
        commitment_types::{commitment::CommitmentRoot, specs::ProofSpecs},
        host::types::identifiers::ChainId,
        primitives::{proto::Any, Timestamp},
    };
    use tendermint::{block::header, serializers::timestamp, time::Time, Hash};
    use tendermint_testgen::{Generator, Validator};

    use base64::{engine::general_purpose, Engine as _};

    /// Test fixture
    #[derive(Clone, Debug)]
    pub struct Fixture {
        pub chain_id: ChainId,
        pub trusted_timestamp: Timestamp,
        pub trusted_height: Height,
        pub validators: Vec<Validator>,
    }

    impl Default for Fixture {
        fn default() -> Self {
            println!("{:?}", Validator::new("1").voting_power(12));
            Fixture {
                chain_id: ChainId::new("chain2").unwrap(),
                trusted_timestamp: Timestamp::now(),
                trusted_height: Height::new(0, 6).unwrap(),
                validators: vec![
                    Validator::new("1").voting_power(20),
                    Validator::new("2").voting_power(30),
                    Validator::new("3").voting_power(30),
                ],
            }
        }
    }

    fn get_header() -> Header {
        serde_json::from_str::<Header>(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/data/header.json"
        ))).unwrap()
    }

    impl Fixture {
        fn dummy_header(&self, header_height: Height) -> Header {
            let header = tendermint_testgen::Header::new(&self.validators)
                .chain_id(self.chain_id.as_str())
                .height(header_height.revision_height())
                .time(Time::now())
                .next_validators(&self.validators)
                .app_hash(vec![0; 32].try_into().expect("never fails"));

            let light_block = tendermint_testgen::LightBlock::new_default_with_header(header)
                .generate()
                .expect("failed to generate light block");

            let tm_header = Header {
                signed_header: light_block.signed_header,
                validator_set: light_block.validators,
                trusted_height: self.trusted_height,
                trusted_next_validator_set: light_block.next_validators,
            };

            return tm_header;
        }

        pub fn dummy_client_message(&self, target_height: Height) -> Header {
            self.dummy_header(target_height)
        }
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
        base64::engine::general_purpose::STANDARD.decode(base64_str). unwrap()
    }
    pub fn dummy_sov_consensus_state() -> ConsensusStateType {
        ConsensusStateType::new(
            base64_to_bytes("EIP4I6oX9Nf8icn2zA11HBeAwjEfabYIUsw9TDd/2iI="). into(),
            Time::from_str(&"2023-03-10T11:56:35.188345Z").expect("not failed"),
            // Hash of default validator set
            Hash::from_str("46DED613D8C7893433B18818CF0FF8D2E918F9A3CE824CAD76FDDAC1F1BAFAF5")
                .expect("Never fails"),
        )
    }

    #[test]
    fn it_works() {
        let default_params: ClientStateParams = ClientStateParams {
            id: ChainId::new("chain2").unwrap(),
            trust_level: TrustThreshold::ONE_THIRD,
            trusting_period: Duration::new(1209600, 0),
            unbonding_period: Duration::new(1814400, 0),
            max_clock_drift: Duration::new(40, 0),
            latest_height: Height::new(0, 6).expect("Never fails"),
            proof_specs: ProofSpecs::cosmos(),
            upgrade_path: vec![
                "upgrade".to_string(),
                "upgradedIBCState".to_string()
              ],
            allow_update: AllowUpdate {
                after_expiry: true,
                after_misbehaviour: true,
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
        let consensus_state = dummy_sov_consensus_state();
        let mut ctx: Ctx<TendermintClient> = Ctx {
            client: client.clone(),
            consensus_state: consensus_state.into(),
            _marker: PhantomData,
        };

        let client_id = ClientId::new("my_client", 10).unwrap();
        client
            .initialise(
                &mut ctx,
                &client_id,
                dummy_sov_consensus_state().into(),
            )
            .unwrap();

        let header = get_header();

        client
            .verify_client_message(&ctx, &client_id, header.clone().into())
            .unwrap();

        // client.update_state(&mut ctx , &client_id, header.into()).unwrap();
    }
}
