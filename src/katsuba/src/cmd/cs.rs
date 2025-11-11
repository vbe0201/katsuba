use std::{
    env::{self, VarError},
    ffi::CStr,
    fs,
    path::PathBuf,
};

use libc::{c_char};

use clap::{Args, Subcommand};
use eyre::Context;
use katsuba_client_sig::PrivateKey;

use super::Command;

pub const KATSUBA_CLIENTSIG_PRIVATE_KEY: &str = "KATSUBA_CLIENTSIG_PRIVATE_KEY";
pub const DEFAULT_OUTPUT_FILE: &str = "ClientSig.dec.bin";

/// Subcommand for working with Client Signatures.
#[derive(Debug, Args)]
pub struct ClientSig {
    #[clap(subcommand)]
    command: ClientSigCommand,

    /// Path to a file containing a PKCS#1-encoded RSA private key in
    /// PEM format.
    ///
    /// A valid key is required for all subcommands related to client
    /// signatures.
    ///
    /// If no argument is provided, Katsuba will try to find a file path
    /// under the `KATSUBA_CLIENTSIG_PRIVATE_KEY` environment variable.
    #[clap(short, long, env = KATSUBA_CLIENTSIG_PRIVATE_KEY)]
    private_key: PathBuf,
}

#[derive(Debug, Subcommand)]
enum ClientSigCommand {
    /// Gets the base64-encoded argument to pass to the game client's
    /// `-CS` CLI flag.
    ///
    /// This will be necessary for obtaining a `ClientSig.bin` file.
    Arg,

    /// Decrypts a given Client Signature file.
    Decrypt {
        /// Path to the signature file to decrypt.
        path: PathBuf,

        /// Optional path to an output file for the decrypted signature.
        ///
        /// Defaults to `ClientSig.dec.bin` in the working directory.
        #[clap(short, long, default_value = DEFAULT_OUTPUT_FILE)]
        output: PathBuf,
    },
}

impl Command for ClientSig {
    fn handle(self) -> eyre::Result<()> {
        match self.command {
            ClientSigCommand::Arg => {
                arg(&self.private_key)
            }

            ClientSigCommand::Decrypt { path, output } => {
                decrypt(&self.private_key, path, output)
            }
        }
    }
}

fn get_private_key(private_key_file: &PathBuf) -> eyre::Result<PrivateKey> {
    let private_key = fs::read_to_string(&private_key_file).with_context(|| {
        format!(
            "failed to read private key from '{}'",
            private_key_file.display()
        )
    })?;
    PrivateKey::new(&private_key).context("failed to parse given private key")
}

fn arg(private_key_file: &PathBuf) -> eyre::Result<()> {
    let private_key = get_private_key(&private_key_file)?;
    let arg = private_key.make_access_key();
    println!("{arg}");
    Ok(())
}

fn decrypt(private_key_file: &PathBuf, path: PathBuf, output: PathBuf) -> eyre::Result<()> {
    let private_key = get_private_key(&private_key_file)?;

    let signature = fs::read(&path)
        .with_context(|| format!("failed to read file '{}'", path.display()))?;

    let decrypted_signature = private_key
        .decrypt_sig(&signature)
        .context("received invalid Client Signature file")?;

    fs::write(output, decrypted_signature)?;
    Ok(())
}

fn get_private_key_path_from_c(private_key_file: *const c_char) -> eyre::Result<PathBuf, VarError> {
    let private_key = if !private_key_file.is_null() {
        match unsafe { CStr::from_ptr(private_key_file) }.to_str() {
            Ok(rust_str) => rust_str,
            Err(_) => "",
        }
    } else {
        ""
    };

    // No private key file was given, try the environment variable instead
    if private_key == "" {
        match env::var(KATSUBA_CLIENTSIG_PRIVATE_KEY) {
            Ok(value) => Ok(PathBuf::from(value)),
            Err(error) => Err(error), // No value found for environment variable
        }
    } else {
        Ok(PathBuf::from(private_key))
    }
}

#[no_mangle]
pub extern "C" fn cs_arg(private_key_file: *const c_char) -> bool {
    let private_key = match get_private_key_path_from_c(private_key_file) {
        Ok(key) => key,
        Err(_) => return false,
    };

    arg(&private_key).is_ok()
}

#[no_mangle]
pub extern "C" fn cs_decrypt(private_key_file: *const c_char, path: *const c_char, output: *const c_char) -> bool {
    let private_key = match get_private_key_path_from_c(private_key_file) {
        Ok(key) => key,
        Err(_) => return false,
    };

    let rust_path = if path.is_null() {
        return false
    } else {
        match unsafe { CStr::from_ptr(path) }.to_str() {
            Ok(rust_str) => PathBuf::from(rust_str),
            Err(_) => return false,
        }
    };

    let rust_output = if output.is_null() {
        PathBuf::from(DEFAULT_OUTPUT_FILE)
    } else {
        match unsafe { CStr::from_ptr(output) }.to_str() {
            Ok(rust_str) => PathBuf::from(rust_str),
            Err(_) => PathBuf::from(DEFAULT_OUTPUT_FILE),
        }
    };

    decrypt(&private_key, rust_path, rust_output).is_ok()
}
