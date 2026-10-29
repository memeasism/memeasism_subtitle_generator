pub fn parse(length: String, fps: f32) -> u32 {
    let length_split: Vec<&str> = length.split(":").collect();
    let [h, m, s, f] = length_split.as_slice() else {
        return 0;
    };
    let hours: u32 = h.parse().unwrap_or(0);
    let minutes: u32 = m.parse().unwrap_or(0);
    let seconds: u32 = s.parse().unwrap_or(0);
    let frames: u32 = f.parse().unwrap_or(0);
    let final_length: u32 = if fps > 0.0 {
        (((hours * 3600) + ((minutes * 60) + seconds)) * 1000)
            + ((frames as f32 / fps) * 1000.0) as u32
    } else {
        0
    };
    final_length
}
