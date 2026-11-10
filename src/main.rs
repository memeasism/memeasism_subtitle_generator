//#![windows_subsystem = "windows"]
mod gen_text;
mod parse_time;
use arboard::Clipboard;
use maudio::data_source::sources::decoder::DecoderBuilder;
use maudio::engine::Engine;
use rc_cell::RcCell;
use slint::ComponentHandle;
use std::time::Duration;
#[cfg(target_os = "windows")]
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
//embed the audio files into the binary
const SUCCESS_SOUND: &[u8] = include_bytes!("../sounds/success.flac");
const ERROR_SOUND: &[u8] = include_bytes!("../sounds/error.flac");
slint::include_modules!();
fn main() {
    #[cfg(target_os = "windows")]
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
    let output: RcCell<String> = RcCell::new(String::new());
    let output_write: RcCell<String> = output.clone(); /*rccell because slint requires moving, wich sucks for changing output
    output is also updated as a variable instead of using ui just in case ui fails to update*/
    let output_read: RcCell<String> = output.clone();
    let reset_error: RcCell<u32> = RcCell::new(0); //counter that allows overwriting error output in gui, so the 3 seconds isnt forced if a success or ui update happens
    let reset_error_write: RcCell<u32> = reset_error.clone();
    let reset_error_read: RcCell<u32> = reset_error.clone();
    let app: GUI = GUI::new().unwrap();
    let app_weak: slint::Weak<GUI> = app.as_weak();
    let engine: Engine = Engine::new().unwrap();
    let _engine_volume: Result<(), maudio::MaudioError> = engine.set_volume(0.075);
    //decode sound effects
    let success_decoder = DecoderBuilder::new_f32()
        .from_memory(SUCCESS_SOUND)
        .unwrap();
    let error_decoder = DecoderBuilder::new_f32().from_memory(ERROR_SOUND).unwrap();
    //cache the decoded sounds for better runtime performance
    let success_sound: maudio::sound::Sound =
        engine.new_sound_from_source(&success_decoder).unwrap();
    let error_sound: maudio::sound::Sound = engine.new_sound_from_source(&error_decoder).unwrap();
    let mut clipboard: Result<Clipboard, String> =
        Clipboard::new().map_err(|error| error.to_string()); //load clipboard once, or return an error string
    app.on_inputs_changed(move || {
        let reset_error_count: u32 = (*reset_error_write.borrow()).wrapping_add(1);
        *reset_error_write.borrow_mut() = reset_error_count;
        let app_output: GUI = app_weak.upgrade().unwrap();
        let word_by_word: bool = app_output.get_word_by_word();
        let pop_out: bool = app_output.get_pop_out();
        let center: bool = app_output.get_center_text();
        let delay: f32 = app_output.get_word_delay();
        let fps: f32 = app_output.get_fps_content().parse().unwrap_or(60.0); //grabs all the needed info
        let scale: f32 = app_output.get_scale();
        let rotation: f32 = app_output.get_rotation();
        let text: &str = &app_output.get_text_content();
        let length: &str = &app_output.get_length_content();
        let duration_ms: u32 = parse_time::parse(length, fps);
        *output_write.borrow_mut() = gen_text::generate(
            word_by_word,
            pop_out,
            center,
            delay,
            scale,
            rotation,
            text,
            duration_ms,
        ); //updates the output variable
        app_output.set_output_text(output_write.borrow().as_str().into()); //updates output ui
    });
    let app_weak: slint::Weak<GUI> = app.as_weak();
    app.on_copy_subtitle(move || {
        let Some(app_output) = app_weak.upgrade() else {
            return;
        };
        let content: String = output_read.borrow().clone();
        if clipboard.is_err() {
            clipboard = Clipboard::new().map_err(|error| error.to_string()); //attempt to initialize clipboard if it errored
        }
        let clipboard_result: Result<(), String> = match clipboard.as_mut() {
            //get clipboard result or error string
            Ok(clipboard) => clipboard
                .set_text(content.as_str())
                .map_err(|e| e.to_string()),
            Err(error) => Err(format!("clipboard unavailable ({})", error)),
        };
        if let Err(result) = clipboard_result {
            //runs actions if error and returns early
            app_output.set_output_text(format!("ERROR COPYING: {}", result).into());
            error_sound.play_sound().unwrap();
            let snapshot_current_error_count: u32 = (*reset_error_read.borrow()).wrapping_add(1);
            *reset_error_read.borrow_mut() = snapshot_current_error_count;
            let reset_error_timer: RcCell<u32> = reset_error_read.clone();
            let app_weak_timer: slint::Weak<GUI> = app_weak.clone();
            let content_reset: String = content;
            slint::Timer::single_shot(Duration::from_secs(3), move || {
                if *reset_error_timer.borrow() != snapshot_current_error_count {
                    return;
                }
                if let Some(app) = app_weak_timer.upgrade() {
                    app.set_output_text(content_reset.into());
                }
            });

            return;
        }
        success_sound.play_sound().unwrap();
        let reset_error_count = (*reset_error_read.borrow()).wrapping_add(1);
        *reset_error_read.borrow_mut() = reset_error_count;
        app_output.set_output_text(content.into());
    });
    app.run().unwrap();
}
