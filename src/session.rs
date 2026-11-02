//! A single typing run: target text, cursor, timing and error tracking.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::layout::LessonRef;
use crate::text::{Generator, Rng};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Lesson(LessonRef),
    Daily,
    Drill,
}

pub struct Session {
    pub kind: Kind,
    pub title: String,
    pub hint: String,
    pub text: Vec<char>,
    pub pos: usize,
    pub mistakes_at: HashSet<usize>,
    pub correct: u32,
    pub errors: u32,
    pub started: Option<Instant>,
    pub ended: Option<Instant>,
    pub limit: Option<Duration>,
    /// Last wrong key and when it was pressed, for a short red flash.
    pub flash: Option<(char, Instant)>,
    /// (expected char, was correct, latency in ms) for every keystroke.
    pub events: Vec<(char, bool, Option<f64>)>,
    pub min_wpm: f64,
    pub min_acc: f64,
    last_hit: Option<Instant>,
    generator: Generator,
    rng: Rng,
}

impl Session {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        kind: Kind,
        title: String,
        hint: String,
        generator: Generator,
        len: usize,
        limit: Option<Duration>,
        min_wpm: f64,
        min_acc: f64,
    ) -> Self {
        let mut rng = Rng::new();
        let text = generator.text(&mut rng, len).chars().collect();
        Self {
            kind,
            title,
            hint,
            text,
            pos: 0,
            mistakes_at: HashSet::new(),
            correct: 0,
            errors: 0,
            started: None,
            ended: None,
            limit,
            flash: None,
            events: Vec::new(),
            min_wpm,
            min_acc,
            last_hit: None,
            generator,
            rng,
        }
    }

    pub fn type_char(&mut self, c: char) {
        if self.is_done() || self.pos >= self.text.len() {
            return;
        }
        let now = Instant::now();
        self.started.get_or_insert(now);
        let expected = self.text[self.pos];
        if c == expected {
            // Long pauses (reading, thinking) would distort per-key speed, so skip them.
            let ms = self
                .last_hit
                .map(|t| now.duration_since(t).as_secs_f64() * 1000.0)
                .filter(|ms| *ms < 2500.0);
            self.events.push((expected, true, ms));
            self.correct += 1;
            self.pos += 1;
            self.last_hit = Some(now);
            self.flash = None;
            if self.limit.is_some() {
                if self.text.len() - self.pos < 150 {
                    let more = self.generator.text(&mut self.rng, 300);
                    self.text.push(' ');
                    self.text.extend(more.chars());
                }
            } else if self.pos >= self.text.len() {
                self.ended = Some(now);
            }
        } else {
            self.errors += 1;
            self.mistakes_at.insert(self.pos);
            self.events.push((expected, false, None));
            self.flash = Some((c, now));
        }
    }

    pub fn tick(&mut self) {
        if let (Some(limit), Some(start), None) = (self.limit, self.started, self.ended)
            && start.elapsed() >= limit
        {
            self.ended = Some(start + limit);
        }
    }

    pub fn is_done(&self) -> bool {
        self.ended.is_some()
    }

    pub fn elapsed(&self) -> Duration {
        match self.started {
            None => Duration::ZERO,
            Some(s) => self.ended.unwrap_or_else(Instant::now).duration_since(s),
        }
    }

    pub fn wpm(&self) -> f64 {
        let minutes = self.elapsed().as_secs_f64() / 60.0;
        if minutes < 2.0 / 60.0 { 0.0 } else { self.correct as f64 / 5.0 / minutes }
    }

    pub fn accuracy(&self) -> f64 {
        let total = self.correct + self.errors;
        if total == 0 { 100.0 } else { self.correct as f64 * 100.0 / total as f64 }
    }

    pub fn next_char(&self) -> Option<char> {
        self.text.get(self.pos).copied()
    }

    /// 0.0..=1.0 progress through the text or time limit.
    pub fn progress(&self) -> f64 {
        match self.limit {
            Some(l) => (self.elapsed().as_secs_f64() / l.as_secs_f64()).min(1.0),
            None => self.pos as f64 / self.text.len().max(1) as f64,
        }
    }
}
