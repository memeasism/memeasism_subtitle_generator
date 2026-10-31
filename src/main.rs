#![windows_subsystem = "windows"]
mod gen_text;
mod parse_time;
use arboard::Clipboard;
use rc_cell::RcCell;
use rodio::{ChannelCount, Decoder, SampleRate, Source};
use slint::ComponentHandle;
use std::io::Cursor;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
const SUCCESS_SOUND: &[u8] = include_bytes!("../sounds/success.flac"); //embed the audio files into the binary
const ERROR_SOUND: &[u8] = include_bytes!("../sounds/error.flac");
slint::include_modules!();
struct Samples {
    data: Arc<[f32]>, //would use Rc, but rodio requires being able to send, which is in Arc, and not Rc
    channels: ChannelCount,
    sample_rate: SampleRate,
    position: usize,
}
impl Samples {
    fn new(data: Arc<[f32]>, channels: ChannelCount, sample_rate: SampleRate) -> Self {
        Self {
            data,
            channels,
            sample_rate,
            position: 0,
        }
    }
}
impl Iterator for Samples {
    //iterator read samples
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let sample: Option<f32> = self.data.get(self.position).copied();
        if sample.is_some() {
            self.position += 1;
        }
        sample
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.data.len() - self.position;
        (rem, Some(rem))
    }
}
impl Source for Samples {
    //get audio metadata
    fn current_span_len(&self) -> Option<usize> {
        Some(self.data.len() - self.position)
    }
    fn channels(&self) -> ChannelCount {
        self.channels
    }
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        let denom: u64 = (self.channels.get() as u64) * (self.sample_rate.get() as u64);
        if denom == 0 {
            return None;
        }
        Some(Duration::from_secs_f64(
            self.data.len() as f64 / denom as f64,
        ))
    }
}
struct DecodedSound {
    channels: ChannelCount,
    sample_rate: SampleRate,
    samples: Arc<[f32]>,
}
fn decode_sound(bytes: &'static [u8]) -> Option<DecodedSound> {
    //build a cached decoded sound
    let decoder: Decoder<Cursor<&[u8]>> = Decoder::try_from(Cursor::new(bytes)).ok()?;
    let channels: std::num::NonZero<u16> = decoder.channels();
    let sample_rate: std::num::NonZero<u32> = decoder.sample_rate();
    let samples: Arc<[f32]> = decoder.collect::<Vec<f32>>().into();
    Some(DecodedSound {
        channels,
        sample_rate,
        samples,
    })
}
fn main() {
    let output: RcCell<String> = RcCell::new(String::new());
    let output_write: RcCell<String> = output.clone(); /*rccell because slint requires moving, wich sucks for changing output
    output is also updated as a variable instead of using ui just in case ui fails to update*/
    let output_read: RcCell<String> = output.clone();
    let reset_error: RcCell<u32> = RcCell::new(0); //counter to overwrite error output in gui, so the 3 seconds isnt forced if a success happens
    let reset_error_write: RcCell<u32> = reset_error.clone();
    let reset_error_read: RcCell<u32> = reset_error.clone();
    let app: GUI = GUI::new().unwrap();
    let app_weak: slint::Weak<GUI> = app.as_weak();
    let success_sound: Option<DecodedSound> = decode_sound(SUCCESS_SOUND); //cache the decoded sound effects for better runtime performance
    let error_sound: Option<DecodedSound> = decode_sound(ERROR_SOUND);
    let audio_handle: Option<Rc<rodio::MixerDeviceSink>> =
        rodio::DeviceSinkBuilder::open_default_sink()
            .ok() //use rc for audio handle incase of a rare scenaro of a device with no audio output
            .map(Rc::new);
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
    let current_audio_handle: Option<Rc<rodio::MixerDeviceSink>> =
        audio_handle.as_ref().map(Rc::clone); //sets cloned audio handle
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
            if let Some(sound) = &error_sound {
                if let Some(handle) = &current_audio_handle {
                    let source: Samples = Samples::new(
                        Arc::clone(&sound.samples),
                        sound.channels,
                        sound.sample_rate,
                    );
                    handle.mixer().add(source.amplify(0.075)); //does a bunch of error checking to ensure we can use audio with no crash
                }
            }
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
        if let Some(handle) = &current_audio_handle {
            if let Some(sound) = &success_sound {
                let source: Samples = Samples::new(
                    Arc::clone(&sound.samples),
                    sound.channels,
                    sound.sample_rate,
                );
                handle.mixer().add(source.amplify(0.075));
            }
        }
        let reset_error_count = (*reset_error_read.borrow()).wrapping_add(1);
        *reset_error_read.borrow_mut() = reset_error_count;
        app_output.set_output_text(content.into());
    });
    app.run().unwrap();
}
