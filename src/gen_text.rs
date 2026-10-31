use std::{fmt::Write, str::SplitWhitespace};
const POP_START_MULTIPLIER: f32 = 0.0; //0% size
const POP_PEAK_MULTIPLIER: f32 = 1.25; //125% size
const POP_IN_MS: u32 = 67; //the length of the pop in, based on my resolve subtitles
//60 fps, 4 frames for POP_IN_MS and 6 frames for POP_MS
//start is 0 because it looks cool, resolve I do 75 actually
const POP_MS: u32 = 100;
const POP_OUT_MS: u32 = POP_MS + POP_IN_MS; //could be static, this is just easier
const TAG_LENGTH: usize = 44; //the literal count of the tag length before adding everything else
fn words(text: &str) -> SplitWhitespace<'_> {
    //function to make code cleaner for the for loops, and uses zero ram as opposed to vectors or caching
    text.split_whitespace()
}
fn count_digit_bytes(number: u32) -> usize {
    //function to grab how many bytes a number is
    if number == 0 {
        1
    } else {
        (number.ilog10() as usize) + 1
    }
}
fn tag_block_length(
    //function to precompute tag length, just an uneccesary optimization
    start: u32,
    fixed_length: usize,
    pop_out_text: &str,
    center_text: &str,
    rotation_text: &str,
) -> usize {
    let pop_in: u32 = start.saturating_add(POP_IN_MS);
    let pop_fin: u32 = start.saturating_add(POP_MS);
    fixed_length
        + count_digit_bytes(start)
        + count_digit_bytes(pop_in) * 2
        + count_digit_bytes(pop_fin)
        + rotation_text.len()
        + center_text.len()
        + pop_out_text.len()
}
fn generate_tags(
    output: &mut String,
    pop_out_text: &str,
    center_text: &str,
    rotation_text: &str,
    start: u32,
    pop_start: u32,
    pop_peak: u32,
    pop_end: u32,
) {
    //seperate function for generating tags, allows early return
    let pop_in: u32 = start.saturating_add(POP_IN_MS);
    let pop_fin: u32 = start.saturating_add(POP_MS);

    let _output: Result<(), std::fmt::Error> = write!(
        // variable = ignore failure = simpler code with no crash
        output,
        "{{\\fscx{}\\fscy{}\\t({},{},\\fscx{}\\fscy{})\\t({},{},\\fscx{}\\fscy{}){}{}{}}}",
        pop_start,
        pop_start,
        start,
        pop_in,
        pop_peak,
        pop_peak,
        pop_in,
        pop_fin,
        pop_end,
        pop_end,
        rotation_text,
        center_text,
        pop_out_text
    );
}
pub fn generate(
    word_by_word: bool,
    pop_out: bool,
    center: bool,
    delay: f32,
    scale: f32,
    rotation: f32,
    text: &str,
    duration_ms: u32,
) -> String {
    let pop_start: u32 = (scale * POP_START_MULTIPLIER).round() as u32;
    let pop_peak: u32 = (scale * POP_PEAK_MULTIPLIER).round() as u32;
    let pop_end: u32 = scale.round() as u32;
    let fixed_length: usize = TAG_LENGTH //precompute static values for less compute
        + count_digit_bytes(pop_start) * 2
        + count_digit_bytes(pop_peak) * 2
        + count_digit_bytes(pop_end) * 2;
    let pop_out_start: u32 = duration_ms.saturating_sub(POP_OUT_MS);
    let pop_out_mid: u32 = pop_out_start.saturating_add(POP_IN_MS);
    let center_text: &str = if center { "\\an5" } else { "" };
    let rotation_text: &str = if rotation != 0.0 {
        &format!("\\fr{}", rotation)
    } else {
        ""
    };
    let pop_out_text: &str = if pop_out {
        //if pop out, then this string has a real value, otherwise its just an empty string
        &format!(
            "\\t({},{},\\fscx{}\\fscy{})\\t({},{},\\fscx{}\\fscy{})",
            pop_out_start,
            pop_out_mid,
            pop_peak,
            pop_peak,
            pop_out_mid,
            duration_ms,
            pop_start,
            pop_start
        )
    } else {
        ""
    };
    if !word_by_word {
        let tag_length: usize =
            tag_block_length(0, fixed_length, pop_out_text, center_text, rotation_text);
        let mut output: String = String::with_capacity(tag_length + text.len()); //preallocate memory, RAMPOCALYPSE! But realistically useless
        generate_tags(
            &mut output,
            pop_out_text,
            center_text,
            rotation_text,
            0,
            pop_start,
            pop_peak,
            pop_end,
        );
        output.push_str(text);
        return output; //early return if not using word by word
    }
    let word_count: usize = words(text).count();
    if word_count == 0 {
        return String::new();
    }
    let delay_ms: u32 = (delay * 10.0).round() as u32;
    let mut word_starts: Vec<u32> = Vec::with_capacity(word_count);
    let mut tag_length: usize = 0;
    let mut word_start: u32 = 0;
    for (iteration, word) in words(text).enumerate() {
        //collect all word starts when getting tag length to save on compute
        word_starts.push(word_start);
        tag_length += tag_block_length(
            word_start,
            fixed_length,
            pop_out_text,
            center_text,
            rotation_text,
        );
        tag_length += word.len();
        if iteration + 1 < word_count {
            tag_length += 1;
        }
        let characters: u32 = word.chars().count() as u32;
        word_start = word_start.saturating_add(characters.saturating_mul(delay_ms));
    }
    let mut output: String = String::with_capacity(tag_length); //preallocate memory, RAMPOCALYPSE! But realistically useless
    for (iteration, word) in words(text).enumerate() {
        generate_tags(
            &mut output,
            pop_out_text,
            center_text,
            rotation_text,
            word_starts[iteration],
            pop_start,
            pop_peak,
            pop_end,
        );
        output.push_str(word);
        if iteration + 1 < word_count {
            output.push(' ');
        }
    }
    output
}
