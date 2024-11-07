pub mod dsl;
mod generate;
pub mod instructions;

use std::io;
use std::path::Path;
use std::process::Command;

/// Generate the assembler `file` containing the DSL-defined functions.
///
/// # Panics
///
/// This function panics if we cannot update the file or if the file has no
/// parent path.
pub fn generate<P: AsRef<Path>>(file: P) {
    let file = file.as_ref();
    eprintln!("Generating {}", file.display());
    let mut f = generate::Formatter::new();
    generate::generate(&mut f, &instructions::list());
    f.write(file).unwrap();
    rustfmt(file).unwrap();
}

/// Use the installed `rustfmt` binary to format the generated code.
fn rustfmt(file: &Path) -> io::Result<()> {
    let status = Command::new("rustfmt").arg(file).status()?;
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("`rustfmt` exited with status {status}"),
        ));
    }
    Ok(())
}
