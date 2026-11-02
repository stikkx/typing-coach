//! German QWERTZ keyboard geometry, finger assignment and the lesson curriculum.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Finger {
    LPinky,
    LRing,
    LMiddle,
    LIndex,
    RIndex,
    RMiddle,
    RRing,
    RPinky,
    Thumb,
}

impl Finger {
    pub fn name(self) -> &'static str {
        match self {
            Finger::LPinky => "left pinky",
            Finger::LRing => "left ring finger",
            Finger::LMiddle => "left middle finger",
            Finger::LIndex => "left index finger",
            Finger::RIndex => "right index finger",
            Finger::RMiddle => "right middle finger",
            Finger::RRing => "right ring finger",
            Finger::RPinky => "right pinky",
            Finger::Thumb => "thumb",
        }
    }

    pub fn is_left(self) -> bool {
        matches!(self, Finger::LPinky | Finger::LRing | Finger::LMiddle | Finger::LIndex)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Modifier {
    None,
    LeftShift,
    RightShift,
    AltGr,
}

pub struct Key {
    pub base: char,
    pub shift: char,
    pub altgr: Option<char>,
    pub finger: Finger,
}

const fn k(base: char, shift: char, altgr: Option<char>, finger: Finger) -> Key {
    Key { base, shift, altgr, finger }
}

use Finger::*;

/// The four character rows of a German ISO keyboard (T1 layout).
pub static ROWS: [&[Key]; 4] = [
    &[
        k('^', '°', None, LPinky),
        k('1', '!', None, LPinky),
        k('2', '"', Some('²'), LRing),
        k('3', '§', Some('³'), LMiddle),
        k('4', '$', None, LIndex),
        k('5', '%', None, LIndex),
        k('6', '&', None, RIndex),
        k('7', '/', Some('{'), RIndex),
        k('8', '(', Some('['), RMiddle),
        k('9', ')', Some(']'), RRing),
        k('0', '=', Some('}'), RPinky),
        k('ß', '?', Some('\\'), RPinky),
        k('´', '`', None, RPinky),
    ],
    &[
        k('q', 'Q', Some('@'), LPinky),
        k('w', 'W', None, LRing),
        k('e', 'E', Some('€'), LMiddle),
        k('r', 'R', None, LIndex),
        k('t', 'T', None, LIndex),
        k('z', 'Z', None, RIndex),
        k('u', 'U', None, RIndex),
        k('i', 'I', None, RMiddle),
        k('o', 'O', None, RRing),
        k('p', 'P', None, RPinky),
        k('ü', 'Ü', None, RPinky),
        k('+', '*', Some('~'), RPinky),
    ],
    &[
        k('a', 'A', None, LPinky),
        k('s', 'S', None, LRing),
        k('d', 'D', None, LMiddle),
        k('f', 'F', None, LIndex),
        k('g', 'G', None, LIndex),
        k('h', 'H', None, RIndex),
        k('j', 'J', None, RIndex),
        k('k', 'K', None, RMiddle),
        k('l', 'L', None, RRing),
        k('ö', 'Ö', None, RPinky),
        k('ä', 'Ä', None, RPinky),
        k('#', '\'', None, RPinky),
    ],
    &[
        k('<', '>', Some('|'), LPinky),
        k('y', 'Y', None, LPinky),
        k('x', 'X', None, LRing),
        k('c', 'C', None, LMiddle),
        k('v', 'V', None, LIndex),
        k('b', 'B', None, LIndex),
        k('n', 'N', None, RIndex),
        k('m', 'M', Some('µ'), RIndex),
        k(',', ';', None, RMiddle),
        k('.', ':', None, RRing),
        k('-', '_', None, RPinky),
    ],
];

/// Where a character lives on the keyboard. Row 4 is the space bar.
#[derive(Clone, Copy, Debug)]
pub struct Located {
    pub row: usize,
    pub col: usize,
    pub modifier: Modifier,
    pub finger: Finger,
}

pub fn locate(c: char) -> Option<Located> {
    if c == ' ' {
        return Some(Located { row: 4, col: 0, modifier: Modifier::None, finger: Thumb });
    }
    for (row, keys) in ROWS.iter().enumerate() {
        for (col, key) in keys.iter().enumerate() {
            let modifier = if key.base == c {
                Modifier::None
            } else if key.shift == c {
                // Shift is always pressed by the opposite hand's pinky.
                if key.finger.is_left() { Modifier::RightShift } else { Modifier::LeftShift }
            } else if key.altgr == Some(c) {
                Modifier::AltGr
            } else {
                continue;
            };
            return Some(Located { row, col, modifier, finger: key.finger });
        }
    }
    None
}

pub struct Lesson {
    pub title: &'static str,
    pub chars: &'static str,
    pub min_wpm: f64,
    pub hint: &'static str,
}

const fn l(title: &'static str, chars: &'static str, min_wpm: f64, hint: &'static str) -> Lesson {
    Lesson { title, chars, min_wpm, hint }
}

pub static MAIN: &[Lesson] = &[
    l("f j", "fj", 5.0, "Rest your index fingers on F and J - feel the little bumps. Thumbs on space."),
    l("d k", "dk", 6.0, "Middle fingers rest on D and K, right next to your index fingers."),
    l("s l", "sl", 7.0, "Ring fingers on S and L. Keep your wrists relaxed and floating."),
    l("a ö", "aö", 8.0, "Pinkies on A and Ö - the full German home row: asdf  jklö."),
    l("g h", "gh", 9.0, "Index fingers reach inward to G and H, then snap back home."),
    l("e i", "ei", 10.0, "First top-row reach: middle fingers up to E and I."),
    l("r u", "ru", 11.0, "Index fingers up to R and U. Always return to F and J."),
    l("t z", "tz", 12.0, "QWERTZ! Z sits where US keyboards have Y. Left index T, right index Z."),
    l("w o", "wo", 12.0, "Ring fingers up to W and O."),
    l("q p", "qp", 12.0, "Pinkies up to Q and P. Stretch the finger, keep the hand still."),
    l("ü ä", "üä", 13.0, "The right pinky owns the umlauts: Ü up, Ä right of Ö."),
    l("c ,", "c,", 13.0, "Bottom row: left middle down to C, right middle down to comma."),
    l("v m", "vm", 14.0, "Index fingers down to V and M."),
    l("b n", "bn", 14.0, "Index fingers down and inward to B and N."),
    l("x .", "x.", 15.0, "Ring fingers down to X and the period."),
    l("y -", "y-", 15.0, "QWERTZ! Y is bottom-left (left pinky). Minus belongs to the right pinky."),
    l("ß", "ß", 15.0, "Right pinky reaches up to ß, next to the 0."),
    l("Capitals (Shift)", "ABCDEFGHIJKLMNOPQRSTUVWXYZÄÖÜ", 14.0,
      "Hold Shift with the pinky of the OPPOSITE hand, then strike the letter."),
    l("Numbers 1-5", "12345", 14.0, "Reach straight up from the top row. Left hand: 1 2 3 4 5."),
    l("Numbers 6-0", "67890", 14.0, "Right hand: 6 7 8 9 0. The index finger covers 6 and 7."),
    l("Punctuation ! ? : ;", "!?:;", 15.0, "Shift+1 = !   Shift+ß = ?   Shift+. = :   Shift+, = ;"),
    l("Quotes \" '", "\"'", 15.0, "Shift+2 = \"   Shift+# = '"),
];

/// The programming track becomes available once capitals are learned.
pub const CODE_UNLOCK_LEVEL: usize = 18;

pub static CODE: &[Lesson] = &[
    l("( )", "()", 12.0, "Shift+8 = (   Shift+9 = )"),
    l("{ }", "{}", 9.0, "AltGr+7 = {   AltGr+0 = }   AltGr is the right Alt key - press it with the right thumb."),
    l("[ ]", "[]", 9.0, "AltGr+8 = [   AltGr+9 = ]"),
    l("= + *", "=+*", 10.0, "Shift+0 = '='   + sits right of Ü   Shift++ = *"),
    l("< >", "<>", 10.0, "< is left of Y (left pinky), Shift+< = >"),
    l("/ \\ |", "/\\|", 9.0, "Shift+7 = /   AltGr+ß = \\   AltGr+< = |"),
    l("_ #", "_#", 10.0, "Shift+- = _   # sits right of Ä"),
    l("& % $", "&%$", 10.0, "Shift+6 = &   Shift+5 = %   Shift+4 = $"),
    l("@ ~", "@~", 9.0, "AltGr+Q = @   AltGr++ = ~"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LessonRef {
    Main(usize),
    Code(usize),
}

pub struct LessonInfo {
    pub title: String,
    pub hint: String,
    pub new_chars: Vec<char>,
    pub min_wpm: f64,
    pub min_acc: f64,
    /// Timed lessons run for 60 seconds instead of a fixed text length.
    pub timed: bool,
}

pub fn lesson_info(r: LessonRef) -> LessonInfo {
    let (list, level, speed_base, speed_step, label) = match r {
        LessonRef::Main(n) => (MAIN, n, 25.0, 5.0, "Speed level"),
        LessonRef::Code(n) => (CODE, n, 20.0, 4.0, "Code speed level"),
    };
    if let Some(lesson) = list.get(level) {
        let symbols = matches!(r, LessonRef::Code(_)) || level >= CODE_UNLOCK_LEVEL;
        LessonInfo {
            title: lesson.title.to_string(),
            hint: lesson.hint.to_string(),
            new_chars: lesson.chars.chars().collect(),
            min_wpm: lesson.min_wpm,
            min_acc: if symbols { 90.0 } else { 94.0 },
            timed: false,
        }
    } else {
        // Past the curriculum: open-ended speed levels to grow into a fast typist.
        let n = level - list.len();
        let target = speed_base + speed_step * n as f64;
        LessonInfo {
            title: format!("{label} {}", n + 1),
            hint: format!("Reach {target:.0} WPM for 60 seconds at 95% accuracy. Rhythm beats rush."),
            new_chars: Vec::new(),
            min_wpm: target,
            min_acc: 95.0,
            timed: true,
        }
    }
}

fn chars_of(list: &[Lesson], n: usize) -> Vec<char> {
    list[..n.min(list.len())].iter().flat_map(|l| l.chars.chars()).collect()
}

/// Characters a lesson may use: everything learned before plus the lesson's new keys.
pub fn allowed_for(r: LessonRef, main_level: usize) -> Vec<char> {
    match r {
        LessonRef::Main(n) => chars_of(MAIN, n + 1),
        LessonRef::Code(n) => {
            let mut v = chars_of(MAIN, main_level);
            v.extend(chars_of(CODE, n + 1));
            v
        }
    }
}

pub fn learned(main_level: usize, code_level: Option<usize>) -> Vec<char> {
    let mut v = chars_of(MAIN, main_level);
    if let Some(c) = code_level {
        v.extend(chars_of(CODE, c));
    }
    v
}
