//! Offline HTML Colors model and Native V2 view. No OS effects or JS runtime.
mod fuse;
mod view;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    OnceLock,
};

pub use view::{render, CopyFormat, Effect};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Color {
    pub name: String,
    pub hex: String,
    pub rgb: String,
    pub category: String,
    pub format: String,
    pub id: String,
    pub shade: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorResult {
    #[serde(flatten)]
    pub color: Color,
    pub categories: Vec<String>,
}
pub fn colors() -> &'static [Color] {
    static DATA: OnceLock<Vec<Color>> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("colors.json")).expect("validated color data")
    })
}
pub const SHADE_ORDER: [&str; 11] = [
    "pink",
    "red",
    "orange",
    "yellow",
    "brown",
    "green",
    "white",
    "black and gray",
    "purple, violet and magenta",
    "blue",
    "cyan",
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum View {
    #[default]
    All,
    Basic,
    Extended,
    Grouped,
}
impl View {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "all" => Ok(Self::All),
            "basic" => Ok(Self::Basic),
            "extended" => Ok(Self::Extended),
            "grouped" => Ok(Self::Grouped),
            _ => Err("Invalid view".into()),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Basic => "basic",
            Self::Extended => "extended",
            Self::Grouped => "grouped",
        }
    }
}

/// Raw ranked records before deduplication, useful for independently checking scores.
pub fn ranked(view: View, query: &str) -> Vec<(usize, f64)> {
    let search = fuse::Searcher::new(query);
    let mut result: Vec<_> = colors()
        .iter()
        .enumerate()
        .filter(|(_, c)| match view {
            View::Basic => c.category == "basic",
            View::Extended => c.category == "extended",
            _ => true,
        })
        .filter_map(|(i, c)| {
            search
                .color_score([&c.name, &c.hex, &c.rgb])
                .filter(|s| *s < 0.5)
                .map(|s| (i, s))
        })
        .collect();
    result.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
    result
}
pub fn search(view: View, query: &str) -> Vec<ColorResult> {
    if query.is_empty() {
        return colors()
            .iter()
            .filter(|c| match view {
                View::Basic => c.category == "basic",
                View::Extended => c.category == "extended",
                _ => true,
            })
            .map(|c| ColorResult {
                color: c.clone(),
                categories: vec![c.category.clone()],
            })
            .collect();
    }
    let mut result: Vec<ColorResult> = Vec::new();
    for (i, _) in ranked(view, query) {
        let c = &colors()[i];
        if let Some(existing) = result.iter_mut().find(|r| r.color.name == c.name) {
            if !existing.categories.contains(&c.category) {
                existing.categories.push(c.category.clone());
            }
        } else {
            result.push(ColorResult {
                color: c.clone(),
                categories: vec![c.category.clone()],
            });
        }
    }
    result
}
pub fn match_score(text: &str, query: &str) -> f64 {
    let text = text.to_lowercase();
    let query = query.to_lowercase();
    if text == query {
        0.0
    } else if text.starts_with(&format!("{query} "))
        || text.contains(&format!(" {query}"))
        || text.ends_with(&format!(" {query}"))
    {
        0.2
    } else if text.starts_with(&query) {
        0.4
    } else if let Some(pos) = text.find(&query) {
        0.6 + text[..pos].encode_utf16().count() as f64 / text.encode_utf16().count() as f64 * 0.2
    } else {
        1.0
    }
}
pub fn sections(rows: &[ColorResult], query: &str) -> Vec<(String, Vec<ColorResult>)> {
    let mut groups: Vec<_> = SHADE_ORDER
        .iter()
        .map(|shade| {
            (
                shade.to_string(),
                rows.iter()
                    .filter(|r| r.color.shade == *shade)
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        })
        .filter(|(_, r)| !r.is_empty())
        .collect();
    if !query.is_empty() {
        let best = |r: &[ColorResult]| {
            r.iter()
                .map(|c| {
                    [&c.color.name, &c.color.hex, &c.color.rgb]
                        .into_iter()
                        .map(|t| match_score(t, query))
                        .fold(1.0, f64::min)
                })
                .fold(1.0, f64::min)
        };
        let count = |r: &[ColorResult]| {
            r.iter()
                .filter(|c| c.color.name.to_lowercase().contains(&query.to_lowercase()))
                .count()
        };
        groups.sort_by(|a, b| {
            best(&a.1)
                .total_cmp(&best(&b.1))
                .then_with(|| count(&b.1).cmp(&count(&a.1)))
        });
    }
    groups
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub query: String,
    pub view: View,
    pub show_hex: bool,
    pub details: bool,
    pub active: bool,
    token: String,
}
fn token() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    // Process nonce plus monotonic generation: distinct across sessions and staged clones.
    static NONCE: OnceLock<u128> = OnceLock::new();
    let nonce = NONCE.get_or_init(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    });
    format!("{nonce:x}-{}", NEXT.fetch_add(1, Ordering::Relaxed))
}
impl Default for Session {
    fn default() -> Self {
        Self {
            query: String::new(),
            view: View::All,
            show_hex: true,
            details: false,
            active: true,
            token: token(),
        }
    }
}
impl Session {
    pub fn rows(&self) -> Vec<ColorResult> {
        search(self.view, &self.query)
    }
    pub fn action_id(&self, suffix: &str) -> String {
        format!("html-colors:{}:{suffix}", self.token)
    }
    pub fn set_query(&mut self, query: String) -> Result<(), String> {
        self.ensure_active()?;
        self.query = query;
        self.token = token();
        Ok(())
    }
    pub fn dispose(&mut self) {
        self.active = false;
        self.token = token();
    }
    pub fn ensure_active(&self) -> Result<(), String> {
        if self.active {
            Ok(())
        } else {
            Err("HTML Colors session is disposed; reopen the command".into())
        }
    }
    pub fn invoke(&mut self, id: &str, values: &serde_json::Value) -> Result<Effect, String> {
        self.ensure_active()?;
        let prefix = self.action_id("");
        let suffix = id
            .strip_prefix(&prefix)
            .ok_or("Stale or invalid HTML Colors action")?;
        if suffix == "view" {
            let obj = values.as_object().ok_or("Missing view value")?;
            if obj.len() != 1 {
                return Err("Only the view value is accepted".into());
            }
            let view = View::parse(
                obj.get("value")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing view value")?,
            )?;
            self.view = view;
            self.token = token();
            return Ok(Effect::Render);
        }
        if !values.is_null() && values.as_object().is_none_or(|o| !o.is_empty()) {
            return Err("Action payload values are not accepted".into());
        }
        let mut parts = suffix.split(':');
        let index: usize = parts
            .next()
            .and_then(|v| v.parse().ok())
            .ok_or("Invalid row action")?;
        let op = parts.next().ok_or("Missing row operation")?;
        if suffix != format!("{index}:{op}") {
            return Err("Noncanonical row action".into());
        }
        if parts.next().is_some() {
            return Err("Invalid row action".into());
        }
        let rows = self.rows();
        let row = rows.get(index).ok_or("Row is no longer present")?;
        match op {
            "hex" => Ok(Effect::Copy(row.color.hex.clone())),
            "rgb" => Ok(Effect::Copy(row.color.rgb.clone())),
            "name" => Ok(Effect::Copy(row.color.id.clone())),
            "format" => {
                self.show_hex = !self.show_hex;
                self.token = token();
                Ok(Effect::Render)
            }
            "detail" => {
                self.details = !self.details;
                self.token = token();
                Ok(Effect::Render)
            }
            _ => Err("Unknown HTML Colors action".into()),
        }
    }
}
