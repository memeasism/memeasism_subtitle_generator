pub fn parse(length: String, fps: f32) -> Option<u32> {
    let length_split: Vec<&str> = length.split(":").collect();
    let [h, m, s, f] = length_split.as_slice() else {
        return None;
    };
    let hours: u32 = h.parse().ok()?;
    let minutes: u32 = m.parse().ok()?;
    let seconds: u32 = s.parse().ok()?;
    let frames: u32 = f.parse().ok()?;
    let final_length: u32 = ((((hours * 3600) + ((minutes * 60) + seconds)) as f32
        + (frames as f32 / fps))
        * 1000.0) as u32;
    return Some(final_length);
}
