use std::{path::PathBuf, str::FromStr};

use anyhow::{Error, bail};
use ed25519_dalek::{Signer, SigningKey};
use hl_core::{
    Context,
    rhex::{
        rhex::RhexStatus,
        signature::{SigType, Signature},
    },
    time::clock::GTClock,
};
use hl_io::{
    fs::{authority as authority_store, rhex::FileSource},
    sink::RhexSink,
    source::RhexSource,
};

use crate::argv::SignArgs;

pub fn sign(sign_args: &SignArgs) -> Result<(), Error> {
    let keypath = &sign_args.key.keyfile;
    let password = &sign_args.key.password;
    let hot = &sign_args.key.hot;
    let sig_type = sign_args.sig_type.as_str();
    let input = &sign_args.input;
    let output = &sign_args.output;

    let password = if *hot { None } else { password.clone() };
    let mut rhex = FileSource::new(PathBuf::from_str(input)?)?;
    let rhex = rhex.next()?;

    let rhex = if rhex.is_none() {
        None
    } else {
        Some(rhex.unwrap())
    };

    if rhex.is_none() {
        bail!("Invalid Rhex")
    }

    let mut rhex = rhex.unwrap();
    let status = rhex.status();
    match status {
        RhexStatus::InvalidSignature => {
            // We are awaiting author signature
            match sig_type {
                "author" => {
                    // Process signing
                    if rhex.signatures.len() == 0 {
                    } else {
                        bail!("Author already signed")
                    }
                }
                "usher" => {
                    bail!("Author not signed")
                }
                "quorum" => {
                    bail!("Author not signed")
                }
                _ => {
                    bail!("Invalid signature type")
                }
            }
        }
        RhexStatus::AuthorSigned => {
            // We are awaiting usher signature
            match sig_type {
                "author" => {
                    bail!("Author already signed")
                }
                "usher" => {
                    if rhex.signatures.len() > 1 {
                        bail!("Usher already signed")
                    } else {
                        if rhex.signatures.len() == 1 {
                            if rhex.signatures[0].sig_type != SigType::Author {
                                bail!("Author not signed")
                            }
                        }
                    }
                }
                "quorum" => {
                    bail!("Usher not signed")
                }
                _ => {
                    bail!("Invalid signature type")
                }
            }
        }
        RhexStatus::UsherSigned => {
            // We are awaiting quorum signature
            match sig_type {
                "author" => {
                    bail!("Author not signed")
                }
                "usher" => {
                    bail!("Usher already signed")
                }
                "quorum" => {
                    if rhex.signatures.len() > 2 {
                        bail!("Quorum already signed")
                    } else {
                        if rhex.signatures.len() == 2 {
                            if rhex.signatures[0].sig_type != SigType::Author {
                                bail!("Author not signed")
                            }
                            if rhex.signatures[1].sig_type != SigType::Usher {
                                bail!("Usher not signed")
                            }
                        }
                    }
                }
                _ => {
                    bail!("Invalid signature type")
                }
            }
        }
        RhexStatus::QuorumSigned(_) => {
            // We have X signatures now
            match sig_type {
                "author" => {
                    bail!("Author already signed")
                }
                "usher" => {
                    bail!("Usher already signed")
                }
                "quorum" => {
                    bail!("Quorum already signed")
                }
                _ => {
                    bail!("Invalid signature type")
                }
            }
        }
        _ => {
            bail!("Invalid Rhex status {:?}", status)
        }
    }

    let sk = if *hot {
        let pb = PathBuf::from_str(keypath)?;
        authority_store::load_key_hot(&pb)?
    } else {
        let pb = PathBuf::from_str(keypath)?;
        authority_store::load_key(&pb, &password.unwrap())?
    };
    let signing_key = SigningKey::from_bytes(&sk);
    match sig_type {
        "author" => {
            // Process signing
            println!("signing author...");
            let hash = rhex.author_hash()?;
            let signature = signing_key.sign(&hash);
            let new_sig = Signature {
                sig_type: SigType::Author,
                public_key: signing_key.verifying_key().to_bytes(),
                sig: signature.to_bytes(),
            };
            rhex.signatures.push(new_sig);
        }
        "usher" => {
            // Update context
            let time = GTClock::now_micromarks_u64(&GTClock::new(0));
            let context = Context::from_at(time);
            rhex.context = context;
            // Process signing
            let author_sig = &rhex
                .signatures
                .iter()
                .find(|s| s.sig_type == SigType::Author)
                .unwrap();
            let hash = rhex.usher_hash(&author_sig)?;
            let signature = signing_key.sign(&hash);
            let new_sig = Signature {
                sig_type: SigType::Usher,
                public_key: signing_key.verifying_key().to_bytes(),
                sig: signature.to_bytes(),
            };
            rhex.signatures.push(new_sig);
        }
        "quorum" => {
            // Process signing
            let author_sig = &rhex
                .signatures
                .iter()
                .find(|s| s.sig_type == SigType::Author)
                .unwrap();
            let usher_sig = &rhex
                .signatures
                .iter()
                .find(|s| s.sig_type == SigType::Usher)
                .unwrap();
            let hash = rhex.quorum_hash(&author_sig, &usher_sig)?;
            let signature = signing_key.sign(&hash);
            let new_sig = Signature {
                sig_type: SigType::Quorum,
                public_key: signing_key.verifying_key().to_bytes(),
                sig: signature.to_bytes(),
            };
            rhex.signatures.push(new_sig);
        }
        _ => {
            bail!("Invalid signature type")
        }
    }

    let pb = PathBuf::from_str(output)?;
    let mut dir_sink = hl_io::fs::rhex::DirSink::new(pb);
    let status = dir_sink.send(&rhex.clone());
    match status {
        Ok(_) => {
            println!("Wrote rhex to {}", output);
        }
        Err(e) => {
            bail!("Error writing rhex: {}", e);
        }
    }
    Ok(())
}
