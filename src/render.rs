use rootcause::prelude::*;
use std::{io::Write, path::PathBuf, process::Stdio};

pub fn render_plantuml(
    out_file: &PathBuf,
    target_format: &str,
    source: &str,
) -> Result<(), Report> {
    log::debug!("Rendering to file {:?}", out_file);

    // Ensure the containing directory
    std::fs::create_dir_all(out_file.parent().ok_or(report!("Invalid output file"))?)?;

    let mut render_proc = std::process::Command::new("plantuml")
        .args(["--format", target_format])
        .arg("--pipe")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = render_proc
        .stdin
        .as_mut()
        .ok_or(report!("Process is missing stdin"))?;
    write!(stdin, "{}", source)?;

    let rendered = render_proc.wait_with_output()?;

    if !rendered.status.success() {
        log::warn!("Rendering producted an error!");
    }

    std::fs::write(out_file, rendered.stdout)?;

    Ok(())
}
