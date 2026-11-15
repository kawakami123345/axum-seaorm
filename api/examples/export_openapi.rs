fn main() -> Result<(), Box<dyn std::error::Error>> {
    let document = api::openapi_document().to_pretty_json()?;
    if let Some(output) = std::env::args_os().nth(1) {
        let output = std::path::PathBuf::from(output);
        let temporary = output.with_extension(format!("{}.tmp", std::process::id()));
        std::fs::write(&temporary, format!("{document}\n"))?;
        if let Err(error) = std::fs::rename(&temporary, &output) {
            let _ = std::fs::remove_file(temporary);
            return Err(error.into());
        }
    } else {
        println!("{document}");
    }
    Ok(())
}
