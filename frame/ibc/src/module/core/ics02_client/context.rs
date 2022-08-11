use crate::*;
use alloc::string::ToString;
use core::str::FromStr;
use log::{error, info, trace, warn};

use crate::context::Context;
use ibc::{
    core::{
        ics02_client::{
            client_consensus::AnyConsensusState,
            client_state::AnyClientState,
            client_type::ClientType,
            context::{ClientKeeper, ClientReader},
            error::Error as Ics02Error,
        },
        ics24_host::{
            identifier::ClientId,
            path::{ClientConsensusStatePath, ClientStatePath, ClientTypePath},
        },
    },
    timestamp::Timestamp,
    Height,
};

impl<T: Config> ClientReader for Context<T> {
    fn client_type(&self, client_id: &ClientId) -> Result<ClientType, Ics02Error> {
        trace!(target:"runtime::pallet-ibc","in client : [client_type]");

        let client_type_path = ClientTypePath(client_id.clone()).to_string().as_bytes().to_vec();
        if <Clients<T>>::contains_key(client_type_path.clone()) {
            let data = <Clients<T>>::get(client_type_path);
            let mut data: &[u8] = &data;
            let data = Vec::<u8>::decode(&mut data).unwrap();
            let data = String::from_utf8(data).unwrap();
            let client_type = ClientType::from_str(&data)
                .map_err(|e| Ics02Error::unknown_client_type(e.to_string()))?;
            Ok(client_type)
        } else {
            trace!(target:"runtime::pallet-ibc","in client : [client_type] >> read client_type is None");
            Err(Ics02Error::client_not_found(client_id.clone()))
        }
    }

    fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, Ics02Error> {
        trace!(target:"runtime::pallet-ibc","in client : [client_state]");

        let client_state_path = ClientStatePath(client_id.clone()).to_string().as_bytes().to_vec();

        if <ClientStates<T>>::contains_key(&client_state_path) {
            let data = <ClientStates<T>>::get(&client_state_path);
            let result = AnyClientState::decode_vec(&*data).unwrap();
            trace!(target:"runtime::pallet-ibc","in client : [client_state] >> any client_state: {:?}", result);

            Ok(result)
        } else {
            trace!(target:"runtime::pallet-ibc","in client : [client_state] >> read any client state is None");
            Err(Ics02Error::client_not_found(client_id.clone()))
        }
    }

    fn consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<AnyConsensusState, Ics02Error> {
        trace!(target:"runtime::pallet-ibc",
			"in client : [consensus_state]"
		);

        // search key
        let client_consensus_state_path = ClientConsensusStatePath {
            client_id: client_id.clone(),
            epoch: height.revision_number(),
            height: height.revision_height(),
        }
            .to_string()
            .as_bytes()
            .to_vec();

        if <ConsensusStates<T>>::contains_key(client_consensus_state_path.clone()) {
            let values = <ConsensusStates<T>>::get(client_consensus_state_path.clone());
            let any_consensus_state =
                AnyConsensusState::decode_vec(&*values).unwrap();
            trace!(target:"runtime::pallet-ibc",
				"in client : [consensus_state] >> any consensus state = {:?}",
				any_consensus_state
			);
            Ok(any_consensus_state)
        } else {
            Err(Ics02Error::consensus_state_not_found(client_id.clone(), height))
        }
    }

    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Option<AnyConsensusState>, Ics02Error> {
        trace!(target:"runtime::pallet-ibc","in client : [next_consensus_state]");

        // search key
        let client_consensus_state_path = ClientConsensusStatePath {
            client_id: client_id.clone(),
            epoch: height.revision_number(),
            height: height.revision_height(),
        }
            .to_string()
            .as_bytes()
            .to_vec();

        if <ConsensusStates<T>>::contains_key(client_consensus_state_path.clone()) {
            let values = <ConsensusStates<T>>::get(client_consensus_state_path.clone());
            let any_consensus_state =
                AnyConsensusState::decode_vec(&*values).unwrap();
            trace!(target:"runtime::pallet-ibc",
				"in client : [next_consensus_state] >> any consensus state = {:?}",
				any_consensus_state
			);
            Ok(Some(any_consensus_state))
        } else {
            trace!(target:"runtime::pallet-ibc",
				"in client : [next_consensus_state] >> any consensus state is not contains at ConsensusStates : client_id = {:?}, height = {:?}", client_id, height
			);
            Err(Ics02Error::consensus_state_not_found(client_id.clone(), height))
        }
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Option<AnyConsensusState>, Ics02Error> {
        trace!(target:"runtime::pallet-ibc","in client : [next_consensus_state]");

        // search key
        let client_consensus_state_path = ClientConsensusStatePath {
            client_id: client_id.clone(),
            epoch: height.revision_number(),
            height: height.revision_height(),
        }
            .to_string()
            .as_bytes()
            .to_vec();

        if <ConsensusStates<T>>::contains_key(client_consensus_state_path.clone()) {
            let values = <ConsensusStates<T>>::get(client_consensus_state_path.clone());
            let any_consensus_state =
                AnyConsensusState::decode_vec(&*values).unwrap();
            trace!(target:"runtime::pallet-ibc",
				"in client : [prev_consensus_state] >> any consensus state = {:?}",
				any_consensus_state
			);
            Ok(Some(any_consensus_state))
        } else {
            trace!(target:"runtime::pallet-ibc",
				"in client : [prev_consensus_state] >> any consensus state is not contains at ConsensusStates : client_id = {:?}, height = {:?}", client_id, height
			);
            Err(Ics02Error::consensus_state_not_found(client_id.clone(), height))
        }
    }

    fn host_height(&self) -> Height {
        trace!(target:"runtime::pallet-ibc","in client : [host_height]");
        let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
        let current_height: u64 = block_number.parse().unwrap_or_default();

        trace!(target:"runtime::pallet-ibc",
			"in channel: [host_height] >> host_height = {:?}",
			Height::new(REVISION_NUMBER, current_height)
		);
        Height::new(REVISION_NUMBER, current_height).unwrap()
    }

    fn host_consensus_state(&self, _height: Height) -> Result<AnyConsensusState, Ics02Error> {
        trace!(target:"runtime::pallet-ibc","in client : [consensus_state]");

       todo!()
    }

    fn pending_host_consensus_state(&self) -> Result<AnyConsensusState, Ics02Error> {
        trace!(target:"runtime::pallet-ibc","in client: [pending_host_consensus_state]");

        todo!()
    }

    fn client_counter(&self) -> Result<u64, Ics02Error> {
        trace!(target:"runtime::pallet-ibc","in client : [client_counter]");

        Ok(<ClientCounter<T>>::get())
    }
}

impl<T: Config> ClientKeeper for Context<T> {
    fn store_client_type(
        &mut self,
        client_id: ClientId,
        client_type: ClientType,
    ) -> Result<(), Ics02Error> {
        info!("in client : [store_client_type]");

        let client_type_path = ClientTypePath(client_id.clone()).to_string().as_bytes().to_vec();
        let client_type = client_type.as_str().encode();
        <Clients<T>>::insert(client_type_path, client_type);
        Ok(())
    }

    fn store_client_state(
        &mut self,
        client_id: ClientId,
        client_state: AnyClientState,
    ) -> Result<(), Ics02Error> {
        trace!(target:"runtime::pallet-ibc","in client : [store_client_state]");

        let client_state_path = ClientStatePath(client_id.clone()).to_string().as_bytes().to_vec();

        let data = client_state.encode_vec().unwrap();
        // store client states key-value
        <ClientStates<T>>::insert(client_state_path.clone(), data);

        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        client_id: ClientId,
        height: Height,
        consensus_state: AnyConsensusState,
    ) -> Result<(), Ics02Error> {
        trace!(target:"runtime::pallet-ibc","in client : [store_consensus_state]");

        // store key
        let client_consensus_state_path = ClientConsensusStatePath {
            client_id: client_id.clone(),
            epoch: height.revision_number(),
            height: height.revision_height(),
        }
            .to_string()
            .as_bytes()
            .to_vec();

        // store value
        let consensus_state = consensus_state.encode_vec().unwrap();
        // store client_consensus_state path as key, consensus_state as value
        <ConsensusStates<T>>::insert(client_consensus_state_path, consensus_state);

        Ok(())
    }

    fn increase_client_counter(&mut self) {
        info!("in client : [increase_client_counter]");

        let ret = <ClientCounter<T>>::try_mutate(|val| -> Result<(), Ics02Error> {
            let new = val.checked_add(1).unwrap();
            *val = new;
            Ok(())
        });
    }

    fn store_update_time(
        &mut self,
        client_id: ClientId,
        height: Height,
        timestamp: Timestamp,
    ) -> Result<(), Ics02Error> {
        trace!(target:"runtime::pallet-ibc","in client: [store_update_time]");

        let encode_timestamp = serde_json::to_string(&timestamp)
            .unwrap()
            .as_bytes()
            .to_vec();

        <ClientProcessedTimes<T>>::insert(
            client_id.as_bytes(),
            height.encode_vec().unwrap(),
            encode_timestamp,
        );

        Ok(())
    }

    fn store_update_height(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_height: Height,
    ) -> Result<(), Ics02Error> {
        trace!(target:"runtime::pallet-ibc","in client: [store_update_height]");

        <ClientProcessedHeights<T>>::insert(
            client_id.as_bytes(),
            height.encode_vec().unwrap(),
            host_height.encode_vec().unwrap(),
        );

        Ok(())
    }
}
