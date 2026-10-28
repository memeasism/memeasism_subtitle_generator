#![windows_subsystem = "windows"]
mod gen_text;
mod parse_time;
use arboard::Clipboard;
use rc_cell::RcCell;
use rodio::Source;
use slint::ComponentHandle;
use std::io::Cursor;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
const SUCCESS_SOUND: &[u8] = include_bytes!("../sounds/success.flac");
const ERROR_SOUND: &[u8] = include_bytes!("../sounds/error.flac");
slint::include_modules!();
fn main() {
    let output: RcCell<String> = RcCell::new(String::new());
    let output_write: RcCell<String> = output.clone();
    let output_read: RcCell<String> = output.clone();
    let app = GUI::new().unwrap();
    let app_weak = app.as_weak();
    let audio_handle =
        Rc::new(rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream"));
    app.on_inputs_changed(move || {
        let app_output = app_weak.upgrade().unwrap();
        let word_by_word: bool = app_output.get_word_by_word();
        let pop_out: bool = app_output.get_pop_out();
        let delay: u16 = app_output.get_word_delay().parse().unwrap_or(2);
        let fps: f32 = app_output.get_fps_content().parse().unwrap_or(60.0);
        let text: String = app_output.get_text_content().to_string();
        let length: String = app_output.get_length_content().to_string();
        let duration_ms: u32 = match parse_time::parse(length, fps) {
            Some(ms) => ms,
            None => {
                app_output.set_output_text("Invalid Timecode".into());
                return;
            }
        };
        *output_write.borrow_mut() =
            gen_text::generate(word_by_word, pop_out, delay, text, duration_ms);
        app_output.set_output_text(output_write.borrow().clone().into());
    });
    let current_audio_handle = Rc::clone(&audio_handle);
    let app_weak = app.as_weak();
    app.on_copy_subtitle(move || {
        let app_output = app_weak.unwrap();
        let content: String = output_read.borrow().clone();

        if let Ok(mut clipboard) = Clipboard::new() {
            if let Err(result) = clipboard.set_text(content.clone()) {
                app_output.set_output_text(format!("ERROR COPYING: {}", result).into());
                let audio = Cursor::new(ERROR_SOUND);
                let source = rodio::Decoder::try_from(audio).unwrap().amplify(0.075);
                current_audio_handle.mixer().add(source);
                thread::sleep(Duration::from_secs(3));
                app_output.set_output_text(content.clone().into());
                return;
            };

            let audio = Cursor::new(SUCCESS_SOUND);
            let source = rodio::Decoder::try_from(audio).unwrap().amplify(0.075);
            current_audio_handle.mixer().add(source);
        }
    });
    app.run().unwrap();
}
