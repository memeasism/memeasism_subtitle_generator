const POP_START: u32 = 0;
const POP_IN_MS: u32 = 67;
const POP_MS: u32 = 100;
const POP_OUT_MS: u32 = POP_MS + POP_IN_MS;
fn generate_tags(start: u32, pop_out_text: &str) -> String {
    let pop_in: u32 = start.saturating_add(POP_IN_MS);
    let pop_fin: u32 = start.saturating_add(POP_MS);
    format!(
        "{{\\fscx{}\\fscy{}\\t({},{},\\fscx125\\fscy125)\\t({},{},\\fscx100\\fscy100){}}}",
        POP_START, POP_START, start, pop_in, pop_in, pop_fin, pop_out_text
    )
}
pub fn generate(
    word_by_word: bool,
    pop_out: bool,
    delay: u32,
    text: String,
    duration_ms: u32,
) -> String {
    let pop_out_start: u32 = duration_ms.saturating_sub(POP_OUT_MS);
    let pop_out_mid: u32 = pop_out_start.saturating_add(POP_IN_MS);
    let pop_out_text: String = if pop_out {
        format!(
            "\\t({},{},\\fscx125\\fscy125)\\t({},{},\\fscx{}\\fscy{})",
            pop_out_start, pop_out_mid, pop_out_mid, duration_ms, POP_START, POP_START
        )
    } else {
        String::new()
    };
    if !word_by_word {
        return format!("{}{}", generate_tags(0, &pop_out_text), text);
    }
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }
    let delay_ms: u32 = delay.saturating_mul(10);
    let mut word_start: u32 = 0;
    let mut output: String = String::new();
    for (iteration, word) in words.iter().enumerate() {
        output.push_str(&format!(
            "{}{}",
            &generate_tags(word_start, &pop_out_text),
            word
        ));
        if iteration + 1 < words.len() {
            output.push(' ');
        }
        let characters: u32 = word.chars().count() as u32;
        let word_delay = characters * delay_ms;
        word_start = word_start + word_delay;
    }

    output
}
