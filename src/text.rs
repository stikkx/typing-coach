//! Practice text generation restricted to the keys the trainee already knows.

use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

/// Small xorshift RNG - plenty for shuffling practice words.
pub struct Rng(u64);

impl Rng {
    pub fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Rng(seed | 1)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 { 0 } else { (self.next() % n as u64) as usize }
    }

    pub fn range(&mut self, lo: usize, hi: usize) -> usize {
        lo + self.below(hi - lo + 1)
    }

    pub fn chance(&mut self, p: f64) -> bool {
        ((self.next() >> 11) as f64 / (1u64 << 53) as f64) < p
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }

    pub fn weighted(&mut self, weights: &[f64]) -> usize {
        let total: f64 = weights.iter().sum();
        let mut r = (self.next() >> 11) as f64 / (1u64 << 53) as f64 * total;
        for (i, w) in weights.iter().enumerate() {
            if r < *w {
                return i;
            }
            r -= w;
        }
        weights.len() - 1
    }
}

static WORDS: &[&str] = &[
    // German everyday words - filtered at runtime to the learned keys.
    "der", "die", "das", "und", "ist", "nicht", "ein", "eine", "zu", "den", "von", "mit", "sich",
    "auf", "für", "im", "dem", "es", "an", "auch", "als", "da", "noch", "nach", "so", "wie", "aus",
    "bei", "wir", "ich", "du", "sie", "er", "hat", "haben", "wird", "kann", "um", "nur", "oder",
    "aber", "vor", "bis", "mehr", "durch", "man", "sehr", "schon", "wenn", "hier", "doch", "alles",
    "jetzt", "immer", "gut", "neu", "alt", "groß", "klein", "lang", "hoch", "viel", "weil", "dann",
    "denn", "also", "heute", "morgen", "abend", "tag", "jahr", "zeit", "mann", "frau", "kind",
    "haus", "welt", "stadt", "land", "weg", "hand", "auge", "kopf", "herz", "leben", "arbeit",
    "wasser", "feuer", "luft", "erde", "licht", "nacht", "sonne", "mond", "stern", "baum", "wald",
    "feld", "see", "meer", "fluss", "berg", "tal", "dorf", "straße", "schule", "buch", "brief",
    "wort", "satz", "frage", "spiel", "freund", "mutter", "vater", "essen", "trinken", "laufen",
    "gehen", "kommen", "sehen", "sagen", "machen", "geben", "nehmen", "finden", "denken", "wissen",
    "lernen", "lesen", "schreiben", "tippen", "taste", "finger", "schnell", "ruhig", "klar", "ja",
    "nein", "danke", "bitte", "hallo", "kaffee", "tee", "brot", "käse", "milch", "apfel", "öl",
    "grün", "blau", "rot", "gelb", "weiß", "schön", "müde", "früh", "spät", "über", "müssen",
    "können", "fahren", "zählen", "hören", "ändern", "ähnlich", "äpfel", "bär", "tür", "übung",
    "glück", "ruhe", "fuß", "spaß", "heiß", "süß", "gruß", "fall", "falls", "lass", "dass", "saal",
    "aal", "all", "kalk", "lasst", "gas", "glas", "halle", "hals", "hass", "sag", "fass", "flasche",
    "jagd", "kahl", "lade", "lage", "seide", "seele", "leise", "eis", "reise", "rose", "keks",
    "idee", "fisch", "eile", "geld", "held", "kiel", "lied", "dieser", "sieg", "teil", "ziel",
    "zug", "tor", "uhr", "rad", "rat", "tat", "post", "pause", "quelle", "quiz", "papier", "wolke",
    "woche", "zwei", "drei", "vier", "fünf", "sechs", "sieben", "acht", "neun", "zehn", "yoga",
    "boxen", "text", "taxi", "mix", "byte", "ja", "jedes", "jeder", "jahr", "joghurt",
    // Programming vocabulary for the code track.
    "fn", "let", "mut", "main", "self", "vec", "str", "len", "map", "key", "value", "data", "list",
    "item", "user", "host", "true", "false", "null", "none", "some", "ok", "err", "if", "else",
    "for", "while", "loop", "match", "return", "impl", "struct", "enum", "type", "use", "pub",
    "mod", "io", "file", "path", "name", "config", "args", "test",
];

static SNIPPETS: &[&str] = &[
    "fn main() {}", "let v = [1, 2, 3];", "if (a == b) { x += 1; }", "vec[i]", "Vec<u8>", "a <= b",
    "x => y", "a && b", "a || b", "// note", "/* tmp */", "&mut self", "my_var", "$HOME",
    "~/.config", "user@host", "#[derive(Debug)]", "{ key: value }", "map[\"key\"]", "f(x, y)",
    "x * (y + z)", "100%", "a != b", "i < n", "a->b", "\\n", "x = {};", "arr[0] = 1;",
    "print(\"hi\");", "c = 'a';", "#include <stdio.h>", "a % b", "x |= y", "${var}",
    "Option<String>", "fn add(a: i32, b: i32) -> i32 { a + b }", "for i in 0..10 {}",
    "let s = &v[1..];", "{ \"id\": 42 }", "if x > 0 { return; }", "obj.items[2]", "x == 0",
    "else { }", "(1 + 2) * 3", "a/b/c", "~user", "@Override", "__init__", "foo_bar",
    "C:\\temp", "cat a | grep b", "a & b", "$x = 5;", "#!/bin/sh", "<div>", "</div>", "[[1, 2], [3]]",
];

pub struct Generator {
    allowed: HashSet<char>,
    letters: Vec<char>,
    digits: Vec<char>,
    puncts: Vec<char>,
    words: Vec<&'static str>,
    snippets: Vec<&'static str>,
    focus: Vec<char>,
    focus_w: Vec<f64>,
    focus_ratio: f64,
}

impl Generator {
    /// `focus` holds (char, weight) pairs that should appear more often; `focus_ratio`
    /// is the share of tokens that are built around a focus character.
    pub fn new(allowed: &[char], focus: &[(char, f64)], focus_ratio: f64) -> Self {
        let mut allowed: HashSet<char> = allowed.iter().copied().collect();
        allowed.insert(' ');
        let mut letters: Vec<char> = allowed.iter().copied().filter(|c| c.is_alphabetic() && c.is_lowercase()).collect();
        letters.sort();
        let mut digits: Vec<char> = allowed.iter().copied().filter(char::is_ascii_digit).collect();
        digits.sort();
        let puncts = ",.!?:;".chars().filter(|c| allowed.contains(c)).collect();
        let fits = |s: &&str| s.chars().all(|c| allowed.contains(&c));
        let words = WORDS.iter().copied().filter(fits).collect();
        let snippets = SNIPPETS.iter().copied().filter(fits).collect();
        let (focus, focus_w) = focus.iter().filter(|(c, _)| *c != ' ' && allowed.contains(c)).copied().unzip();
        Self { allowed, letters, digits, puncts, words, snippets, focus, focus_w, focus_ratio }
    }

    pub fn text(&self, rng: &mut Rng, min_len: usize) -> String {
        let mut out = String::new();
        let mut last = String::new();
        let mut len = 0;
        while len < min_len {
            let mut t = self.token(rng);
            if t == last {
                t = self.token(rng);
            }
            if t.is_empty() {
                break;
            }
            if !out.is_empty() {
                out.push(' ');
                len += 1;
            }
            len += t.chars().count();
            out.push_str(&t);
            last = t;
        }
        out
    }

    fn token(&self, rng: &mut Rng) -> String {
        for _ in 0..20 {
            let t = if !self.focus.is_empty() && rng.chance(self.focus_ratio) {
                let c = self.focus[rng.weighted(&self.focus_w)];
                self.focus_token(c, rng)
            } else {
                self.plain(rng)
            };
            if !t.is_empty() && t.chars().all(|c| self.allowed.contains(&c)) {
                return t;
            }
        }
        self.pseudo(rng)
    }

    fn pseudo(&self, rng: &mut Rng) -> String {
        if self.letters.is_empty() {
            return String::new();
        }
        let max = if self.letters.len() <= 4 { 4 } else { 6 };
        (0..rng.range(2, max)).map(|_| *rng.pick(&self.letters)).collect()
    }

    fn word(&self, rng: &mut Rng) -> String {
        let p = if self.words.len() >= 8 { 0.8 } else { 0.3 };
        if !self.words.is_empty() && rng.chance(p) {
            rng.pick(&self.words).to_string()
        } else {
            self.pseudo(rng)
        }
    }

    fn capitalize(&self, w: &str) -> String {
        let mut chars = w.chars();
        let Some(first) = chars.next() else { return String::new() };
        let mut up = first.to_uppercase();
        match (up.next(), up.next()) {
            (Some(u), None) if self.allowed.contains(&u) => std::iter::once(u).chain(chars).collect(),
            _ => w.to_string(),
        }
    }

    fn number(&self, first: Option<char>, rng: &mut Rng) -> String {
        let mut s: String = first.into_iter().collect();
        let len = rng.range(1, 4);
        while s.chars().count() < len {
            s.push(*rng.pick(&self.digits));
        }
        s
    }

    fn plain(&self, rng: &mut Rng) -> String {
        if !self.snippets.is_empty() && rng.chance(0.2) {
            return rng.pick(&self.snippets).to_string();
        }
        if !self.digits.is_empty() && rng.chance(0.1) {
            return self.number(None, rng);
        }
        let mut w = self.word(rng);
        if rng.chance(0.25) {
            w = self.capitalize(&w);
        }
        if !self.puncts.is_empty() && rng.chance(0.12) {
            w.push(*rng.pick(&self.puncts));
        }
        w
    }

    fn focus_token(&self, c: char, rng: &mut Rng) -> String {
        let w = self.word(rng);
        match c {
            c if c.is_alphabetic() && c.is_lowercase() => {
                let containing: Vec<&str> = self.words.iter().copied().filter(|w| w.contains(c)).collect();
                if !containing.is_empty() && rng.chance(0.5) {
                    return rng.pick(&containing).to_string();
                }
                let mut p: Vec<char> = self.pseudo(rng).chars().collect();
                for _ in 0..rng.range(1, 2) {
                    let i = rng.below(p.len());
                    p[i] = c;
                }
                p.into_iter().collect()
            }
            c if c.is_uppercase() => {
                let lower = c.to_lowercase().next().unwrap_or(c);
                let starting: Vec<&str> = self.words.iter().copied().filter(|w| w.starts_with(lower)).collect();
                if !starting.is_empty() {
                    self.capitalize(rng.pick(&starting))
                } else {
                    let tail: String = (0..rng.range(1, 3)).map(|_| *rng.pick(&self.letters)).collect();
                    format!("{c}{tail}")
                }
            }
            c if c.is_ascii_digit() => self.number(Some(c), rng),
            '(' | ')' => format!("({w})"),
            '{' | '}' => if rng.chance(0.5) { format!("{{{w}}}") } else { format!("{{ {w} }}") },
            '[' | ']' => format!("[{w}]"),
            '<' | '>' => if rng.chance(0.5) { format!("<{w}>") } else { format!("{w} {c} {}", self.word(rng)) },
            '"' => format!("\"{w}\""),
            '\'' => format!("'{w}'"),
            '.' | ',' | '!' | '?' | ':' | ';' => format!("{w}{c}"),
            '-' | '+' | '*' | '/' | '\\' | '|' | '_' | '@' => format!("{w}{c}{}", self.word(rng)),
            '=' => format!("{w} = {}", self.word(rng)),
            '%' if !self.digits.is_empty() => format!("{}%", self.number(None, rng)),
            '#' | '&' | '$' | '~' | '§' | '%' => format!("{c}{w}"),
            _ => format!("{c}{w}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{self, LessonRef};

    #[test]
    fn every_lesson_only_uses_allowed_keys() {
        let mut rng = Rng::new();
        let refs = (0..layout::MAIN.len() + 2)
            .map(LessonRef::Main)
            .chain((0..layout::CODE.len() + 2).map(LessonRef::Code));
        for r in refs {
            let allowed = layout::allowed_for(r, layout::MAIN.len());
            let focus: Vec<(char, f64)> = layout::lesson_info(r).new_chars.iter().map(|c| (*c, 1.0)).collect();
            let generator = Generator::new(&allowed, &focus, 0.55);
            for _ in 0..50 {
                let text = generator.text(&mut rng, 200);
                assert!(text.chars().count() >= 200, "{r:?}: text too short");
                for c in text.chars() {
                    assert!(c == ' ' || allowed.contains(&c), "{r:?}: unexpected {c:?} in {text:?}");
                }
            }
            if !focus.is_empty() {
                let text = generator.text(&mut rng, 2000);
                assert!(focus.iter().any(|(c, _)| text.contains(*c)), "{r:?}: no new keys in text");
            }
        }
    }

    #[test]
    fn every_lesson_key_is_on_the_keyboard() {
        for lesson in layout::MAIN.iter().chain(layout::CODE) {
            for c in lesson.chars.chars() {
                assert!(layout::locate(c).is_some(), "{c:?} missing from layout");
            }
        }
    }
}
