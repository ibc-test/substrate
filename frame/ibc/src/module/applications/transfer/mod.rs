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

pub mod channel;
pub mod transfer_handle_callback;

use crate::{context::Context, *};
use frame_support::traits::{
	fungibles::{Mutate, Transfer},
	ExistenceRequirement::AllowDeath,
};
use log::{error, trace};

use crate::utils::get_channel_escrow_address;
use ibc::{
	applications::transfer::{
		context::{BankKeeper, Ics20Context, Ics20Keeper, Ics20Reader},
		error::Error as Ics20Error,
		PrefixedCoin, PORT_ID_STR,
	},
	core::ics24_host::identifier::PortId,
	signer::Signer,
};
use sp_runtime::{
	traits::{CheckedConversion, IdentifyAccount, Verify},
	MultiSignature,
};

use transfer_handle_callback::TransferModule;

impl<T: Config> Ics20Keeper for TransferModule<T> {
	type AccountId = <Self as Ics20Context>::AccountId;
}

impl<T: Config> BankKeeper for TransferModule<T> {
	type AccountId = <Self as Ics20Context>::AccountId;

	fn send_coins(
		&mut self,
		from: &Self::AccountId,
		to: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), Ics20Error> {
		// TODO(davirain): trace_path now is private
		// let is_native_asset = amt.denom.trace_path().is_empty();
		let is_native_asset = true;
		match is_native_asset {
			// transfer native token
			true => {
				// TODO(davirain): amount now is private, and base_denom is private
				// let amount = amt.amount.as_u256().low_u128().checked_into().expect("Convert MUST
				// NOT Failed"); let ibc_token_name = amt.denom.base_denom().as_str().as_bytes();
				let amount = todo!();
				let ibc_token_name = &[1, 1, 2, 3];
				let native_token_name = T::NATIVE_TOKEN_NAME;

				// assert native token name equal want to send ibc token name
				assert_eq!(
					native_token_name, ibc_token_name,
					"send ibc token name is not native token name"
				);

				<T::Currency as Currency<T::AccountId>>::transfer(
					&from.clone().into_account(),
					&to.clone().into_account(),
					amount,
					AllowDeath,
				)
				.map_err(|error| {
					error!("❌ [send_coins] : Error: ({:?})", error);
					Ics20Error::invalid_token()
				})?;

				// add emit transfer native token event
				Pallet::<T>::deposit_event(Event::<T>::TransferNativeToken(
					from.clone(),
					to.clone(),
					amount,
				))
			},
			// transfer non-native token
			false => {
				// TODO(davirain): amount now is private, and base_denom is private
				// let amount = amt.amount.as_u256().low_u128().into();
				// let denom = amt.denom.base_denom().as_str();
				let amount = todo!();
				let denom = &[1, 1, 2, 3];
				// look cross chain asset have register in host chain
				match T::AssetIdByName::try_get_asset_id(denom) {
					Ok(token_id) => {
						<T::Assets as Transfer<T::AccountId>>::transfer(
							token_id.into(),
							&from.clone().into_account(),
							&to.clone().into_account(),
							amount,
							true,
						)
						.map_err(|error| {
							error!("❌ [send_coins] : Error: ({:?})", error);
							Ics20Error::invalid_token()
						})?;

						// add emit transfer no native token event
						Pallet::<T>::deposit_event(Event::<T>::TransferNoNativeToken(
							from.clone(),
							to.clone(),
							amount,
						));
					},
					Err(_error) => {
						error!("❌ [send_coins]: denom: ({:?})", denom);
						return Err(Ics20Error::invalid_token())
					},
				}
			},
		}

		Ok(())
	}

	fn mint_coins(
		&mut self,
		account: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), Ics20Error> {
		// TODO(davirain): amount now is private, and base_denom is private
		// let amount = amt.amount.as_u256().low_u128().into();
		// let denom = amt.denom.base_denom().as_str();
		let amount = todo!();
		let denom = &[1, 1, 2, 3];
		// look cross chain asset have register in host chain
		match T::AssetIdByName::try_get_asset_id(denom) {
			Ok(token_id) => {
				<T::Assets as Mutate<T::AccountId>>::mint_into(
					token_id.into(),
					&account.clone().into_account(),
					amount,
				)
				.map_err(|error| {
					error!("❌ [mint_coins] : Error: ({:?})", error);
					Ics20Error::invalid_token()
				})?;

				// add mint token event
				Pallet::<T>::deposit_event(Event::<T>::MintToken(
					token_id,
					account.clone(),
					amount,
				));
			},
			Err(_error) => {
				error!("❌ [mint_coins]: denom: ({:?})", denom);
				return Err(Ics20Error::invalid_token())
			},
		}
		Ok(())
	}

	fn burn_coins(
		&mut self,
		account: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), Ics20Error> {
		// TODO(davirain): amount now is private, and base_denom is private
		// let amount = amt.amount.as_u256().low_u128().into();
		// let denom = amt.denom.base_denom().as_str();
		let amount = todo!();
		let denom = &[1, 1, 2, 3];
		// look cross chain asset have register in host chain
		match T::AssetIdByName::try_get_asset_id(denom) {
			Ok(token_id) => {
				<T::Assets as Mutate<T::AccountId>>::burn_from(
					token_id.into(),
					&account.clone().into_account(),
					amount,
				)
				.map_err(|error| {
					error!("❌ [burn_coins] : Error: ({:?})", error);
					Ics20Error::invalid_token()
				})?;

				// add burn token event
				Pallet::<T>::deposit_event(Event::<T>::BurnToken(
					token_id,
					account.clone(),
					amount,
				));
			},
			Err(_error) => {
				error!("❌ [burn_coins]: denom: ({:?})", denom);
				return Err(Ics20Error::invalid_token())
			},
		}
		Ok(())
	}
}

impl<T: Config> Ics20Reader for TransferModule<T> {
	type AccountId = <Self as Ics20Context>::AccountId;

	fn get_port(&self) -> Result<PortId, Ics20Error> {
		PortId::from_str(PORT_ID_STR)
			.map_err(|e| Ics20Error::invalid_port_id(PORT_ID_STR.to_string(), e))
	}

	fn get_channel_escrow_address(
		&self,
		port_id: &PortId,
		channel_id: &IbcChannelId,
	) -> Result<Self::AccountId, Ics20Error> {
		get_channel_escrow_address(port_id, channel_id)?
			.try_into()
			.map_err(|_| Ics20Error::parse_account_failure())
	}

	fn is_send_enabled(&self) -> bool {
		// TODO(davirain), need according channelEnd def
		true
	}

	fn is_receive_enabled(&self) -> bool {
		// TODO(davirain), need according channelEnd def
		true
	}
}

impl<T: Config> Ics20Context for TransferModule<T> {
	type AccountId = <T as Config>::AccountIdConversion; // Need Setting Account TODO(davirian)
}

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

#[derive(Clone, Debug, PartialEq, TypeInfo, Encode, Decode)]
pub struct IbcAccount(AccountId);

impl IdentifyAccount for IbcAccount {
	type AccountId = AccountId;
	fn into_account(self) -> Self::AccountId {
		self.0
	}
}

impl TryFrom<Signer> for IbcAccount
where
	AccountId: From<[u8; 32]>,
{
	type Error = &'static str;

	/// Convert a signer to an IBC account.
	/// Only valid hex strings are supported for now.
	fn try_from(signer: Signer) -> Result<Self, Self::Error> {
		let acc_str = signer.as_ref();
		if acc_str.starts_with("0x") {
			match acc_str.strip_prefix("0x") {
				Some(hex_string) => TryInto::<[u8; 32]>::try_into(
					hex::decode(hex_string).map_err(|_| "Error decoding invalid hex string")?,
				)
				.map_err(|_| "Invalid account id hex string")
				.map(|acc| Self(acc.into())),
				_ => Err("Signer does not hold a valid hex string"),
			}
		}
		// Do SS58 decoding instead
		else {
			error!("Convert Signer ❌ : Failed! ");
			Err("invalid ibc address or substrate address")
		}
	}
}
