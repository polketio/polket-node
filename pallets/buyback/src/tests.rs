// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

use codec::Encode;
use hex_literal::hex;
use p256::{
	ecdsa::{
		signature::{DigestVerifier, Signature, Signer, Verifier},
		Signature as OSignature, SigningKey, VerifyingKey,
	},
	elliptic_curve::{sec1::ToEncodedPoint, PublicKey},
	NistP256,
};
use sha2::{Digest, Sha256, Sha512};

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
	let x = &hex!["03be679ee49518a6b3471403538dd7b0514b8357fc52c6e8cf44c560685028df11"];
	// let signing_key = SigningKey::from_bytes(x).unwrap();
	let pubkey = VerifyingKey::from_sec1_bytes(x).unwrap();

	let msg = &hex!["010101010101010101010101010101010101010101010101"];
	// let msg = Sha256::Hash::hash(&msg.as_ref());

	// create a Sha256 object
	let sha_msg = Sha256::new_with_prefix(msg);

	// println!("message = {}", hex::encode(message));

	let nonce: u32 = 1111;
	println!("nonce 大端= {}", hex::encode(nonce.to_be_bytes()));
	println!("nonce 小端= {}", hex::encode(nonce.to_le_bytes()));
	println!("nonce encode = {}", hex::encode(nonce.encode()));

	// let msg = &hex!["5b8b4d29020ea5b1bc427c40a0cab2bf944be057ec482110f1d12b68008cd286"];
	let sig = Signature::from_bytes(&hex!["02e404621bc572723a498c97592360d071cf6f434a53e856ce5daff5c426c9d96c2183a0f5642cba77da726914b5909313d00973d6b0f60e157393564063b0dd"]).unwrap();

	// let verifying_key = signing_key.verifying_key();
	assert!(pubkey.verify(msg.as_ref(), &sig).is_ok());
	assert!(pubkey.verify_digest(sha_msg, &sig).is_ok());
}
