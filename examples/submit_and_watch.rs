// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of substrate-subxt.
//
// subxt is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// subxt is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-subxt.  If not, see <http://www.gnu.org/licenses/>.

use futures::future::Future;
use substrate_subxt::{balances, system::System, DefaultNodeRuntime as Runtime};
use sp_keyring::AccountKeyring;

type AccountId = <Runtime as System>::AccountId;
type Balance = <Runtime as balances::Balances>::Balance;

fn main() {
    env_logger::init();
    let signer = AccountKeyring::Alice.pair();

    let dest = AccountKeyring::Bob.to_account_id();

    let fut = substrate_subxt::ClientBuilder::<Runtime>::new()
        .build()
        .and_then(|cli| cli.xt(signer, None))
        .and_then(move |xt| {
            xt.watch()
                .events_decoder(|decoder| {
                    // for any primitive event with no type size registered
                    decoder.register_type_size::<(u64, u64)>("IdentificationTuple")
                })
                .submit(balances::transfer::<Runtime>(dest.clone().into(), 10_000))
        });

    let mut rt = tokio::runtime::Runtime::new().unwrap();
    match rt.block_on(fut) {
        Ok(extrinsic_success) => {
            match extrinsic_success.find_event::<(AccountId, AccountId, Balance, Balance)>("Balances", "Transfer") {
                Some(Ok((_from, _to, value, _fees))) =>
                    println!("Balance transfer success: value: {:?}", value),
                Some(Err(err)) => println!("Failed to decode code hash: {}", err),
                None => println!("Failed to find Contracts::CodeStored Event"),
            }
        },
        Err(err) => println!("Error: {}", err)
    }
}
