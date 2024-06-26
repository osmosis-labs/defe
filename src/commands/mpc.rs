use dialoguer::{Confirm, Input, Select};
use hex::decode;
use serde::{Deserialize, Serialize};
use sharks::{Share, Sharks};
use std::fs;
use std::io;
use threshold_crypto::ff::Field;
use threshold_crypto::ff::PrimeField;
use threshold_crypto::{Fr, FrRepr, PublicKey, PublicKeySet, SecretKey, SecretKeySet, PK_SIZE};

#[derive(Serialize, Deserialize)]
struct KeyShares {
    public_key_set: PublicKeySet,
    encrypted_shares: Vec<Vec<u8>>,
}

pub fn run() -> io::Result<()> {
    println!("Welcome to the MPC (Multi-Party Computation) module!");

    loop {
        let selections = vec!["Shamir Secret Sharing", "Threshold Validation", "Exit"];
        let selection = Select::new()
            .with_prompt("Choose an operation:")
            .items(&selections)
            .interact()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        match selection {
            0 => shamir()?,
            1 => threshval()?,
            2 => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn get_threshold_and_total() -> io::Result<(usize, usize)> {
    let threshold = Input::<usize>::new()
        .with_prompt("Enter the threshold number (minimum number of shares required to reconstruct the secret)")
        .default(3)
        .interact()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let total_shares = Input::<usize>::new()
        .with_prompt("Enter the total number of shares to create")
        .default(5)
        .validate_with(|input: &usize| -> Result<(), &str> {
            if *input < threshold {
                Err("Total shares must be greater than or equal to the threshold")
            } else {
                Ok(())
            }
        })
        .interact()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    Ok((threshold, total_shares))
}

use rand::thread_rng;

fn shamir() -> io::Result<()> {
    println!("Starting Shamir Secret Sharing process...");

    let (threshold, total_shares) = get_threshold_and_total()?;

    // Generate secret key set
    let mut rng = thread_rng();
    let secret_key_set = SecretKeySet::random(threshold - 1, &mut rng);

    // Create Shamir's Secret Sharing scheme
    let sharks = Sharks(threshold as u8);
    let secret_key_share = secret_key_set.secret_key_share(0);
    let secret_key = secret_key_share.reveal();
    let secret_bytes = hex::decode(secret_key).expect("Failed to decode secret key");
    let shares = sharks
        .dealer(&secret_bytes)
        .take(total_shares)
        .collect::<Vec<_>>();

    // Collect public keys from users
    let mut public_keys = Vec::new();
    for i in 0..total_shares {
        let public_key: String = Input::new()
            .with_prompt(format!("Enter public key {} (hex-encoded)", i + 1))
            .interact()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        let public_key_bytes: [u8; PK_SIZE] = decode(public_key)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?
            .try_into()
            .map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid public key length")
            })?;

        public_keys.push(
            PublicKey::from_bytes(&public_key_bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?,
        );
    }

    // Encrypt shares
    let encrypted_shares: Vec<Vec<u8>> = shares
        .iter()
        .zip(public_keys.iter())
        .map(|(share, pub_key)| {
            let ciphertext = pub_key.encrypt(&Vec::from(share));
            bincode::serialize(&ciphertext).expect("Failed to serialize ciphertext")
        })
        .collect();

    // Save shares
    let key_shares = KeyShares {
        public_key_set: secret_key_set.public_keys(),
        encrypted_shares,
    };

    let serialized = serde_json::to_string(&key_shares)?;
    fs::write("key_shares.json", serialized)?;

    println!("Shamir Secret Sharing completed. Shares saved to 'key_shares.json'.");
    Ok(())
}

fn threshval() -> io::Result<()> {
    println!("Starting Threshold Validation process...");

    // Load shares
    let serialized = fs::read_to_string("key_shares.json")?;
    let key_shares: KeyShares = serde_json::from_str(&serialized)?;

    let threshold = Input::<usize>::new()
        .with_prompt("Enter the threshold number (minimum number of shares required to reconstruct the secret)")
        .default(3)
        .interact()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    // Collect shares from users
    let mut collected_shares = Vec::new();
    loop {
        let share: String = Input::new()
            .with_prompt(format!(
                "Enter share {} (hex-encoded, or press Enter to finish)",
                collected_shares.len() + 1
            ))
            .allow_empty(true)
            .interact()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        if share.is_empty() {
            if collected_shares.len() >= threshold {
                break;
            } else {
                println!(
                    "You need at least {} shares. Please continue entering shares.",
                    threshold
                );
                continue;
            }
        }

        let share_bytes = decode(&share)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
        collected_shares.push(
            Share::try_from(share_bytes.as_slice())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?,
        );

        if collected_shares.len() >= threshold
            && Confirm::new()
                .with_prompt("Do you want to proceed with reconstruction?")
                .interact()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
        {
            break;
        }
    }

    // Reconstruct the secret
    let sharks = Sharks(threshold as u8);
    let recovered_secret = sharks
        .recover(&collected_shares)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

    // Convert the recovered secret to Fr
    let mut recovered_secret_fr = Fr::zero();
    for (i, byte) in recovered_secret.iter().enumerate() {
        let mut repr = FrRepr::default();
        repr.as_mut()[0] = *byte as u64;
        let mut tmp = Fr::from_repr(repr)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
        for _ in 0..i {
            let mut base_repr = FrRepr::default();
            base_repr.as_mut()[0] = 256;
            let base = Fr::from_repr(base_repr)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
            tmp.mul_assign(&base);
        }
        recovered_secret_fr.add_assign(&tmp);
    }

    let mut recovered_secret_fr_mut = recovered_secret_fr;
    let recovered_key = SecretKey::from_mut(&mut recovered_secret_fr_mut);

    // Verify the secret against the public key set
    if key_shares.public_key_set.public_key() == recovered_key.public_key() {
        println!("Threshold validation successful!");
    } else {
        println!("Threshold validation failed!");
    }

    Ok(())
}
