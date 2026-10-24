//! Relevant Fuse.js 7.5.0 Bitap and field scoring, ported to Rust.
//! Copyright (c) 2026 Kiro Risk. Apache-2.0; see licenses/FUSE-7.5.0.txt.
//! Uses UTF-16 code units, 32-bit wrapping shifts, and overlapping final chunks.
use std::collections::HashMap;

fn score(errors: usize, position: usize, expected: usize, length: usize) -> f64 {
    errors as f64 / length as f64 + expected.abs_diff(position) as f64 / 150.0
}

fn bitap(text: &[u16], pattern: &[u16], location: usize) -> (bool, f64) {
    let m = pattern.len();
    let n = text.len();
    let expected = location.min(n);
    let mut alphabet = HashMap::<u16, u32>::new();
    for (i, c) in pattern.iter().enumerate() {
        *alphabet.entry(*c).or_default() |= 1u32 << (m - i - 1);
    }
    let mut threshold: f64 = 0.5;
    let mut match_mask = vec![false; n];
    let mut from = expected;
    while from + m <= n {
        let Some(offset) = text[from..].windows(m).position(|w| w == pattern) else {
            break;
        };
        let index = from + offset;
        threshold = threshold.min(score(0, index, expected, m));
        match_mask[index..index + m].fill(true);
        from = index + m;
    }
    let mut best = None;
    let mut last = Vec::<u32>::new();
    let mut final_score: f64 = 1.0;
    let mut best_errors = 0;
    let mut bin_max = m + n;
    let mask = 1u32 << (m - 1);
    for errors in 0..m {
        let mut bin_min = 0;
        let mut bin_mid = bin_max;
        while bin_min < bin_mid {
            if score(errors, expected + bin_mid, expected, m) <= threshold {
                bin_min = bin_mid;
            } else {
                bin_max = bin_mid;
            }
            bin_mid = (bin_max - bin_min) / 2 + bin_min;
        }
        bin_max = bin_mid;
        let mut start = (expected + 1).saturating_sub(bin_mid).max(1);
        let finish = (expected + bin_mid).min(n) + m;
        let mut bits = vec![0u32; finish + 2];
        bits[finish + 1] = 1u32.wrapping_shl(errors as u32).wrapping_sub(1);
        let mut j = finish;
        while j >= start {
            let pos = j - 1;
            let char_match = text
                .get(pos)
                .and_then(|c| alphabet.get(c))
                .copied()
                .unwrap_or(0);
            bits[j] = (bits[j + 1].wrapping_shl(1) | 1) & char_match;
            if errors > 0 {
                let next = last.get(j + 1).copied().unwrap_or(0);
                let current = last.get(j).copied().unwrap_or(0);
                bits[j] |= (next | current).wrapping_shl(1) | 1 | next;
            }
            if bits[j] & mask != 0 {
                final_score = score(errors, pos, expected, m);
                if final_score <= threshold {
                    threshold = final_score;
                    best = Some(pos);
                    best_errors = errors;
                    if pos <= expected {
                        break;
                    }
                    start = (2 * expected).saturating_sub(pos).max(1);
                }
            }
            j -= 1;
        }
        if score(errors + 1, expected, expected, m) > threshold {
            break;
        }
        last = bits;
    }
    // 7.5 computes the fuzzy match mask only inside the best span.
    if let Some(best) = best {
        let end = n.min(best + m + best_errors);
        for k in best..end {
            if alphabet.contains_key(&text[k]) {
                match_mask[k] = true;
            }
        }
    }
    let has_two = match_mask.windows(2).any(|w| w[0] && w[1]);
    (best.is_some() && has_two, final_score.max(0.001))
}

pub(crate) struct Searcher {
    pattern: Vec<u16>,
    chunks: Vec<(usize, Vec<u16>)>,
}
impl Searcher {
    pub(crate) fn new(query: &str) -> Self {
        let pattern: Vec<_> = query.to_lowercase().encode_utf16().collect();
        let len = pattern.len();
        let mut chunks = Vec::new();
        if len > 32 {
            let end = len - len % 32;
            for i in (0..end).step_by(32) {
                chunks.push((i, pattern[i..i + 32].to_vec()));
            }
            if len % 32 != 0 {
                chunks.push((len - 32, pattern[len - 32..].to_vec()));
            }
        } else if len > 0 {
            chunks.push((0, pattern.clone()));
        }
        Self { pattern, chunks }
    }
    pub(crate) fn field(&self, text: &str) -> Option<f64> {
        let units: Vec<_> = text.to_lowercase().encode_utf16().collect();
        if self.pattern == units {
            return (units.len() >= 2).then_some(0.0);
        }
        let mut any = false;
        let mut total = 0.0;
        for (location, chunk) in &self.chunks {
            let (matched, score) = bitap(&units, chunk, *location);
            any |= matched;
            total += score;
        }
        any.then_some(total / self.chunks.len() as f64)
    }
    pub(crate) fn color_score(&self, fields: [&str; 3]) -> Option<f64> {
        let mut matched = false;
        let mut total = 1.0;
        for (text, weight) in fields.into_iter().zip([0.5, 0.25, 0.25]) {
            if let Some(score) = self.field(text) {
                matched = true;
                let tokens = text
                    .split(|c: char| matches!(c, '\t'..='\r' | ' ' | '\u{a0}'))
                    .filter(|t| !t.is_empty())
                    .count()
                    .max(1);
                let norm = (1000.0 / (tokens as f64).sqrt()).round() / 1000.0;
                total *= (if score == 0.0 { f64::EPSILON } else { score }).powf(weight * norm);
            }
        }
        matched.then_some(total)
    }
}
