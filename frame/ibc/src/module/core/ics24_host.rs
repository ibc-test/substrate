// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{alloc::string::ToString, from_channel_id_to_vec, Config, Event, REVISION_NUMBER};
use alloc::string::String;
use ibc::{
	core::{
		ics02_client::{client_type::ClientType as IbcClientType, height::Height as IbcHeight},
		ics04_channel::packet::{Packet as IbcPacket, Sequence as IbcSequence},
		ics24_host::{
			error::ValidationError,
			identifier::{
				ChainId as IbcChainId, ChannelId as IbcChannelId, ClientId as IbcClientId,
				ConnectionId as IbcConnectionId, PortId as IbcPortId,
			},
		},
	},
	timestamp::Timestamp as IbcTimestamp,
};
use sp_std::{str::FromStr, vec::Vec};

use codec::{Decode, Encode};
use scale_info::TypeInfo;

use sp_runtime::RuntimeDebug;

use flex_error::{define_error, DisplayOnly, TraceError};
use ibc::core::ics04_channel::timeout::TimeoutHeight;
use tendermint_proto::Error as TendermintError;

define_error! {
	#[derive(Debug, PartialEq, Eq)]
	Error {
		InvalidFromUtf8
			[DisplayOnly<alloc::string::FromUtf8Error>]
			| _ | { "invalid from utf8 error" },
		InvalidDecode
			[DisplayOnly<codec::Error>]
			| _ | { "invalid decode error" },
		ParseTimestampFailed
			[DisplayOnly<ibc::timestamp::ParseTimestampError>]
			| _ | { "invalid parse timestamp error" },
		ValidationFailed
			[DisplayOnly<ValidationError>]
			| _ | { "invalid validation error"},
		InvalidChainId
			[DisplayOnly<core::convert::Infallible>]
			|_| { "invalid chain id error" },
	}
}

/// ibc-rs' `PortId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct PortId(pub Vec<u8>);

impl From<IbcPortId> for PortId {
	fn from(value: IbcPortId) -> Self {
		let value = value.as_str().as_bytes().to_vec();
		Self(value)
	}
}

impl From<PortId> for IbcPortId {
	fn from(value: PortId) -> Self {
		let value = String::from_utf8(value.0).expect("convert Never faild");
		IbcPortId::from_str(&value).expect("convert Never faild")
	}
}

/// ibc-rs' `ChannelId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ChannelId(pub Vec<u8>);

impl From<IbcChannelId> for ChannelId {
	fn from(value: IbcChannelId) -> Self {
		let value = from_channel_id_to_vec(value);
		Self(value)
	}
}

impl From<ChannelId> for IbcChannelId {
	fn from(value: ChannelId) -> Self {
		let value = String::from_utf8(value.0).expect("convert Never faild");
		Self::from_str(&value).expect("convert Never faild")
	}
}

/// ibc-rs' `Height` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Height {
	/// Previously known as "epoch"
	pub revision_number: u64,

	/// The height of a block
	pub revision_height: u64,
}

impl From<IbcHeight> for Height {
	fn from(ibc_height: IbcHeight) -> Self {
		Height::new(ibc_height.revision_number(), ibc_height.revision_height())
	}
}

impl From<Height> for IbcHeight {
	fn from(height: Height) -> Self {
		IbcHeight::new(REVISION_NUMBER, height.revision_height)
			.expect("Contruct IbcHeight Never faild")
	}
}

impl Height {
	pub fn new(revision_number: u64, revision_height: u64) -> Self {
		Self { revision_number, revision_height }
	}
}

/// ibc-rs' `ClientType` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum ClientType {
	Tendermint,
	Grandpa,
}

impl From<IbcClientType> for ClientType {
	fn from(value: IbcClientType) -> Self {
		match value {
			IbcClientType::Tendermint => ClientType::Tendermint,
			_ => unreachable!(),
		}
	}
}

impl ClientType {
	pub fn to_ibc_client_type(self) -> IbcClientType {
		match self {
			ClientType::Tendermint => IbcClientType::Tendermint,
			_ => unreachable!(),
		}
	}
}

/// ibc-rs' `ClientId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ClientId(pub Vec<u8>);

impl From<IbcClientId> for ClientId {
	fn from(value: IbcClientId) -> Self {
		let value = value.as_str().as_bytes().to_vec();
		Self(value)
	}
}

impl From<ClientId> for IbcClientId {
	fn from(value: ClientId) -> Self {
		let value = String::from_utf8(value.0).expect("convert Never faild");
		IbcClientId::from_str(&value).expect("convert Never faild")
	}
}

/// ibc-rs' `ConnectionId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ConnectionId(pub Vec<u8>);

impl From<IbcConnectionId> for ConnectionId {
	fn from(value: IbcConnectionId) -> Self {
		let value = value.as_str().as_bytes().to_vec();
		Self(value)
	}
}

impl From<ConnectionId> for IbcConnectionId {
	fn from(value: ConnectionId) -> Self {
		let value = String::from_utf8(value.0).expect("convert Never faild");
		IbcConnectionId::from_str(&value).expect("convert Never faild")
	}
}

/// ibc-rs' `Timestamp` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Timestamp {
	pub time: Vec<u8>,
}

impl From<IbcTimestamp> for Timestamp {
	fn from(val: IbcTimestamp) -> Self {
		Self { time: val.nanoseconds().to_string().as_bytes().to_vec() }
	}
}

impl From<Timestamp> for IbcTimestamp {
	fn from(value: Timestamp) -> Self {
		let value = String::from_utf8(value.time).expect("convert Never faild");
		Self::from_str(&value).expect("convert Never faild")
	}
}

/// ibc-rs' `Sequence` representation in substrate

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Sequence(u64);

impl From<IbcSequence> for Sequence {
	fn from(val: IbcSequence) -> Self {
		Self(u64::from(val))
	}
}

impl From<Sequence> for IbcSequence {
	fn from(val: Sequence) -> Self {
		IbcSequence::from(val.0)
	}
}

/// ibc-rs' `Packet` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Packet {
	pub sequence: Sequence,
	pub source_port: PortId,
	pub source_channel: ChannelId,
	pub destination_port: PortId,
	pub destination_channel: ChannelId,
	pub data: Vec<u8>,
	pub timeout_height: Height,
	pub timeout_timestamp: Timestamp,
}

impl From<IbcPacket> for Packet {
	fn from(val: IbcPacket) -> Self {
		Self {
			sequence: val.sequence.into(),
			source_port: val.source_port.into(),
			source_channel: val.source_channel.into(),
			destination_port: val.destination_port.into(),
			destination_channel: val.destination_channel.into(),
			data: val.data,
			timeout_height: match val.timeout_height {
				TimeoutHeight::Never => Height::new(REVISION_NUMBER, u64::MAX),
				TimeoutHeight::At(value) => value.into(),
			},
			timeout_timestamp: val.timeout_timestamp.into(),
		}
	}
}

impl From<Packet> for IbcPacket {
	fn from(value: Packet) -> Self {
		Self {
			sequence: value.sequence.into(),
			source_port: value.source_port.into(),
			source_channel: value.source_channel.into(),
			destination_port: value.destination_port.into(),
			destination_channel: value.destination_channel.into(),
			data: value.data,
			timeout_height: TimeoutHeight::At(value.timeout_height.into()),
			timeout_timestamp: value.timeout_timestamp.into(),
		}
	}
}
