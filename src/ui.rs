//! Rendering of all screens.

use std::collections::{BTreeMap, HashSet};
use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier as Mod, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Gauge, Padding, Paragraph, Sparkline, Wrap};

use crate::app::{App, Screen};
use crate::layout::{self, Finger, Modifier, ROWS};
use crate::session::Session;
use crate::state::{self, KeyStat};

const ACCENT: Color = Color::Rgb(255, 196, 87);
const DIM: Color = Color::Rgb(110, 110, 120);
const GOOD: Color = Color::Rgb(120, 200, 120);
const BAD: Color = Color::Rgb(240, 110, 100);
const KEY_BG: Color = Color::Rgb(40, 42, 48);

fn finger_color(f: Finger) -> Color {
    match f {
        Finger::LPinky => Color::Rgb(186, 134, 230),
        Finger::LRing => Color::Rgb(110, 160, 245),
        Finger::LMiddle => Color::Rgb(80, 200, 190),
        Finger::LIndex => Color::Rgb(130, 210, 120),
        Finger::RIndex => Color::Rgb(200, 225, 110),
        Finger::RMiddle => Color::Rgb(255, 196, 87),
        Finger::RRing => Color::Rgb(250, 130, 160),
        Finger::RPinky => Color::Rgb(160, 140, 250),
        Finger::Thumb => Color::Rgb(200, 200, 200),
    }
}

fn block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(DIM))
        .title(Span::styled(format!(" {title} "), Style::new().fg(ACCENT).bold()))
}

/// Centers a column of at most `width` cells.
fn centered(area: Rect, width: u16) -> Rect {
    let w = width.min(area.width);
    Rect { x: area.x + (area.width - w) / 2, width: w, ..area }
}

pub fn draw(f: &mut Frame, app: &App) {
    let area = centered(f.area(), 78);
    match app.screen {
        Screen::Menu => draw_menu(f, app, area),
        Screen::Typing => {
            if let Some(s) = &app.session {
                draw_typing(f, app, s, area);
            }
        }
        Screen::Result => draw_result(f, app, area),
        Screen::Stats => draw_stats(f, app, area),
        Screen::Help => draw_help(f, area),
    }
}

fn stars(n: u8) -> Span<'static> {
    let s: String = (0..3).map(|i| if i < n { '★' } else { '☆' }).collect();
    Span::styled(s, Style::new().fg(ACCENT).bold())
}

fn draw_menu(f: &mut Frame, app: &App, area: Rect) {
    let p = &app.progress;
    let [head, menu, kb, foot] =
        Layout::vertical([Constraint::Length(6), Constraint::Min(10), Constraint::Length(7), Constraint::Length(2)])
            .areas(area);

    let streak = p.current_streak();
    let main_total = layout::MAIN.len();
    let header = vec![
        Line::from(vec![
            Span::styled("⌨  TYPING COACH", Style::new().fg(ACCENT).bold()),
            Span::styled("  ·  German QWERTZ", Style::new().fg(DIM)),
        ]),
        Line::from(vec![
            Span::raw("Rank: "),
            Span::styled(p.rank(), Style::new().bold()),
            Span::raw(format!("   Best: {:.0} WPM   ", p.best_wpm)),
            Span::styled(
                format!("Streak: {streak} day{}", if streak == 1 { "" } else { "s" }),
                Style::new().fg(if streak > 0 { ACCENT } else { DIM }),
            ),
        ]),
        Line::from(vec![
            Span::raw(format!("Lessons {}/{main_total}", p.main_level.min(main_total))),
            Span::styled(
                if p.code_enabled { format!("   Code {}/{}", p.code_level.min(layout::CODE.len()), layout::CODE.len()) } else { String::new() },
                Style::new(),
            ),
            Span::styled(
                if p.daily_done_today() { "   Daily training ✓" } else if p.main_level > 0 { "   Daily training pending" } else { "" },
                Style::new().fg(if p.daily_done_today() { GOOD } else { ACCENT }),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(header).block(block("Welcome")).alignment(Alignment::Center), head);

    let items = app.menu_items();
    let sel = app.menu_idx.min(items.len() - 1);
    let mut lines = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let selected = i == sel;
        let marker = if selected { "▸ " } else { "  " };
        let style = match (selected, item.enabled) {
            (true, true) => Style::new().fg(Color::Black).bg(ACCENT).bold(),
            (true, false) => Style::new().fg(DIM).bg(KEY_BG),
            (false, true) => Style::new(),
            (false, false) => Style::new().fg(DIM),
        };
        lines.push(Line::from(vec![
            Span::styled(marker, Style::new().fg(ACCENT)),
            Span::styled(format!(" {} ", i + 1), Style::new().fg(DIM)),
            Span::styled(format!(" {:<36}", item.label), style),
            Span::styled(format!("  {}", item.note), Style::new().fg(DIM)),
        ]));
    }
    if let Some(status) = &app.status {
        lines.push(Line::raw(""));
        lines.push(Line::styled(format!("  {status}"), Style::new().fg(ACCENT)));
    }
    f.render_widget(Paragraph::new(lines).block(block("Menu")), menu);

    let learned: HashSet<char> = p.learned_chars().into_iter().collect();
    f.render_widget(
        Paragraph::new(keyboard(None, &learned, None)).block(block("Your keys")).alignment(Alignment::Center),
        kb,
    );
    f.render_widget(
        Paragraph::new(Line::styled("↑↓ select · Enter start · 1-9 quick pick · q quit", Style::new().fg(DIM)))
            .alignment(Alignment::Center),
        foot,
    );
}

/// Splits text into lines of at most `width` chars, breaking after spaces.
fn wrap(text: &[char], width: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = 0;
    while start < text.len() {
        let mut end = (start + width).min(text.len());
        if end < text.len()
            && let Some(sp) = text[start..end].iter().rposition(|&c| c == ' ')
            && sp > 0
        {
            end = start + sp + 1;
        }
        out.push((start, end));
        start = end;
    }
    out
}

fn draw_typing(f: &mut Frame, app: &App, s: &Session, area: Rect) {
    let [head, stats, text, kb, hint, foot] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(2),
        Constraint::Length(5),
        Constraint::Length(7),
        Constraint::Length(2),
        Constraint::Length(1),
    ])
    .areas(area);

    f.render_widget(
        Paragraph::new(Line::styled(&s.hint, Style::new().fg(DIM).italic()))
            .wrap(Wrap { trim: true })
            .block(block(&s.title)),
        head,
    );

    let time = match s.limit {
        Some(l) => {
            let left = l.saturating_sub(s.elapsed());
            format!("{} left", fmt_dur(left))
        }
        None => fmt_dur(s.elapsed()),
    };
    let [stat_line, gauge] = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(stats);
    let acc_color = if s.accuracy() >= s.min_acc.max(90.0) { GOOD } else { BAD };
    let wpm_color = if s.wpm() >= s.min_wpm { GOOD } else { Color::White };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(" WPM "),
            Span::styled(format!("{:5.1}", s.wpm()), Style::new().fg(wpm_color).bold()),
            Span::raw("   Accuracy "),
            Span::styled(format!("{:5.1}%", s.accuracy()), Style::new().fg(acc_color).bold()),
            Span::raw(format!("   Errors {}   ", s.errors)),
            Span::styled(time, Style::new().fg(ACCENT)),
            Span::styled(
                if s.min_wpm > 0.0 { format!("   goal {:.0} WPM / {:.0}%", s.min_wpm, s.min_acc) } else { String::new() },
                Style::new().fg(DIM),
            ),
        ])),
        stat_line,
    );
    f.render_widget(
        Gauge::default().ratio(s.progress()).gauge_style(Style::new().fg(ACCENT).bg(KEY_BG)).label(""),
        gauge,
    );

    let inner_w = text.width.saturating_sub(4).max(10) as usize;
    let lines = wrap(&s.text, inner_w);
    let cur = lines.iter().position(|&(_, e)| s.pos < e).unwrap_or(lines.len().saturating_sub(1));
    let flashing = s.flash.is_some_and(|(_, t)| t.elapsed() < Duration::from_millis(300));
    let mut out = Vec::new();
    for &(a, b) in lines.iter().skip(cur).take(3) {
        let spans: Vec<Span> = (a..b)
            .map(|i| {
                let c = s.text[i];
                if i < s.pos {
                    let color = if s.mistakes_at.contains(&i) { BAD } else { GOOD };
                    Span::styled(c.to_string(), Style::new().fg(color))
                } else if i == s.pos {
                    let shown = if c == ' ' { '␣' } else { c };
                    let bg = if flashing { BAD } else { ACCENT };
                    Span::styled(shown.to_string(), Style::new().fg(Color::Black).bg(bg).bold())
                } else {
                    Span::styled(c.to_string(), Style::new().fg(Color::Rgb(215, 215, 220)))
                }
            })
            .collect();
        out.push(Line::from(spans));
    }
    f.render_widget(Paragraph::new(out).block(block("Type")).style(Style::new()), text);

    let learned: HashSet<char> = app.progress.learned_chars().into_iter().chain(s.text.iter().copied()).collect();
    let next = s.next_char();
    f.render_widget(
        Paragraph::new(keyboard(next, &learned, None)).block(block("Keyboard")).alignment(Alignment::Center),
        kb,
    );

    let mut hint_spans = Vec::new();
    if let Some(c) = next
        && let Some(loc) = layout::locate(c)
    {
        let shown = if c == ' ' { "space".to_string() } else { format!("'{c}'") };
        hint_spans.push(Span::raw("Next: "));
        hint_spans.push(Span::styled(shown, Style::new().bold()));
        hint_spans.push(Span::raw("  →  "));
        hint_spans.push(Span::styled(loc.finger.name(), Style::new().fg(finger_color(loc.finger)).bold()));
        let m = match loc.modifier {
            Modifier::None => "",
            Modifier::LeftShift => "  + left Shift (left pinky)",
            Modifier::RightShift => "  + right Shift (right pinky)",
            Modifier::AltGr => "  + AltGr (right thumb)",
        };
        hint_spans.push(Span::styled(m, Style::new().fg(ACCENT)));
    }
    if flashing && let Some((wrong, _)) = s.flash {
        hint_spans.push(Span::styled(format!("   ✗ you typed '{wrong}'"), Style::new().fg(BAD)));
    }
    let waiting = if s.started.is_none() { "Start typing - the clock starts with your first key." } else { "" };
    f.render_widget(
        Paragraph::new(vec![Line::from(hint_spans), Line::styled(waiting, Style::new().fg(DIM))])
            .alignment(Alignment::Center),
        hint,
    );
    f.render_widget(
        Paragraph::new(Line::styled("Esc abort · eyes on the screen, not the keys", Style::new().fg(DIM)))
            .alignment(Alignment::Center),
        foot,
    );
}

fn fmt_dur(d: Duration) -> String {
    let s = d.as_secs();
    format!("{}:{:02}", s / 60, s % 60)
}

fn heat_color(stat: Option<&KeyStat>) -> Option<Color> {
    let rate = stat.and_then(KeyStat::error_rate)?;
    Some(match rate {
        r if r < 0.02 => Color::Rgb(90, 190, 100),
        r if r < 0.05 => Color::Rgb(170, 210, 90),
        r if r < 0.10 => Color::Rgb(240, 210, 80),
        r if r < 0.20 => Color::Rgb(245, 150, 70),
        _ => Color::Rgb(235, 80, 70),
    })
}

/// Draws the QWERTZ keyboard. `next` is highlighted with its finger color;
/// with `heat` set, keys are colored by error rate instead.
fn keyboard(next: Option<char>, learned: &HashSet<char>, heat: Option<&BTreeMap<char, KeyStat>>) -> Vec<Line<'static>> {
    let loc = next.and_then(layout::locate);
    let modifier = loc.map_or(Modifier::None, |l| l.modifier);
    let mod_cell = |label: &str, width: usize, active: bool| {
        let style = if active {
            Style::new().fg(Color::Black).bg(Color::Rgb(230, 230, 230)).bold()
        } else {
            Style::new().fg(DIM).bg(KEY_BG)
        };
        Span::styled(format!("{label:^width$}"), style)
    };
    let gap = || Span::raw(" ");

    let mut lines = Vec::new();
    for (r, keys) in ROWS.iter().enumerate() {
        let mut spans = Vec::new();
        match r {
            1 => spans.extend([mod_cell("Tab", 5, false), gap()]),
            2 => spans.extend([mod_cell("Caps", 6, false), gap()]),
            3 => spans.extend([mod_cell("⇧", 4, modifier == Modifier::LeftShift), gap()]),
            _ => {}
        }
        for (c, key) in keys.iter().enumerate() {
            let is_target = loc.is_some_and(|l| l.row == r && l.col == c);
            let known = learned.contains(&key.base)
                || learned.contains(&key.shift)
                || key.altgr.is_some_and(|a| learned.contains(&a));
            let label = if key.base.is_alphabetic() && key.base != 'ß' { key.shift } else { key.base };
            let mut style = if is_target {
                Style::new().fg(Color::Black).bg(finger_color(key.finger)).bold()
            } else if let Some(h) = heat {
                let stat = h.get(&key.base);
                match heat_color(stat) {
                    Some(col) => Style::new().fg(Color::Black).bg(col),
                    None => Style::new().fg(DIM).bg(KEY_BG),
                }
            } else if known {
                Style::new().fg(finger_color(key.finger)).bg(KEY_BG).bold()
            } else {
                Style::new().fg(Color::Rgb(75, 75, 85)).bg(KEY_BG)
            };
            if key.base == 'f' || key.base == 'j' {
                style = style.add_modifier(Mod::UNDERLINED);
            }
            spans.push(Span::styled(format!(" {label} "), style));
            spans.push(gap());
        }
        match r {
            0 => spans.push(mod_cell("⌫", 5, false)),
            1 => spans.push(mod_cell("↵", 4, false)),
            2 => spans.push(mod_cell("", 3, false)),
            3 => spans.push(mod_cell("⇧", 8, modifier == Modifier::RightShift)),
            _ => {}
        }
        lines.push(Line::from(spans));
    }
    let space_active = loc.is_some_and(|l| l.row == 4);
    let space_style = if space_active {
        Style::new().fg(Color::Black).bg(finger_color(Finger::Thumb)).bold()
    } else if heat.is_some() {
        heat_color(heat.and_then(|h| h.get(&' '))).map_or(Style::new().bg(KEY_BG), |c| Style::new().bg(c))
    } else {
        Style::new().fg(DIM).bg(KEY_BG)
    };
    lines.push(Line::from(vec![
        mod_cell("Strg", 5, false),
        gap(),
        mod_cell("Alt", 4, false),
        gap(),
        Span::styled(format!("{:^27}", "space"), space_style),
        gap(),
        mod_cell("AltGr", 6, modifier == Modifier::AltGr),
        gap(),
        mod_cell("Strg", 5, false),
    ]));
    lines
}

fn draw_result(f: &mut Frame, app: &App, area: Rect) {
    let Some(o) = &app.outcome else { return };
    let [main, foot] = Layout::vertical([Constraint::Min(14), Constraint::Length(2)]).areas(area);

    let verdict = match o.passed {
        Some(true) => Span::styled("PASSED  ", Style::new().fg(GOOD).bold()),
        Some(false) => Span::styled("NOT YET  ", Style::new().fg(BAD).bold()),
        None => Span::styled("DONE  ", Style::new().fg(GOOD).bold()),
    };
    let goal = |v: f64, unit: &str| if v > 0.0 { format!("   (goal {v:.0}{unit})") } else { String::new() };
    let mut lines = vec![
        Line::from(vec![verdict, stars(o.stars)]),
        Line::raw(""),
        Line::from(vec![
            Span::raw("Speed      "),
            Span::styled(format!("{:5.1} WPM", o.wpm), Style::new().bold()),
            Span::styled(goal(o.min_wpm, " WPM"), Style::new().fg(DIM)),
        ]),
        Line::from(vec![
            Span::raw("Accuracy   "),
            Span::styled(format!("{:5.1} %", o.accuracy), Style::new().bold()),
            Span::styled(goal(o.min_acc, "%"), Style::new().fg(DIM)),
        ]),
        Line::raw(format!(
            "Time       {}   ·   {} chars   ·   {} errors",
            fmt_dur(Duration::from_secs_f64(o.seconds)),
            o.chars,
            o.errors
        )),
    ];
    if !o.tricky.is_empty() {
        let keys: Vec<String> = o
            .tricky
            .iter()
            .map(|(c, n)| format!("{} ×{n}", if *c == ' ' { "space".to_string() } else { c.to_string() }))
            .collect();
        lines.push(Line::from(vec![
            Span::raw("Tricky     "),
            Span::styled(keys.join("   "), Style::new().fg(BAD)),
        ]));
    }
    lines.push(Line::raw(""));
    for m in &o.messages {
        lines.push(Line::styled(m.clone(), Style::new().fg(ACCENT)));
    }
    f.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(block(&o.title).padding(Padding::new(14, 4, 1, 0))),
        main,
    );
    f.render_widget(
        Paragraph::new(Line::styled("Enter menu · r go again", Style::new().fg(DIM))).alignment(Alignment::Center),
        foot,
    );
}

fn draw_stats(f: &mut Frame, app: &App, area: Rect) {
    let p = &app.progress;
    let [summary, spark, kb, weak, foot] = Layout::vertical([
        Constraint::Length(6),
        Constraint::Length(5),
        Constraint::Length(7),
        Constraint::Min(4),
        Constraint::Length(1),
    ])
    .areas(area);

    let total_secs: f64 = p.history.iter().map(|r| r.seconds).sum();
    let today = state::today();
    let calendar: Vec<Span> = (0..14)
        .rev()
        .map(|d| {
            let day = today - chrono::Duration::days(d);
            let done = p.history.iter().any(|r| r.date == day && r.kind == "daily");
            if done { Span::styled("■ ", Style::new().fg(GOOD)) } else { Span::styled("□ ", Style::new().fg(DIM)) }
        })
        .collect();
    let recent: Vec<&state::Record> = p.history.iter().rev().take(10).collect();
    let avg = |f: fn(&state::Record) -> f64| {
        if recent.is_empty() { 0.0 } else { recent.iter().map(|r| f(r)).sum::<f64>() / recent.len() as f64 }
    };
    let lines = vec![
        Line::raw(format!(
            "Rank: {}   ·   Best {:.1} WPM   ·   Streak {} day(s)",
            p.rank(),
            p.best_wpm,
            p.current_streak()
        )),
        Line::raw(format!(
            "Last 10 sessions: {:.1} WPM, {:.1}% accuracy   ·   {} sessions, {} min total",
            avg(|r| r.wpm),
            avg(|r| r.accuracy),
            p.history.len(),
            (total_secs / 60.0).round()
        )),
        Line::from([vec![Span::raw("Daily training, last 14 days: ")], calendar].concat()),
    ];
    f.render_widget(Paragraph::new(lines).block(block("Statistics")), summary);

    let data: Vec<u64> = p.history.iter().rev().take(70).rev().map(|r| r.wpm.round() as u64).collect();
    f.render_widget(
        Sparkline::default().data(&data).style(Style::new().fg(ACCENT)).block(block("WPM per session")),
        spark,
    );

    let learned: HashSet<char> = p.learned_chars().into_iter().collect();
    f.render_widget(
        Paragraph::new(keyboard(None, &learned, Some(&p.keys)))
            .block(block("Error heatmap (green good · red needs work)"))
            .alignment(Alignment::Center),
        kb,
    );

    let learned_vec = p.learned_chars();
    let weakest = p.weakest(&learned_vec, 8);
    let spans: Vec<Line> = weakest
        .iter()
        .filter_map(|(c, _)| {
            let st = p.keys.get(c)?;
            Some(Line::raw(format!(
                "{:>5}  {:4.1}% errors   {:4.0} ms/key   ({} presses)",
                c,
                st.error_rate().unwrap_or(0.0) * 100.0,
                st.ema_ms,
                st.hits + st.misses
            )))
        })
        .collect();
    let body = if spans.is_empty() { vec![Line::styled("No data yet - start with lesson 1!", Style::new().fg(DIM))] } else { spans };
    f.render_widget(Paragraph::new(body).block(block("Keys that need love")), weak);
    f.render_widget(
        Paragraph::new(Line::styled("any key: back", Style::new().fg(DIM))).alignment(Alignment::Center),
        foot,
    );
}

fn draw_help(f: &mut Frame, area: Rect) {
    let text = vec![
        Line::styled("The method", Style::new().fg(ACCENT).bold()),
        Line::raw("1. Lessons add two new keys at a time, starting on the home row with F and J."),
        Line::raw("   Each lesson only uses keys you already know plus the new ones."),
        Line::raw("2. Pass a lesson by reaching its accuracy and speed goal. Accuracy comes first:"),
        Line::raw("   a wrong key does not advance the cursor - you must type the right one."),
        Line::raw("3. Pace: by default at most 2 new lessons per day. Motor skills consolidate"),
        Line::raw("   during sleep, so short daily sessions beat one long marathon."),
        Line::raw("4. The daily 2-minute training mixes everything you've learned and shows your"),
        Line::raw("   weakest keys (most errors, slowest) more often. Keep the streak alive!"),
        Line::raw("5. After the curriculum, speed levels push you from 25 WPM upward in 5 WPM steps."),
        Line::raw("6. Programming track (after Capitals): ( ) { } [ ] = + * < > / \\ | _ # & % $ @ ~"),
        Line::raw(""),
        Line::styled("Tips", Style::new().fg(ACCENT).bold()),
        Line::raw("· Fingers rest on  A S D F  and  J K L Ö . Feel the bumps on F and J."),
        Line::raw("· Never look at the keyboard - the on-screen keyboard shows the next key"),
        Line::raw("  in the color of the finger that should press it."),
        Line::raw("· Shift is pressed with the pinky of the opposite hand; AltGr with the right thumb."),
        Line::raw("· Slow and correct beats fast and sloppy. Speed grows by itself."),
        Line::raw(""),
        Line::styled(format!("Progress is saved to {}", state::Progress::path().display()), Style::new().fg(DIM)),
    ];
    f.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }).block(block("How it works")), area);
}
