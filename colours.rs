pub enum Colour {
    // These are the standard numeric sequences.
    // See http://invisible-island.net/xterm/ctlseqs/ctlseqs.html
    Black = 30, Red = 31, Green = 32, Yellow = 33, Blue = 34, Purple = 35, Cyan = 36, White = 37,
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
                        Some(c) => format!("{};", c as int + 10),
                        None => "".to_string()
                    };
                    let bo = if bold { "1;" } else { "" };
                    let un = if underline { "4;" } else { "" };
                    let painted = format!("\x1B[{}{}{}{}m{}\x1B[0m", bo, un, bg, foreground as int, input.to_string());
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
        Style(st) => Style(StyleStruct     { foreground: st.foreground, background: st.background, bold: true, underline: false }),
      }
    }

    pub fn underline(&self) -> Style {
      match *self {
        Plain => Style(StyleStruct         { foreground: White,         background: None,          bold: false, underline: true }),
        Foreground(c) => Style(StyleStruct { foreground: c,             background: None,          bold: false, underline: true }),
        Style(st) => Style(StyleStruct     { foreground: st.foreground, background: st.background, bold: false, underline: true }),
      }
    }

    pub fn on(&self, background: Colour) -> Style {
      match *self {
        Plain => Style(StyleStruct         { foreground: White,         background: Some(background), bold: false, underline: false }),
        Foreground(c) => Style(StyleStruct { foreground: c,             background: Some(background), bold: false, underline: false }),
        Style(st) => Style(StyleStruct     { foreground: st.foreground, background: Some(background), bold: false, underline: false }),
      }
    }
}

impl Colour {

    // This is a short-cut so you don't have to use Blue.normal() just
    // to turn Blue into a Style.
    pub fn paint(&self, input: &str) -> String {
        let re = format!("\x1B[{}m{}\x1B[0m", *self as int, input);
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
