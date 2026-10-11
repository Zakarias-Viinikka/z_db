/// Fold a byte stream into lines. `\r` replaces the current line (cargo's
/// progress bar uses many of these to overwrite itself), `\n` commits it.
/// Only the last `\r`-separated piece per line is ever visible, so this is
/// what makes the progress ticks disappear without a real parser.
#[derive(Debug, Default, Clone)]
pub struct LineFolder {
    pub lines: Vec<String>,
    pub current: String,
    /// The most recent `] N/M` seen, kept across `\r`-clears so the UI can
    /// always read a live value even if `current` was just wiped.
    pub last_progress: Option<(u64, u64)>,
}

impl LineFolder {
    pub fn push_str(&mut self, s: &str) {
        for c in s.chars() {
            match c {
                '\n' => {
                    if !self.current.is_empty() {
                        let line = std::mem::take(&mut self.current);
                        self.lines.push(line);
                    }
                }
                '\r' => {
                    // Grab the fraction from the line we're about to wipe.
                    if let Some(p) = progress_in(&self.current) {
                        self.last_progress = Some(p);
                    }
                    self.current.clear();
                }
                _ => self.current.push(c),
            }
        }
        // Also catch a fraction in a chunk that ends without a \r.
        if let Some(p) = progress_in(&self.current) {
            self.last_progress = Some(p);
        }
    }

    pub fn flush(&mut self) {
        if !self.current.is_empty() {
            let line = std::mem::take(&mut self.current);
            self.lines.push(line);
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.current.clear();
        self.last_progress = None;
    }

    pub fn progress_from_current(&self) -> Option<(u64, u64)> {
        self.last_progress
    }
}

/// Find the last `] N/M` fraction in a string, if any.
fn progress_in(s: &str) -> Option<(u64, u64)> {
    let bytes = s.as_bytes();
    let mut last: Option<(u64, u64)> = None;
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b']' && bytes[i + 1] == b' ' {
            let mut j = i + 2;
            let cur_start = j;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > cur_start && j < bytes.len() && bytes[j] == b'/' {
                let mut k = j + 1;
                let total_start = k;
                while k < bytes.len() && bytes[k].is_ascii_digit() {
                    k += 1;
                }
                if k > total_start {
                    if let (Ok(cur), Ok(total)) = (
                        s[cur_start..j].parse::<u64>(),
                        s[total_start..k].parse::<u64>(),
                    ) {
                        last = Some((cur, total));
                    }
                }
            }
        }
        i += 1;
    }
    last
}