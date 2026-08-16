//! Verify a Tauri updater `.sig` file against its installer and the updater
//! public key, using the exact verification path the client app uses
//! (`minisign-verify`, mirroring `tauri-plugin-updater::verify_signature`).
//!
//! Usage:
//! ```
//! cargo run --example verify_updater_sig -- <installer> <installer.sig> <public_key>
//! ```
//!
//! Example:
//! ```
//! cargo run --example verify_updater_sig -- ^
//!   "target/release/bundle/nsis/PRO MAX OS_2.6.3_x64-setup.exe" ^
//!   "target/release/bundle/nsis/PRO MAX OS_2.6.3_x64-setup.exe.sig" ^
//!   "$env:USERPROFILE\.tauri\promax-os.key.pub"
//! ```
//!
//! Exits 0 when the signature is valid, 1 otherwise.

use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("usage: verify_updater_sig <installer> <installer.sig> <public_key>");
        std::process::exit(2);
    }

    let data = match fs::read(&args[1]) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("failed to read installer: {e}");
            std::process::exit(2);
        }
    };

    let sig = match read_sig(&args[2]) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to parse signature: {e}");
            std::process::exit(2);
        }
    };

    let pubkey = match read_pubkey(&args[3]) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("failed to parse public key: {e}");
            std::process::exit(2);
        }
    };

    match pubkey.verify(&data, &sig, true) {
        Ok(()) => {
            println!("SIG_VERIFY_OK");
        }
        Err(e) => {
            eprintln!("SIG_VERIFY_FAIL: {e}");
            std::process::exit(1);
        }
    }
}

fn base64_decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s)
}

fn read_sig(path: &str) -> Result<minisign_verify::Signature, Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(path)?;
    let decoded = String::from_utf8(base64_decode(raw.trim())?)?;
    Ok(minisign_verify::Signature::decode(&decoded)?)
}

fn read_pubkey(path: &str) -> Result<minisign_verify::PublicKey, Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(path)?;
    let decoded = String::from_utf8(base64_decode(raw.trim())?)?;
    Ok(minisign_verify::PublicKey::decode(&decoded)?)
}
