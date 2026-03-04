use crate::{sink::RhexSink, source::RhexSource};
use anyhow::{Context, Result, bail};
use hl_core::Rhex;
use hl_core::b32::b32::to_base32_crockford;
use std::fs;
use std::path::PathBuf; // optional helper

// helper (your existing Crockford encoder)
fn hash_to_name(hash: &[u8; 32]) -> String {
    format!("{}.rhex", to_base32_crockford(hash))
}

enum WalkState {
    Init,           // before yielding genesis
    Next([u8; 32]), // expect file named by this previous_hash
    Done,
}

pub struct DirSource {
    root: PathBuf,
    state: WalkState,
    // cache genesis so we can yield it first
    genesis: Option<Rhex>,
}

impl DirSource {
    /// Create a new DirSource from the given directory path.
    pub fn new(root: PathBuf) -> Result<Self> {
        if !root.exists() {
            bail!("scope dir does not exist: {}", root.display());
        }
        let genesis_path = root.join("genesis.rhex");
        if !genesis_path.exists() {
            bail!("genesis.rhex missing in {}", root.display());
        }

        let bytes = fs::read(&genesis_path)
            .with_context(|| format!("reading {}", genesis_path.display()))?;
        let genesis = Rhex::from_cbor(&bytes)
            .with_context(|| format!("decoding {}", genesis_path.display()))?;

        if genesis.current_hash.is_none() {
            bail!("genesis.rhex has no current_hash");
        }

        Ok(Self {
            root,
            state: WalkState::Init, // <-- start in Init so we yield genesis first
            genesis: Some(genesis),
        })
    }

    fn load_by_prev(&self, prev: &[u8; 32]) -> Result<Option<Rhex>> {
        let name = hash_to_name(prev);
        let path = self.root.join(&name.to_ascii_lowercase());
        if !path.exists() {
            return Ok(None); // end of chain
        }
        let bytes = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        let r = Rhex::from_cbor(&bytes).with_context(|| format!("decoding {}", path.display()))?;
        // sanity: linkage
        if r.intent.previous_hash != Some(*prev) {
            bail!("chain break at {}: previous_hash mismatch", path.display());
        }
        Ok(Some(r))
    }
}

impl RhexSource for DirSource {
    fn next(&mut self) -> Result<Option<Rhex>> {
        match std::mem::replace(&mut self.state, WalkState::Done) {
            WalkState::Init => {
                // yield genesis first
                if let Some(genesis) = self.genesis.take() {
                    // after yielding genesis, continue from its current_hash
                    let next = genesis.current_hash.unwrap_or([0u8; 32]);
                    self.state = WalkState::Next(next);
                    return Ok(Some(genesis));
                }
                self.state = WalkState::Done;
                Ok(None)
            }
            WalkState::Next(prev) => {
                match self.load_by_prev(&prev)? {
                    Some(r) => {
                        if let Some(ch) = r.current_hash {
                            self.state = WalkState::Next(ch);
                        } else {
                            // leaf without current_hash ends the walk
                            self.state = WalkState::Done;
                        }
                        Ok(Some(r))
                    }
                    None => {
                        self.state = WalkState::Done;
                        Ok(None)
                    }
                }
            }
            WalkState::Done => Ok(None),
        }
    }
}

pub struct FileSource {
    path: PathBuf,
    done: bool,
}

impl FileSource {
    /// Create a new FileSource from the given file path.
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            bail!("file does not exist: {}", path.display());
        }
        Ok(Self { path, done: false })
    }
}

impl RhexSource for FileSource {
    fn next(&mut self) -> Result<Option<Rhex>> {
        if self.done {
            return Ok(None);
        }
        let bytes =
            fs::read(&self.path).with_context(|| format!("reading {}", self.path.display()))?;
        let r =
            Rhex::from_cbor(&bytes).with_context(|| format!("decoding {}", self.path.display()))?;
        self.done = true; // mark consumed
        Ok(Some(r)) // yield exactly once
    }
}
pub struct DirSink {
    root: PathBuf,
}

impl DirSink {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl RhexSink for DirSink {
    fn send(&mut self, r: &Rhex) -> Result<()> {
        let bytes = r.into_cbor()?;
        let name = if r.intent.previous_hash == None {
            "genesis.rhex".to_string()
        } else {
            format!(
                "{}.rhex",
                to_base32_crockford(r.intent.previous_hash.as_ref().unwrap()).to_ascii_lowercase()
            )
        };
        println!(
            "Writing {} bytes to {}/{}",
            bytes.len(),
            self.root.display(),
            name
        );
        fs::write(self.root.join(name), bytes)?;
        Ok(())
    }
}
