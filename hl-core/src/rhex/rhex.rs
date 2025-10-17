use crate::{
    rhex::{
        context::Context,
        intent::Intent,
        record_types,
        signature::{SigType, Signature},
    },
    time::clock::GTClock,
};
use serde::{Deserialize, Serialize};

// Enable serde_with macros in Cargo.toml:
// serde_with = { version = "3", features = ["macros"] }

#[serde_with::serde_as] // must be before derive
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rhex {
    // Encode fixed arrays as CBOR byte strings
    #[serde_as(as = "serde_with::Bytes")]
    pub magic: [u8; 6],

    pub intent: Intent,
    pub context: Context,
    pub signatures: Vec<Signature>,

    // Option<[u8;32]> as optional bytes
    #[serde_as(as = "Option<serde_with::Bytes>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_hash: Option<[u8; 32]>,
}

#[derive(Debug, PartialEq)]
pub enum RhexStatus {
    Finalized,
    InvalidPreviousHash,
    InvalidScope,
    InvalidAuthorPK,
    InvalidUsherPK,
    InvalidRecordType,
    InvalidData,
    InvalidAt,
    InvalidSpacial,
    InvalidSignature,
    AuthorSigned,
    UsherSigned,
    QuorumSigned(u16),
}

impl Rhex {
    pub fn new() -> Self {
        Self {
            magic: *b"RHEX\x00\x00",
            intent: Intent {
                previous_hash: None,
                scope: String::new(),
                nonce: String::new(),
                author_pk: [0u8; 32],
                usher_pk: [0u8; 32],
                record_type: String::new(),
                data: serde_json::Value::Null,
            },
            context: Context {
                at: 0,
                x: None,
                y: None,
                z: None,
                refer: None,
            },
            signatures: Vec::new(),
            current_hash: None,
        }
    }

    pub fn from_cbor(cbor: &[u8]) -> anyhow::Result<Self> {
        // &[u8] doesn’t impl Read; &mut &[u8] does
        let mut slice = cbor;
        Ok(ciborium::de::from_reader(&mut slice)?)
        // or: Ok(ciborium::de::from_reader(&mut std::io::Cursor::new(cbor))?)
    }

    pub fn into_cbor(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();
        ciborium::ser::into_writer(self, &mut buf)?;
        Ok(buf)
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(&self)?)
    }

    /// Preimage for the author (intent only)
    pub fn author_hash(&self) -> anyhow::Result<[u8; 32]> {
        let mut hasher = blake3::Hasher::new();
        ciborium::ser::into_writer(&self.intent, &mut hasher)?; // Hasher: Write
        Ok(*hasher.finalize().as_bytes())
    }

    /// Add context to the current Rhex
    pub fn add_context(&mut self, spacial: &Option<(f64, f64, f64, String)>) -> anyhow::Result<()> {
        let clock = GTClock::new(0);
        self.context.at = clock.now_micromarks_u64();
        if let Some((x, y, z, refer)) = spacial {
            self.context.x = Some(*x);
            self.context.y = Some(*y);
            self.context.z = Some(*z);
            self.context.refer = Some(refer.clone());
        };
        Ok(())
    }

    /// Preimage for usher (author sig + context)
    pub fn usher_hash(&self, author_sig: &Signature) -> anyhow::Result<[u8; 32]> {
        let mut hasher = blake3::Hasher::new();
        // Encode signature as CBOR bytes, not array-of-u8
        ciborium::ser::into_writer(serde_bytes::Bytes::new(&author_sig.sig), &mut hasher)?;
        ciborium::ser::into_writer(&self.context, &mut hasher)?;
        Ok(*hasher.finalize().as_bytes())
    }

    /// Preimage for quorum (author sig + usher sig)
    pub fn quorum_hash(
        &self,
        author_sig: &Signature,
        usher_sig: &Signature,
    ) -> anyhow::Result<[u8; 32]> {
        let mut hasher = blake3::Hasher::new();
        ciborium::ser::into_writer(serde_bytes::Bytes::new(&author_sig.sig), &mut hasher)?;
        ciborium::ser::into_writer(serde_bytes::Bytes::new(&usher_sig.sig), &mut hasher)?;
        Ok(*hasher.finalize().as_bytes())
    }

    /// Finalize: intent + context + signatures (in canonical order)
    pub fn finalize(&mut self) -> anyhow::Result<()> {
        self.sort_signatures()?;
        let current_hash = self.generate_current_hash()?;
        self.current_hash = Some(current_hash);
        Ok(())
    }

    pub fn generate_current_hash(&self) -> Result<[u8; 32], anyhow::Error> {
        let mut hasher = blake3::Hasher::new();
        ciborium::ser::into_writer(&self.intent, &mut hasher)?;
        ciborium::ser::into_writer(&self.context, &mut hasher)?;
        ciborium::ser::into_writer(&self.signatures, &mut hasher)?;
        Ok(*hasher.finalize().as_bytes())
    }

    pub fn sort_signatures(&mut self) -> anyhow::Result<()> {
        self.signatures.sort_by(|a, b| {
            // Ensure SigType implements Ord, or compare ranks
            a.sig_type
                .cmp(&b.sig_type)
                .then_with(|| a.public_key.as_slice().cmp(b.public_key.as_slice()))
                .then_with(|| a.sig.as_slice().cmp(b.sig.as_slice()))
        });
        Ok(())
    }

    pub fn status(&self) -> RhexStatus {
        // TODO: check status of previous_hash, see if it matches the
        // previous record or not

        // TODO: still haven't figured what kind of check for scope we
        // need to do here

        if self.intent.author_pk == [0u8; 32] {
            return RhexStatus::InvalidAuthorPK;
        }
        if self.intent.usher_pk == [0u8; 32] {
            return RhexStatus::InvalidUsherPK;
        }

        let valid = record_types::is_valid_record_type(&self.intent.record_type);
        if !valid {
            return RhexStatus::InvalidRecordType;
        }

        // AI just wants to check if JSON is null, but really, it can
        // be for certain records so we need to come up with something
        // better

        let have_x = self.context.x.is_some();
        let have_y = self.context.y.is_some();
        let have_z = self.context.z.is_some();
        // If you want empty string to count as “absent”, use map(|s| !s.is_empty()).
        // If not, just .is_some().
        let have_refer = self.context.refer.is_some();

        let valid = match (have_x, have_y, have_z, have_refer) {
            (true, true, true, true) => RhexStatus::Finalized,
            (false, false, false, false) => RhexStatus::Finalized,
            _ => {
                return RhexStatus::InvalidSpacial;
            }
        };
        if valid != RhexStatus::Finalized {
            return valid;
        }

        if self.signatures.len() == 0 {
            return RhexStatus::InvalidSignature;
        }

        if self.signatures.len() == 1 {
            if self.signatures[0].sig_type == SigType::Author {
                return RhexStatus::AuthorSigned;
            }
        }

        if self.signatures.len() == 2 {
            if self.signatures[0].sig_type == SigType::Author
                && self.signatures[1].sig_type == SigType::Usher
            {
                return RhexStatus::UsherSigned;
            }
        }
        if self.current_hash.is_some() {
            return RhexStatus::Finalized;
        } else {
            RhexStatus::QuorumSigned((self.signatures.len() - 2) as u16)
        }
    }
}
