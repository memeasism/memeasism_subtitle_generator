fn main() {
    slint_build::compile("src/main.slint").unwrap(); //build the UI
    #[cfg(windows)] //run only on Windows
    {
        winres::WindowsResource::new()
            .set_icon("icons/icon.ico")
            .compile() //set exe icon
            .unwrap();
    }
}
