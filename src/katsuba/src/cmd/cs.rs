use std::{fs, path::PathBuf};

use clap::{Args, Subcommand};
use eyre::Context;
use katsuba_client_sig::PrivateKey;

use super::Command;

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
    #[clap(short, long, env = "KATSUBA_CLIENTSIG_PRIVATE_KEY")]
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
        #[clap(short, long, default_value = "ClientSig.dec.bin")]
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
