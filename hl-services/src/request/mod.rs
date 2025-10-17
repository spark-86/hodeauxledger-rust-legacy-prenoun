use hl_core::{
    Rhex, Signature,
    keymaster::keymaster::Keymaster,
    rhex::{intent::Intent, signature::SigType},
};
use hl_io::{net::net::Transport, screen::print::pretty_print};
use serde_json::json;
use std::{fs, time::Duration};
pub async fn request(
    host: &str,
    port: &str,
    scope: &str,
    record_types: &[&str],
    keyfile: &str,
) -> Result<Vec<Rhex>, anyhow::Error> {
    println!(
        "Requesting from {}:{} for scope {} with keyfile {}",
        host, port, scope, keyfile
    );
    let mut keymaster = Keymaster::new();
    let key_bytes = fs::read(keyfile)?;
    keymaster.load_keys(&vec![key_bytes.try_into().unwrap()])?;

    let author_key = keymaster.hot_keys.get(0).ok_or_else(|| {
        anyhow::anyhow!("No keys found in keyfile. At least one key is required.")
    })?;

    let intent = Intent {
        previous_hash: None, // This is a new request, so no previous hash
        scope: scope.to_string(),
        nonce: Intent::gen_nonce(),
        author_pk: author_key.public_key_bytes()?,
        usher_pk: author_key.public_key_bytes()?, // For now, usher is self
        record_type: "request:rhex".to_string(),
        data: json!({
            "record_types": record_types.to_vec()
        }),
    };

    let mut request_rhex = Rhex::new();
    request_rhex.intent = intent;

    // Sign with author key
    let author_sig = Signature {
        sig_type: SigType::Author,
        public_key: author_key.public_key_bytes()?,
        sig: author_key.sign(&request_rhex.author_hash()?)?,
    };
    request_rhex.signatures.push(author_sig.clone());

    pretty_print(&request_rhex)?;

    let mut transport = Transport::new();
    transport.connect(host, port).await?;
    transport.send_rhex(&request_rhex).await?;

    let mut count = 0;
    let mut out = Vec::new();
    loop {
        count += 1;
        if count > 10 {
            break;
        }
        let rhex_out = transport
            .recv_next_with_timeout(Duration::from_millis(1500))
            .await?;
        if rhex_out.is_none() {
            break;
        }
        let rhex_out = rhex_out.unwrap();
        out.push(rhex_out);
    }

    Ok(out)
}
