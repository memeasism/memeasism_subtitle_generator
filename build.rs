fn main() {
    slint_build::compile("gui/main.slint").unwrap();
    #[cfg(windows)]
    {
        winres::WindowsResource::new()
            .set_icon("icons/icon.ico") // Path relative to crate root
            .compile()
            .unwrap();
    }
}
