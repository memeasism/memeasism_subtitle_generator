pub fn parse(length: &str, fps: f32) -> u32 {
    let mut length_split: std::str::SplitN<'_, char> = length.splitn(5, ':');
    let (Some(h), Some(m), Some(s), Some(f), None) = (
        //get 4 values, if there's a fifth return 0, means if more than 5 values, it will also return 0
        length_split.next(),
        length_split.next(),
        length_split.next(),
        length_split.next(),
        length_split.next(),
    ) else {
        return 0;
    };
    let hours: u32 = h.parse().unwrap_or(0);
    let minutes: u32 = m.parse().unwrap_or(0);
    let seconds: u32 = s.parse().unwrap_or(0); //get the numbers for the values, or set to 0 if it can't
    let frames: u32 = f.parse().unwrap_or(0);
    hours //use saturating math to prevent errors, and if fps is lower than 0 fps gets forced to 0 to prevent error
        .saturating_mul(3600)
        .saturating_add(minutes.saturating_mul(60))
        .saturating_add(seconds)
        .saturating_mul(1000)
        .saturating_add(if fps > 0.0 {
            ((frames as f32 / fps) * 1000.0) as u32
        } else {
            0
        })
}
