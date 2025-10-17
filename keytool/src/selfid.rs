use std::fs;

use crate::argv::SelfIDArgs;

pub fn selfid(selfid_args: &SelfIDArgs) -> Result<(), anyhow::Error> {
    let key_data = fs::read(&selfid_args.keyfile)?;
    let key = hl_core::Key::from_bytes(key_data[..32].try_into().unwrap());
    let self_id = key.compute_self_id()?;
    let formatted = format!(
        "SelfID: {}-{}-{}-{}",
        &self_id[0..4],
        &self_id[4..8],
        &self_id[8..12],
        &self_id[12..16]
    );
    println!("{}", formatted);
    Ok(())
}
