// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

use bitcoin_hashes::{sha256 as Sha256, Hash};
use codec::Encode;
use hex_literal::hex;
use p256::{
	ecdsa::{
		signature::{Signature, Signer, Verifier},
		Signature as OSignature, SigningKey, VerifyingKey,
	},
	elliptic_curve::{sec1::ToEncodedPoint, PublicKey},
	NistP256,
};

#[test]
fn calculate() {
	let max32 = u32::MAX;
	let max64 = u64::MAX;
	let max128 = u128::MAX;
	println!("max32 = {}", max32);
	println!("max64 = {}", max64);
	println!("max128 = {}", max128);
}

#[test]
fn generate_secp256r1_pk() {
	// Signing
	let x = &hex!["c9afa9d845ba75166b5c215767b1d6934e50c3db36e89b127b8a622b120f6721"];
	let signing_key = SigningKey::from_bytes(x).unwrap(); // Serialize with `::to_bytes()`
	let message = b"ECDSA proves knowledge of a secret number in the context of a single message";
	// let sha_msg = Sha256::Hash::hash(&message.as_ref()).as_ref();
	let signature = signing_key.sign(message);
	println!("signature = {}", hex::encode(signature.as_bytes()));
	//b1c4b565eb28a6753325dd81ba50ac4bc485934f819d6fdfc6d337a0a8f2dd68f37ba1e630ed5fe97b65b6f25dd02543742ccf69074f6a62e23673605f61ca97
	println!("message = {}", hex::encode(message));
	//45434453412070726f766573206b6e6f776c65646765206f66206120736563726574206e756d62657220696e2074686520636f6e74657874206f6620612073696e676c65206d657373616765

	let verifying_key = signing_key.verifying_key(); // Serialize with `::to_encoded_point()`
	let publickey: PublicKey<NistP256> = verifying_key.into();
	let encoded_point = publickey.to_encoded_point(true);
	println!("pks = {}", hex::encode(encoded_point));
	//0360fed4ba255a9d31c961eb74c6356d68c049b8923b61fa6ce669622e60f29fb6
	assert!(verifying_key.verify(message, &signature).is_ok());
}

#[test]
fn verify_secp256r1_pk() {
	let x = &hex!["0360fed4ba255a9d31c961eb74c6356d68c049b8923b61fa6ce669622e60f29fb6"];
	// let signing_key = SigningKey::from_bytes(x).unwrap();
	let pubkey = VerifyingKey::from_sec1_bytes(x).unwrap();

	let msg = [1u8; 24];
	// let sha_msg = Sha256::Hash::hash(&msg.as_ref()).encode();

	let sha_msg = &hex!["45434453412070726f766573206b6e6f776c65646765206f66206120736563726574206e756d62657220696e2074686520636f6e74657874206f6620612073696e676c65206d657373616765"];
	let sig = Signature::from_bytes(&hex!["b1c4b565eb28a6753325dd81ba50ac4bc485934f819d6fdfc6d337a0a8f2dd68f37ba1e630ed5fe97b65b6f25dd02543742ccf69074f6a62e23673605f61ca97"]).unwrap();

	// let verifying_key = signing_key.verifying_key();
	assert!(pubkey.verify(sha_msg.as_ref(), &sig).is_ok());
}
