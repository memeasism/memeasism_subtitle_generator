fn main() {
    slint_build::compile("gui/main.slint").unwrap();
    #[cfg(windows)]
    {
        winres::WindowsResource::new()
            .set_icon("icons/icon.ico")
            .compile()
            .unwrap();
    }
}
