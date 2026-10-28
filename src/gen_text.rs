const POP_START: u32 = 0;
const POP_IN_MS: u32 = 67;
const POP_MS: u32 = 100;
const POP_OUT_MS: u32 = POP_MS + POP_IN_MS;
pub fn generate(
    word_by_word: bool,
    pop_out: bool,
    delay: u16,
    text: String,
    duration_ms: u32,
) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }
    if duration_ms < POP_OUT_MS + POP_MS {
        return words.join(" ");
    }
    let pop_out_start: u32 = duration_ms - POP_OUT_MS;
    let pop_out_mid: u32 = &pop_out_start + POP_IN_MS;
    let delay_ms: u32 = delay as u32 * 10;
    let mut word_start: u32 = 0;
    let mut output: String = String::new();
    for (iteration, word) in words.iter().enumerate() {
        let word_in: u32 = word_start + POP_IN_MS;
        let word_fin: u32 = word_start + POP_MS;
        output.push('{');
        output.push_str(&format!("\\fscx{}\\fscy{}", POP_START, POP_START));
        output.push_str(&format!(
            "\\t({},{},\\fscx125\\fscy125)\
             \\t({},{},\\fscx100\\fscy100)",
            word_start, word_in, word_in, word_fin
        ));
        if pop_out {
            output.push_str(&format!(
                "\\t({},{},\\fscx125\\fscy125)\\t({},{},\\fscx{}\\fscy{}",
                pop_out_start, pop_out_mid, pop_out_mid, duration_ms, POP_START, POP_START
            ));
        }
        output.push('}');
        output.push_str(word);
        if iteration + 1 < words.len() {
            output.push(' ');
        }
        if word_by_word {
            let characters: u32 = word.chars().count() as u32;
            let word_delay = characters * delay_ms;
            word_start = word_start + word_delay;
        }
    }
    return output;
}
