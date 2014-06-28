// Provide standard values for the eight standard colours and custom
// values for up to 256. There are terminals that can do the full RGB
// spectrum, but for something as simple as discerning file types this
// doesn't really seem worth it.

// Bear in mind that the first eight (and their bold variants) are
// user-definable and can look different on different terminals, but
// the other 256 have their values fixed. Prefer using a fixed grey,
// such as Fixed(244), to bold black, as bold black looks really weird
// on some terminals.

pub enum Colour {
    Black, Red, Green, Yellow, Blue, Purple, Cyan, White, Fixed(u8),
}

// These are the standard numeric sequences.
// See http://invisible-island.net/xterm/ctlseqs/ctlseqs.html

impl Colour {
    fn foreground_code(&self) -> String {
        match *self {
            Black => "30".to_string(),
            Red => "31".to_string(),
            Green => "32".to_string(),
            Yellow => "33".to_string(),
            Blue => "34".to_string(),
            Purple => "35".to_string(),
            Cyan => "36".to_string(),
            White => "37".to_string(),
            Fixed(num) => format!("38;5;{}", num),
        }
    }

    fn background_code(&self) -> String {
        match *self {
            Black => "40".to_string(),
            Red => "41".to_string(),
            Green => "42".to_string(),
            Yellow => "43".to_string(),
            Blue => "44".to_string(),
            Purple => "45".to_string(),
            Cyan => "46".to_string(),
            White => "47".to_string(),
            Fixed(num) => format!("48;5;{}", num),
        }
    }        
}

// There are only three different styles: plain (no formatting), only
// a foreground colour, and a catch-all for anything more complicated
// than that. It's technically possible to write other cases such as
// "bold foreground", but probably isn't worth writing all the code.

pub enum Style {
    Plain,
    Foreground(Colour),
    Style(StyleStruct),
}

// Having a struct inside an enum is currently unfinished in Rust, but
// should be put in there when that feature is complete.

pub struct StyleStruct {
    foreground: Colour,
    background: Option<Colour>,
    bold: bool,
    underline: bool,
}

impl Style {
    pub fn paint(&self, input: &str) -> String {
        match *self {
            Plain => input.to_string(),
            Foreground(c) => c.paint(input),
            Style(s) => match s {
                StyleStruct { foreground, background, bold, underline } => {
                    let bg = match background {
                        Some(c) => format!("{};", c.background_code()),
                        None => "".to_string()
                    };
                    let bo = if bold { "1;" } else { "" };
                    let un = if underline { "4;" } else { "" };
                    let painted = format!("\x1B[{}{}{}{}m{}\x1B[0m", bo, un, bg, foreground.foreground_code(), input.to_string());
                    return painted.to_string();
                }
            }
        }
    }
}

impl Style {
    pub fn bold(&self) -> Style {
      match *self {
        Plain => Style(StyleStruct         { foreground: White,         background: None,          bold: true, underline: false }),
        Foreground(c) => Style(StyleStruct { foreground: c,             background: None,          bold: true, underline: false }),
        Style(st) => Style(StyleStruct     { foreground: st.foreground, background: st.background, bold: true, underline: st.underline }),
      }
    }

    pub fn underline(&self) -> Style {
      match *self {
        Plain => Style(StyleStruct         { foreground: White,         background: None,          bold: false,   underline: true }),
        Foreground(c) => Style(StyleStruct { foreground: c,             background: None,          bold: false,   underline: true }),
        Style(st) => Style(StyleStruct     { foreground: st.foreground, background: st.background, bold: st.bold, underline: true }),
      }
    }

    pub fn on(&self, background: Colour) -> Style {
      match *self {
        Plain => Style(StyleStruct         { foreground: White,         background: Some(background), bold: false,   underline: false }),
        Foreground(c) => Style(StyleStruct { foreground: c,             background: Some(background), bold: false,   underline: false }),
        Style(st) => Style(StyleStruct     { foreground: st.foreground, background: Some(background), bold: st.bold, underline: st.underline }),
      }
    }
}

impl Colour {
    // This is a short-cut so you don't have to use Blue.normal() just
    // to turn Blue into a Style. Annoyingly, this means that Blue and
    // Blue.normal() aren't of the same type, but this hasn't been an
    // issue so far.
    
    pub fn paint(&self, input: &str) -> String {
        let re = format!("\x1B[{}m{}\x1B[0m", self.foreground_code(), input);
        return re.to_string();
    }

    pub fn underline(&self) -> Style {
        Style(StyleStruct { foreground: *self, background: None, bold: false, underline: true })
    }

    pub fn bold(&self) -> Style {
        Style(StyleStruct { foreground: *self, background: None, bold: true, underline: false })
    }

    pub fn normal(&self) -> Style {
        Style(StyleStruct { foreground: *self, background: None, bold: false, underline: false })
    }

    pub fn on(&self, background: Colour) -> Style {
        Style(StyleStruct { foreground: *self, background: Some(background), bold: false, underline: false })
    }
}

pub fn strip_formatting(input: &String) -> String {
    let re = regex!("\x1B\\[.+?m");
    re.replace_all(input.as_slice(), "").to_string()
}

#[test]
fn test_red() {
    let hi = Red.paint("hi");
    assert!(hi == "\x1B[31mhi\x1B[0m".to_string());
}

#[test]
fn test_black() {
    let hi = Black.normal().paint("hi");
    assert!(hi == "\x1B[30mhi\x1B[0m".to_string());
}

#[test]
fn test_yellow_bold() {
    let hi = Yellow.bold().paint("hi");
    assert!(hi == "\x1B[1;33mhi\x1B[0m".to_string());
}

#[test]
fn test_yellow_bold_2() {
    let hi = Yellow.normal().bold().paint("hi");
    assert!(hi == "\x1B[1;33mhi\x1B[0m".to_string());
}

#[test]
fn test_blue_underline() {
    let hi = Blue.underline().paint("hi");
    assert!(hi == "\x1B[4;34mhi\x1B[0m".to_string());
}

#[test]
fn test_green_bold_underline() {
    let hi = Green.bold().underline().paint("hi");
    assert!(hi == "\x1B[1;4;32mhi\x1B[0m".to_string());
}

#[test]
fn test_green_bold_underline_2() {
    let hi = Green.underline().bold().paint("hi");
    assert!(hi == "\x1B[1;4;32mhi\x1B[0m".to_string());
}

#[test]
fn test_purple_on_white() {
    let hi = Purple.on(White).paint("hi");
    assert!(hi == "\x1B[47;35mhi\x1B[0m".to_string());
}

#[test]
fn test_purple_on_white_2() {
    let hi = Purple.normal().on(White).paint("hi");
    assert!(hi == "\x1B[47;35mhi\x1B[0m".to_string());
}

#[test]
fn test_cyan_bold_on_white() {
    let hi = Cyan.bold().on(White).paint("hi");
    assert!(hi == "\x1B[1;47;36mhi\x1B[0m".to_string());
}

#[test]
fn test_cyan_underline_on_white() {
    let hi = Cyan.underline().on(White).paint("hi");
    assert!(hi == "\x1B[4;47;36mhi\x1B[0m".to_string());
}

#[test]
fn test_cyan_bold_underline_on_white() {
    let hi = Cyan.bold().underline().on(White).paint("hi");
    assert!(hi == "\x1B[1;4;47;36mhi\x1B[0m".to_string());
}

#[test]
fn test_cyan_underline_bold_on_white() {
    let hi = Cyan.underline().bold().on(White).paint("hi");
    assert!(hi == "\x1B[1;4;47;36mhi\x1B[0m".to_string());
}

#[test]
fn test_fixed() {
    let hi = Fixed(100).paint("hi");
    assert!(hi == "\x1B[38;5;100mhi\x1B[0m".to_string());
}

#[test]
fn test_fixed_on_purple() {
    let hi = Fixed(100).on(Purple).paint("hi");
    assert!(hi == "\x1B[45;38;5;100mhi\x1B[0m".to_string());
}

#[test]
fn test_fixed_on_fixed() {
    let hi = Fixed(100).on(Fixed(200)).paint("hi");
    assert!(hi == "\x1B[48;5;200;38;5;100mhi\x1B[0m".to_string());
}





