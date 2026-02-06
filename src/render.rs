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
        .spawn()?;

    let stdin = render_proc
        .stdin
        .as_mut()
        .ok_or(report!("Process is missing stdin"))?;
    write!(stdin, "{}", source)?;

    let rendered = render_proc.wait_with_output()?;

    if !rendered.status.success() {
        let report = report!("Rendering failed!")
            .attach(String::from_utf8(rendered.stdout)?)
            .attach(String::from_utf8(rendered.stderr)?);
        bail!(report);
    }

    std::fs::write(out_file, rendered.stdout)?;

    Ok(())
}
