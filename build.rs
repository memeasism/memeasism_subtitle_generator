#[cfg(windows)]
use std::io;
use winres::WindowsResource;
fn main() -> io::Result<()> {
    {
        WindowsResource::new()
            .set_icon("icons/icon.ico") // Path relative to crate root
            .compile()?;
    }
    slint_build::compile("gui/main.slint").unwrap();
    Ok(())
}
