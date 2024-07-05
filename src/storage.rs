use std::{
    collections::{BTreeMap, HashMap},
    ops::Bound,
};

use ibc_core::client::types::Height;

use crate::context::ClientType;

pub enum Direction {
    Next,
    Previous,
}

#[derive(Clone)]
pub struct Storage<C: ClientType> {
    pub client_state: Option<C::ClientState>,
    pub consensus_state: Option<C::ConsensusState>,
    pub consensus_state_height_map: BTreeMap<Height, C::ConsensusState>,
}

impl<C: ClientType> Default for Storage<C> {
    fn default() -> Self {
        Self {
            client_state: None,
            consensus_state: None,
            consensus_state_height_map: BTreeMap::new(),
        }
    }
}

impl<C: ClientType> Storage<C> {
    pub fn get_heights(&self) -> Vec<Height> {
        self.consensus_state_height_map.keys().cloned().collect()
    }

    pub fn get_adjacent_height(
        &self,
        current: &Height,
        direction: Direction,
    ) -> Option<C::ConsensusState> {
        match direction {
            Direction::Next => {
                let mut it = self
                    .consensus_state_height_map
                    .range((Bound::Included(current), Bound::Unbounded));
                it.next().map(|(_, s)| s.to_owned())
            }
            Direction::Previous => {
                let mut it = self
                    .consensus_state_height_map
                    .range((Bound::Unbounded, Bound::Included(current)));
                it.next_back().map(|(_, s)| s.to_owned())
            }
        }
    }
}
