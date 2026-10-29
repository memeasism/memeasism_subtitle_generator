#![windows_subsystem = "windows"]
mod gen_text;
mod parse_time;
use arboard::Clipboard;
use rc_cell::RcCell;
use rodio::Source;
use slint::ComponentHandle;
use std::io::Cursor;
use std::rc::Rc;
use std::time::Duration;
const SUCCESS_SOUND: &[u8] = include_bytes!("../sounds/success.flac");
const ERROR_SOUND: &[u8] = include_bytes!("../sounds/error.flac");
slint::include_modules!();
fn main() {
    let output: RcCell<String> = RcCell::new(String::new());
    let output_write: RcCell<String> = output.clone(); /*rccell because slint requires moving, wich sucks for changing output
    output is also updated as a variable instead of using ui just in case ui fails to update*/
    let output_read: RcCell<String> = output.clone();
    let app = GUI::new().unwrap();
    let app_weak = app.as_weak();
    let audio_handle: Option<Rc<rodio::MixerDeviceSink>> =
        rodio::DeviceSinkBuilder::open_default_sink()
            .ok() //use rc for audio handle incase of a rare scenaro of a device with no audio output
            .map(Rc::new);
    app.on_inputs_changed(move || {
        let app_output = app_weak.upgrade().unwrap();
        let word_by_word: bool = app_output.get_word_by_word();
        let pop_out: bool = app_output.get_pop_out();
        let delay: u32 = app_output.get_word_delay().parse().unwrap_or(2);
        let fps: f32 = app_output.get_fps_content().parse().unwrap_or(60.0); //grabs all the needed info
        let text: String = app_output.get_text_content().to_string();
        let length: String = app_output.get_length_content().to_string();
        let duration_ms: u32 = parse_time::parse(length, fps);
        *output_write.borrow_mut() =
            gen_text::generate(word_by_word, pop_out, delay, text, duration_ms); //updates the output variable
        app_output.set_output_text(output_write.borrow().as_str().into()); //updates output ui
    });
    let current_audio_handle = audio_handle.as_ref().map(Rc::clone); //sets cloned audio handle
    let app_weak = app.as_weak();
    app.on_copy_subtitle(move || {
        let app_output = app_weak.unwrap();
        let content: String = output_read.borrow().clone();

        if let Ok(mut clipboard) = Clipboard::new() {
            if let Err(result) = clipboard.set_text(content.clone()) {
                //runs actions if error and returns early
                app_output.set_output_text(format!("ERROR COPYING: {}", result).into());
                if let Some(handle) = &current_audio_handle {
                    if let Ok(source) = rodio::Decoder::try_from(Cursor::new(ERROR_SOUND)) {
                        handle.mixer().add(source.amplify(0.075)); //does a bunch of error checking to ensure we can use audio with no crash
                    }
                }
                let app_weak_reset = app_weak.clone();
                let content_reset = content.clone();
                slint::Timer::single_shot(Duration::from_secs(3), move || {
                    if let Some(app_output) = app_weak_reset.upgrade() {
                        app_output.set_output_text(content_reset.clone().into());
                    }
                });
                return;
            };
            if let Some(handle) = &current_audio_handle {
                if let Ok(source) = rodio::Decoder::try_from(Cursor::new(SUCCESS_SOUND)) {
                    handle.mixer().add(source.amplify(0.075));
                }
            }
        }
    });
    app.run().unwrap();
}
