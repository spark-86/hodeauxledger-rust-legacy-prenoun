use std::{fs, io::Write};

use hl_core::{b64::b64::from_base64_to_32, to_base64};

use crate::argv::B64Args;

pub fn base64convert(b64_args: &B64Args) -> Result<(), anyhow::Error> {
    let input = &b64_args.input;
    let output = &b64_args.output;
    if input.starts_with("base64:") {
        // Input is base64
        let input = &input[7..];
        let input = from_base64_to_32(input)?;
        let mut output = fs::File::create(output)?;
        output.write_all(&input)?;
    } else {
        if input.starts_with("hex:") {
            // Input is hex
            let input = &input[4..];
            let input = hex::decode(input)?;
            let mut output = fs::File::create(output)?;
            output.write_all(&input)?;
            return Ok(());
        } else {
            // Output is base64
            let input = fs::read(input).expect("Failed to read file");
            let output = to_base64(&input);
            println!("{}", output)
        }
    }
    Ok(())
}
