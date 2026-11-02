//! Application state machine: menu, sessions, results and progress updates.

use std::collections::BTreeMap;
use std::time::Duration;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::layout::{self, LessonRef};
use crate::session::{Kind, Session};
use crate::state::{self, Progress, Record};
use crate::text::Generator;

pub enum Screen {
    Menu,
    Typing,
    Result,
    Stats,
    Help,
}

#[derive(Clone, Copy)]
pub enum Action {
    Daily,
    Lesson(LessonRef),
    Drill,
    ToggleCode,
    CyclePace,
    Stats,
    Help,
    Quit,
}

pub struct MenuItem {
    pub action: Action,
    pub label: String,
    pub note: String,
    pub enabled: bool,
}

pub struct Outcome {
    pub kind: Kind,
    pub title: String,
    pub wpm: f64,
    pub accuracy: f64,
    pub seconds: f64,
    pub chars: u32,
    pub errors: u32,
    pub stars: u8,
    pub passed: Option<bool>,
    pub min_wpm: f64,
    pub min_acc: f64,
    pub messages: Vec<String>,
    pub tricky: Vec<(char, u32)>,
}

pub struct App {
    pub progress: Progress,
    pub screen: Screen,
    pub menu_idx: usize,
    pub session: Option<Session>,
    pub outcome: Option<Outcome>,
    pub status: Option<String>,
    pub quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            progress: Progress::load(),
            screen: Screen::Menu,
            menu_idx: 0,
            session: None,
            outcome: None,
            status: None,
            quit: false,
        }
    }

    pub fn menu_items(&self) -> Vec<MenuItem> {
        let p = &self.progress;
        let started = p.main_level > 0;
        let mut items = vec![MenuItem {
            action: Action::Daily,
            label: "Daily 2-minute training".into(),
            note: if !started {
                "finish lesson 1 first".into()
            } else if p.daily_done_today() {
                "done today - once more?".into()
            } else {
                "do this every day".into()
            },
            enabled: started,
        }];

        let lesson_note = |info: &layout::LessonInfo| {
            if !p.can_advance_today() {
                "today's new-lesson limit reached - practice only".to_string()
            } else {
                format!("goal: {:.0} WPM, {:.0}% accuracy", info.min_wpm, info.min_acc)
            }
        };
        let main = layout::lesson_info(LessonRef::Main(p.main_level));
        let label = if p.main_level < layout::MAIN.len() {
            format!("Lesson {}: {}", p.main_level + 1, main.title)
        } else {
            main.title.clone()
        };
        items.push(MenuItem {
            action: Action::Lesson(LessonRef::Main(p.main_level)),
            label,
            note: lesson_note(&main),
            enabled: true,
        });

        if p.code_enabled && p.code_unlocked() {
            let code = layout::lesson_info(LessonRef::Code(p.code_level));
            let label = if p.code_level < layout::CODE.len() {
                format!("Code lesson {}: {}", p.code_level + 1, code.title)
            } else {
                code.title.clone()
            };
            items.push(MenuItem {
                action: Action::Lesson(LessonRef::Code(p.code_level)),
                label,
                note: lesson_note(&code),
                enabled: true,
            });
        }

        items.push(MenuItem {
            action: Action::Drill,
            label: "Weak-key drill (1 min)".into(),
            note: if started { "hammer your slowest and most error-prone keys".into() } else { String::new() },
            enabled: started,
        });
        items.push(MenuItem {
            action: Action::ToggleCode,
            label: format!("Programming track: {}", if p.code_enabled { "ON" } else { "OFF" }),
            note: if p.code_unlocked() {
                "{ } ( ) [ ] = < > / \\ | _ # & % $ @ ~".into()
            } else {
                "unlocks after Capitals".into()
            },
            enabled: true,
        });
        items.push(MenuItem {
            action: Action::CyclePace,
            label: match p.pace {
                0 => "Pace: unlimited new lessons per day".into(),
                n => format!("Pace: max {n} new lesson{} per day", if n == 1 { "" } else { "s" }),
            },
            note: format!("{} unlocked today", p.lessons_today()),
            enabled: true,
        });
        items.push(MenuItem { action: Action::Stats, label: "Statistics".into(), note: String::new(), enabled: true });
        items.push(MenuItem { action: Action::Help, label: "How it works".into(), note: String::new(), enabled: true });
        items.push(MenuItem { action: Action::Quit, label: "Quit".into(), note: String::new(), enabled: true });
        items
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        let ctrl_c = key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c');
        match self.screen {
            Screen::Menu => self.on_menu_key(key, ctrl_c),
            Screen::Typing => match key.code {
                _ if ctrl_c => self.abort(),
                KeyCode::Esc => self.abort(),
                // AltGr arrives as Ctrl+Alt on some terminals, so accept chars with any modifiers.
                KeyCode::Char(c) => {
                    if let Some(s) = self.session.as_mut() {
                        s.type_char(c);
                    }
                }
                _ => {}
            },
            Screen::Result => match key.code {
                KeyCode::Char('r') => {
                    if let Some(o) = &self.outcome {
                        let action = match o.kind {
                            Kind::Daily => Action::Daily,
                            Kind::Drill => Action::Drill,
                            Kind::Lesson(LessonRef::Main(_)) => Action::Lesson(LessonRef::Main(self.progress.main_level)),
                            Kind::Lesson(LessonRef::Code(_)) => Action::Lesson(LessonRef::Code(self.progress.code_level)),
                        };
                        self.run(action);
                    }
                }
                _ if ctrl_c => self.quit = true,
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') | KeyCode::Char('q') => self.screen = Screen::Menu,
                _ => {}
            },
            Screen::Stats | Screen::Help => {
                if ctrl_c {
                    self.quit = true;
                } else {
                    self.screen = Screen::Menu;
                }
            }
        }
    }

    fn on_menu_key(&mut self, key: KeyEvent, ctrl_c: bool) {
        let items = self.menu_items();
        self.status = None;
        match key.code {
            _ if ctrl_c => self.quit = true,
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Up | KeyCode::Char('k') => self.menu_idx = (self.menu_idx + items.len() - 1) % items.len(),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => self.menu_idx = (self.menu_idx + 1) % items.len(),
            KeyCode::Enter | KeyCode::Char(' ') => {
                let item = &items[self.menu_idx.min(items.len() - 1)];
                if item.enabled {
                    self.run(item.action);
                } else {
                    self.status = Some(format!("Not yet: {}", item.note));
                }
            }
            KeyCode::Char(c @ '1'..='9') => {
                let i = c as usize - '1' as usize;
                if let Some(item) = items.get(i) {
                    self.menu_idx = i;
                    if item.enabled {
                        self.run(item.action);
                    }
                }
            }
            _ => {}
        }
    }

    fn run(&mut self, action: Action) {
        let p = &mut self.progress;
        match action {
            Action::Quit => self.quit = true,
            Action::Stats => self.screen = Screen::Stats,
            Action::Help => self.screen = Screen::Help,
            Action::CyclePace => {
                p.pace = match p.pace {
                    1 => 2,
                    2 => 3,
                    3 => 0,
                    _ => 1,
                };
                self.save();
            }
            Action::ToggleCode => {
                p.code_enabled = !p.code_enabled;
                if p.code_enabled && !p.code_unlocked() {
                    self.status = Some(format!(
                        "Programming track will appear after lesson {} (Capitals).",
                        layout::CODE_UNLOCK_LEVEL
                    ));
                }
                self.save();
            }
            Action::Daily => {
                let learned = p.learned_chars();
                // Weight every learned key by weakness, with a bonus for the newest keys.
                let newest = layout::lesson_info(LessonRef::Main(p.main_level.saturating_sub(1))).new_chars;
                let focus: Vec<(char, f64)> = p
                    .weakest(&learned, usize::MAX)
                    .into_iter()
                    .map(|(c, w)| (c, if newest.contains(&c) { w * 1.5 } else { w }))
                    .collect();
                self.start(Session::new(
                    Kind::Daily,
                    "Daily 2-minute training".into(),
                    "Everything you have learned so far - weak keys show up more often.".into(),
                    Generator::new(&learned, &focus, 0.45),
                    300,
                    Some(Duration::from_secs(120)),
                    0.0,
                    0.0,
                ));
            }
            Action::Drill => {
                let learned = p.learned_chars();
                let focus = p.weakest(&learned, 6);
                let names: Vec<String> = focus.iter().map(|(c, _)| c.to_string()).collect();
                self.start(Session::new(
                    Kind::Drill,
                    "Weak-key drill".into(),
                    format!("Focus keys: {}", names.join(" ")),
                    Generator::new(&learned, &focus, 0.8),
                    300,
                    Some(Duration::from_secs(60)),
                    0.0,
                    0.0,
                ));
            }
            Action::Lesson(r) => {
                let info = layout::lesson_info(r);
                let allowed = layout::allowed_for(r, p.main_level);
                let focus: Vec<(char, f64)> = if info.new_chars.is_empty() {
                    p.weakest(&allowed, 6)
                } else {
                    info.new_chars.iter().map(|c| (*c, 1.0)).collect()
                };
                let (title, len) = match r {
                    LessonRef::Main(n) if n < layout::MAIN.len() => {
                        (format!("Lesson {}: {}", n + 1, info.title), (100 + 8 * n).min(260))
                    }
                    LessonRef::Code(n) if n < layout::CODE.len() => {
                        (format!("Code lesson {}: {}", n + 1, info.title), 180)
                    }
                    _ => (info.title.clone(), 300),
                };
                let ratio = if info.new_chars.is_empty() { 0.3 } else { 0.55 };
                self.start(Session::new(
                    Kind::Lesson(r),
                    title,
                    info.hint,
                    Generator::new(&allowed, &focus, ratio),
                    len,
                    info.timed.then(|| Duration::from_secs(60)),
                    info.min_wpm,
                    info.min_acc,
                ));
            }
        }
    }

    fn start(&mut self, session: Session) {
        self.session = Some(session);
        self.screen = Screen::Typing;
    }

    fn abort(&mut self) {
        self.session = None;
        self.screen = Screen::Menu;
        self.status = Some("Session aborted - nothing was recorded.".into());
    }

    fn save(&mut self) {
        if let Err(e) = self.progress.save() {
            self.status = Some(format!("Could not save progress: {e}"));
        }
    }

    /// Called every frame: finishes the running session once it's done.
    pub fn tick(&mut self) {
        let done = match self.session.as_mut() {
            Some(s) => {
                s.tick();
                s.is_done()
            }
            None => false,
        };
        if done {
            self.finish();
        }
    }

    fn finish(&mut self) {
        let Some(s) = self.session.take() else { return };
        let p = &mut self.progress;

        let mut misses: BTreeMap<char, u32> = BTreeMap::new();
        for &(c, ok, ms) in &s.events {
            p.keys.entry(c).or_default().record(ok, ms);
            if !ok {
                *misses.entry(c).or_default() += 1;
            }
        }
        let mut tricky: Vec<(char, u32)> = misses.into_iter().collect();
        tricky.sort_by(|a, b| b.1.cmp(&a.1));
        tricky.truncate(4);

        let wpm = s.wpm();
        let accuracy = s.accuracy();
        let seconds = s.elapsed().as_secs_f64();
        let kind_label = match s.kind {
            Kind::Daily => "daily".to_string(),
            Kind::Drill => "drill".to_string(),
            Kind::Lesson(LessonRef::Main(n)) => format!("lesson {}", n + 1),
            Kind::Lesson(LessonRef::Code(n)) => format!("code {}", n + 1),
        };
        p.history.push(Record { date: state::today(), kind: kind_label, wpm, accuracy, chars: s.correct, seconds });

        let mut messages = Vec::new();
        let mut passed = None;
        let mut counts_for_best = matches!(s.kind, Kind::Daily | Kind::Drill);

        match s.kind {
            Kind::Lesson(r) => {
                let ok = accuracy >= s.min_acc && wpm >= s.min_wpm;
                passed = Some(ok);
                let (level, is_main) = match r {
                    LessonRef::Main(n) => (n, true),
                    LessonRef::Code(n) => (n, false),
                };
                counts_for_best |= is_main && level >= layout::MAIN.len();
                let current = if is_main { p.main_level } else { p.code_level };
                if ok && level == current {
                    if p.can_advance_today() {
                        if is_main {
                            p.main_level += 1;
                        } else {
                            p.code_level += 1;
                        }
                        p.note_advance();
                        let next = layout::lesson_info(if is_main {
                            LessonRef::Main(level + 1)
                        } else {
                            LessonRef::Code(level + 1)
                        });
                        messages.push(format!("Unlocked: {}!", next.title));
                        if is_main && p.main_level == layout::CODE_UNLOCK_LEVEL {
                            messages.push("The programming track is now available - switch it on in the menu.".into());
                        }
                    } else {
                        messages.push(format!(
                            "Passed! But you've reached today's limit of {} new lesson(s). Sleep turns practice into skill - the next lesson unlocks tomorrow.",
                            p.pace
                        ));
                    }
                } else if !ok {
                    if accuracy < s.min_acc {
                        messages.push(format!(
                            "Accuracy first: aim for {:.0}%. Slow down a little - speed follows precision.",
                            s.min_acc
                        ));
                    }
                    if wpm < s.min_wpm {
                        messages.push(format!("Almost! You need {:.0} WPM. Keep a steady rhythm, don't look down.", s.min_wpm));
                    }
                }
            }
            Kind::Daily => {
                let before = p.current_streak();
                p.complete_daily();
                let now = p.current_streak();
                if now > before {
                    messages.push(format!("Streak: {now} day{} in a row!", if now == 1 { "" } else { "s" }));
                } else {
                    messages.push("Extra practice - your brain thanks you.".into());
                }
            }
            Kind::Drill => {}
        }

        if counts_for_best && accuracy >= 90.0 && wpm > p.best_wpm {
            if p.best_wpm > 0.0 {
                messages.push(format!("New personal best: {wpm:.1} WPM (was {:.1})!", p.best_wpm));
            }
            p.best_wpm = wpm;
        }

        let stars = match passed {
            Some(false) => 0,
            Some(true) if accuracy >= 99.0 && wpm >= s.min_wpm * 1.5 => 3,
            Some(true) if accuracy >= 97.0 && wpm >= s.min_wpm * 1.2 => 2,
            Some(true) => 1,
            None if accuracy >= 98.0 => 3,
            None if accuracy >= 95.0 => 2,
            None if accuracy >= 90.0 => 1,
            None => 0,
        };

        self.outcome = Some(Outcome {
            kind: s.kind,
            title: s.title,
            wpm,
            accuracy,
            seconds,
            chars: s.correct,
            errors: s.errors,
            stars,
            passed,
            min_wpm: s.min_wpm,
            min_acc: s.min_acc,
            messages,
            tricky,
        });
        self.screen = Screen::Result;
        self.save();
    }
}
