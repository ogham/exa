pub enum Colour {
    Black = 30, Red = 31, Green = 32, Yellow = 33, Blue = 34, Purple = 35, Cyan = 36, White = 37,
}

pub enum Style {
    Plain,
    Foreground(Colour),
    Style(StyleStruct),
}

struct StyleStruct {
    foreground: Colour,
    background: Option<Colour>,
    bold: bool,
    underline: bool,
}

impl Style {
    pub fn paint(&self, input: ~str) -> ~str {
        match *self {
            Plain => input,
            Foreground(c) => c.paint(input),
            Style(s) => match s {
                StyleStruct { foreground, background, bold, underline } => {
                    let bg: ~str = match background {
                        Some(c) => format!("{};", c as int + 10),
                        None => ~"",
                    };
                    let bo: ~str = if bold { ~"1;" } else { ~"" };
                    let un: ~str = if underline { ~"4;" } else { ~"" };
                    format!("\x1B[{}{}{}{}m{}\x1B[0m", bo, un, bg, foreground as int, input)
                }
            }
        }
    }
}

impl Style {
    pub fn bold(&self) -> Style {
      match *self {
        Plain => Style(StyleStruct { foreground: White, background: None, bold: true, underline: false }),
        Foreground(c) => Style(StyleStruct { foreground: c, background: None, bold: true, underline: false }),
        Style(st) => Style(StyleStruct { foreground: st.foreground, background: st.background, bold: true, underline: false }),
      }
    }

    pub fn underline(&self) -> Style {
      match *self {
        Plain => Style(StyleStruct { foreground: White, background: None, bold: false, underline: true }),
        Foreground(c) => Style(StyleStruct { foreground: c, background: None, bold: false, underline: true }),
        Style(st) => Style(StyleStruct { foreground: st.foreground, background: st.background, bold: false, underline: true }),
      }
    }

    pub fn on(&self, background: Colour) -> Style {
      match *self {
        Plain => Style(StyleStruct { foreground: White, background: Some(background), bold: false, underline: false }),
        Foreground(c) => Style(StyleStruct { foreground: c, background: Some(background), bold: false, underline: false }),
        Style(st) => Style(StyleStruct { foreground: st.foreground, background: Some(background), bold: false, underline: false }),
      }
    }
}

impl Colour {
    pub fn paint(&self, input: &str) -> ~str {
        format!("\x1B[{}m{}\x1B[0m", *self as int, input)
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
