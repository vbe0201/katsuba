use std::{env, fs, path::PathBuf};

use clap::{Args, Subcommand};
use kobold_client_sig::PrivateKey;
use kobold_utils::anyhow;

const PRIVATE_KEY_ENV: &str = "KOBOLD_CLIENTSIG_PRIVATE_KEY";

#[derive(Debug, Args)]
pub struct ClientSig {
    #[clap(subcommand)]
    command: ClientSigCommand,

    /// Path to a file containing a PKCS#1-encoded RSA private key in PEM format.
    ///
    /// This is required for all subcommands related to client signatures. If the
    /// argument is not provided, kobold will try to find such a file through the
    /// `KOBOLD_CLIENTSIG_PRIVATE_KEY` environment variable.
    ///
    /// If none of these paths lead to a valid private key, kobold will error out.
    #[clap(short, long)]
    private_key: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum ClientSigCommand {
    /// Gets the base64-encoded argument to pass to the game client's `-CS` CLI flag.
    ///
    /// This will be necessary for obtaining a `ClientSig.bin` file.
    Arg,

    /// Decrypts a given client signature file.
    Decrypt {
        /// Path to the signature file to decrypt.
        path: PathBuf,

        /// Optional path to an output file for the decrypted signature.
        #[clap(short, long, default_value = "ClientSig.dec.bin")]
        output: PathBuf,
    },
}

pub fn process(cs: ClientSig) -> anyhow::Result<()> {
    // Attempt to find a private key path from all supported sources.
    let private_key_path = cs
        .private_key
        .or_else(|| env::var_os(PRIVATE_KEY_ENV).map(PathBuf::from))
        .ok_or_else(|| anyhow::anyhow!("no private key file specified"))?;

    // Read the private key data and try to parse it.
    let private_key = fs::read_to_string(private_key_path)?;
    let private_key = PrivateKey::new(&private_key)?;

    // Execute the selected subcommand.
    match cs.command {
        ClientSigCommand::Arg => {
            let arg = private_key.make_access_key()?;
            println!("{arg}");
        }

        ClientSigCommand::Decrypt { path, output } => {
            let signature = fs::read(path)?;
            let decrypted_signature = private_key.decrypt_sig(&signature)?;
            fs::write(output, decrypted_signature)?;
        }
    }

    Ok(())
}
