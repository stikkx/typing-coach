//! Persistent trainee progress, stored as JSON in the user's data directory.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::layout;

/// Per-key statistics. The EMA fields reflect recent performance so that
/// improvement shows up quickly instead of being buried under old data.
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct KeyStat {
    pub hits: u32,
    pub misses: u32,
    pub ema_ms: f64,
    pub ema_err: f64,
}

impl KeyStat {
    const ALPHA: f64 = 0.15;

    pub fn record(&mut self, correct: bool, ms: Option<f64>) {
        if correct {
            self.hits += 1;
            if let Some(ms) = ms {
                self.ema_ms = if self.ema_ms == 0.0 { ms } else { self.ema_ms * (1.0 - Self::ALPHA) + ms * Self::ALPHA };
            }
            self.ema_err *= 1.0 - Self::ALPHA;
        } else {
            self.misses += 1;
            self.ema_err = self.ema_err * (1.0 - Self::ALPHA) + Self::ALPHA;
        }
    }

    /// Higher means the key needs more practice.
    pub fn weakness(&self) -> f64 {
        if self.hits + self.misses < 10 {
            return 2.0;
        }
        self.ema_err * 6.0 + self.ema_ms / 500.0
    }

    pub fn error_rate(&self) -> Option<f64> {
        let total = self.hits + self.misses;
        (total > 0).then(|| self.misses as f64 / total as f64)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Record {
    pub date: NaiveDate,
    pub kind: String,
    pub wpm: f64,
    pub accuracy: f64,
    pub chars: u32,
    pub seconds: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Progress {
    /// Number of completed main lessons (values past the curriculum are speed levels).
    pub main_level: usize,
    pub code_level: usize,
    pub code_enabled: bool,
    /// Max new lessons per day; 0 = unlimited.
    pub pace: u8,
    pub lessons_day: Option<NaiveDate>,
    pub lessons_today: u8,
    pub last_daily: Option<NaiveDate>,
    pub streak: u32,
    pub best_wpm: f64,
    pub keys: BTreeMap<char, KeyStat>,
    pub history: Vec<Record>,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            main_level: 0,
            code_level: 0,
            code_enabled: false,
            pace: 2,
            lessons_day: None,
            lessons_today: 0,
            last_daily: None,
            streak: 0,
            best_wpm: 0.0,
            keys: BTreeMap::new(),
            history: Vec::new(),
        }
    }
}

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

impl Progress {
    pub fn path() -> PathBuf {
        if let Ok(p) = std::env::var("TYPING_COACH_FILE") {
            return PathBuf::from(p);
        }
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("typing_coach")
            .join("progress.json")
    }

    pub fn load() -> Self {
        let path = Self::path();
        let Ok(data) = fs::read_to_string(&path) else {
            return Self::default();
        };
        match serde_json::from_str(&data) {
            Ok(p) => p,
            Err(_) => {
                // Keep the unreadable file around rather than silently overwriting it.
                let _ = fs::rename(&path, path.with_extension("json.bak"));
                Self::default()
            }
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        fs::rename(tmp, path)
    }

    pub fn lessons_today(&self) -> u8 {
        if self.lessons_day == Some(today()) { self.lessons_today } else { 0 }
    }

    pub fn can_advance_today(&self) -> bool {
        self.pace == 0 || self.lessons_today() < self.pace
    }

    pub fn note_advance(&mut self) {
        self.lessons_today = self.lessons_today() + 1;
        self.lessons_day = Some(today());
    }

    pub fn code_unlocked(&self) -> bool {
        self.main_level >= layout::CODE_UNLOCK_LEVEL
    }

    pub fn learned_chars(&self) -> Vec<char> {
        let code = (self.code_enabled && self.code_unlocked()).then_some(self.code_level);
        layout::learned(self.main_level, code)
    }

    pub fn daily_done_today(&self) -> bool {
        self.last_daily == Some(today())
    }

    /// The streak only counts while it is still alive (done today or yesterday).
    pub fn current_streak(&self) -> u32 {
        match self.last_daily {
            Some(d) if (today() - d).num_days() <= 1 => self.streak,
            _ => 0,
        }
    }

    pub fn complete_daily(&mut self) {
        let t = today();
        match self.last_daily {
            Some(d) if d == t => {}
            Some(d) if (t - d).num_days() == 1 => self.streak += 1,
            _ => self.streak = 1,
        }
        self.last_daily = Some(t);
    }

    /// Learned characters sorted from weakest to strongest.
    pub fn weakest(&self, among: &[char], n: usize) -> Vec<(char, f64)> {
        let mut v: Vec<(char, f64)> = among
            .iter()
            .filter(|c| **c != ' ')
            .map(|c| (*c, self.keys.get(c).map_or(2.0, KeyStat::weakness)))
            .collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1));
        v.truncate(n);
        v
    }

    pub fn rank(&self) -> &'static str {
        let coder = self.code_level >= layout::CODE.len();
        match (self.best_wpm, coder) {
            (w, true) if w >= 50.0 => "Code Wizard",
            (w, true) if w >= 30.0 => "Code Ninja",
            (w, _) if w >= 80.0 => "Typing Legend",
            (w, _) if w >= 65.0 => "Speed Demon",
            (w, _) if w >= 50.0 => "Keyboard Racer",
            (w, _) if w >= 40.0 => "Swift Fingers",
            (w, _) if w >= 30.0 => "Steady Typist",
            (w, _) if w >= 20.0 => "Home Row Hopper",
            _ => "Hunt & Peck Hatchling",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_round_trips_through_json() {
        let mut p = Progress::default();
        p.keys.entry('ö').or_default().record(false, None);
        p.keys.entry('{').or_default().record(true, Some(320.0));
        p.complete_daily();
        let json = serde_json::to_string(&p).unwrap();
        let back: Progress = serde_json::from_str(&json).unwrap();
        assert_eq!(back.keys.len(), 2);
        assert_eq!(back.keys[&'ö'].misses, 1);
        assert_eq!(back.current_streak(), 1);
    }
}
