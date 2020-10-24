use std::iter::Peekable;
use std::ops::FnMut;

use ansi_term::{Colour, Style};
use ansi_term::Colour::*;


// Parsing the LS_COLORS environment variable into a map of names to Style values.
//
// This is sitting around undocumented at the moment because it’s a feature
// that should really be unnecessary! exa highlights its output by creating a
// theme of one Style value per part of the interface that can be coloured,
// then reading styles from that theme. The LS_COLORS variable, on the other
// hand, can contain arbitrary characters that ls is supposed to add to the
// output, without needing to know what they actually do. This puts exa in the
// annoying position of having to parse the ANSI escape codes _back_ into
// Style values before it’s able to use them. Doing this has a lot of
// downsides: if a new terminal feature is added with its own code, exa won’t
// be able to use this without explicit support for parsing the feature, while
// ls would not even need to know it existed. And there are some edge cases in
// ANSI codes, where terminals would accept codes exa is strict about it. It’s
// just not worth doing, and there should really be a way to just use slices
// of the LS_COLORS string without having to parse them.

pub struct LSColors<'var>(pub &'var str);

impl<'var> LSColors<'var> {
    pub fn each_pair<C>(&mut self, mut callback: C)
    where C: FnMut(Pair<'var>)
    {
        for next in self.0.split(':') {
            let bits = next.split('=')
                           .take(3)
                           .collect::<Vec<_>>();

            if bits.len() == 2 && ! bits[0].is_empty() && ! bits[1].is_empty() {
                callback(Pair { key: bits[0], value: bits[1] });
            }
        }
    }
}


fn parse_into_high_colour<'a, I>(iter: &mut Peekable<I>) -> Option<Colour>
where I: Iterator<Item = &'a str>
{
    match iter.peek() {
        Some(&"5") => {
            let _ = iter.next();
            if let Some(byte) = iter.next() {
                if let Ok(num) = byte.parse() {
                    return Some(Fixed(num));
                }
            }
        }

        Some(&"2") => {
            let _ = iter.next();
            if let Some(hexes) = iter.next() {
                // Some terminals support R:G:B instead of R;G;B
                // but this clashes with splitting on ‘:’ in each_pair above.
                /*if hexes.contains(':') {
                    let rgb = hexes.splitn(3, ':').collect::<Vec<_>>();
                    if rgb.len() != 3 {
                        return None;
                    }
                    else if let (Ok(r), Ok(g), Ok(b)) = (rgb[0].parse(), rgb[1].parse(), rgb[2].parse()) {
                        return Some(RGB(r, g, b));
                    }
                }*/

                if let (Some(r), Some(g), Some(b)) = (hexes.parse().ok(),
                                                      iter.next().and_then(|s| s.parse().ok()),
                                                      iter.next().and_then(|s| s.parse().ok()))
                {
                    return Some(RGB(r, g, b));
                }
            }
        }

        _ => {},
    }

    None
}


pub struct Pair<'var> {
    pub key: &'var str,
    pub value: &'var str,
}

impl<'var> Pair<'var> {
    pub fn to_style(&self) -> Style {
        let mut style = Style::default();
        let mut iter = self.value.split(';').peekable();

        while let Some(num) = iter.next() {
            match num.trim_start_matches('0') {

                // Bold and italic
                "1" => style = style.bold(),
                "2" => style = style.dimmed(),
                "3" => style = style.italic(),
                "4" => style = style.underline(),
                "5" => style = style.blink(),
                // 6 is supposedly a faster blink
                "7" => style = style.reverse(),
                "8" => style = style.hidden(),
                "9" => style = style.strikethrough(),

                // Foreground colours
                "30" => style = style.fg(Black),
                "31" => style = style.fg(Red),
                "32" => style = style.fg(Green),
                "33" => style = style.fg(Yellow),
                "34" => style = style.fg(Blue),
                "35" => style = style.fg(Purple),
                "36" => style = style.fg(Cyan),
                "37" => style = style.fg(White),
                "38" => if let Some(c) = parse_into_high_colour(&mut iter) { style = style.fg(c) },

                // Background colours
                "40" => style = style.on(Black),
                "41" => style = style.on(Red),
                "42" => style = style.on(Green),
                "43" => style = style.on(Yellow),
                "44" => style = style.on(Blue),
                "45" => style = style.on(Purple),
                "46" => style = style.on(Cyan),
                "47" => style = style.on(White),
                "48" => if let Some(c) = parse_into_high_colour(&mut iter) { style = style.on(c) },

                 _   => {/* ignore the error and do nothing */},
            }
        }

        style
    }
}


#[cfg(test)]
mod ansi_test {
    use super::*;
    use ansi_term::Style;

    macro_rules! test {
        ($name:ident: $input:expr => $result:expr) => {
            #[test]
            fn $name() {
                assert_eq!(Pair { key: "", value: $input }.to_style(), $result);
            }
        };
    }

    // Styles
    test!(bold:  "1"         => Style::default().bold());
    test!(bold2: "01"        => Style::default().bold());
    test!(under: "4"         => Style::default().underline());
    test!(unde2: "04"        => Style::default().underline());
    test!(both:  "1;4"       => Style::default().bold().underline());
    test!(both2: "01;04"     => Style::default().bold().underline());
    test!(fg:    "31"        => Red.normal());
    test!(bg:    "43"        => Style::default().on(Yellow));
    test!(bfg:   "31;43"     => Red.on(Yellow));
    test!(bfg2:  "0031;0043" => Red.on(Yellow));
    test!(all:   "43;31;1;4" => Red.on(Yellow).bold().underline());
    test!(again: "1;1;1;1;1" => Style::default().bold());

    // Failure cases
    test!(empty: ""          => Style::default());
    test!(semis: ";;;;;;"    => Style::default());
    test!(nines: "99999999"  => Style::default());
    test!(word:  "GREEN"     => Style::default());

    // Higher colours
    test!(hifg:  "38;5;149"  => Fixed(149).normal());
    test!(hibg:  "48;5;1"    => Style::default().on(Fixed(1)));
    test!(hibo:  "48;5;1;1"  => Style::default().on(Fixed(1)).bold());
    test!(hiund: "4;48;5;1"  => Style::default().on(Fixed(1)).underline());

    test!(rgb:   "38;2;255;100;0"     => Style::default().fg(RGB(255, 100, 0)));
    test!(rgbi:  "38;2;255;100;0;3"   => Style::default().fg(RGB(255, 100, 0)).italic());
    test!(rgbbg: "48;2;255;100;0"     => Style::default().on(RGB(255, 100, 0)));
    test!(rgbbi: "48;2;255;100;0;3"   => Style::default().on(RGB(255, 100, 0)).italic());

    test!(fgbg:  "38;5;121;48;5;212"  => Fixed(121).on(Fixed(212)));
    test!(bgfg:  "48;5;121;38;5;212"  => Fixed(212).on(Fixed(121)));
    test!(toohi: "48;5;999"           => Style::default());
}


#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test {
        ($name:ident: $input:expr => $result:expr) => {
            #[test]
            fn $name() {
                let mut lscs = Vec::new();
                LSColors($input).each_pair(|p| lscs.push( (p.key.clone(), p.to_style()) ));
                assert_eq!(lscs, $result.to_vec());
            }
        };
    }

    // Bad parses
    test!(empty:    ""       => []);
    test!(jibber:   "blah"   => []);

    test!(equals:     "="    => []);
    test!(starts:     "=di"  => []);
    test!(ends:     "id="    => []);

    // Foreground colours
    test!(green:   "cb=32"   => [ ("cb", Green.normal()) ]);
    test!(red:     "di=31"   => [ ("di", Red.normal()) ]);
    test!(blue:    "la=34"   => [ ("la", Blue.normal()) ]);

    // Background colours
    test!(yellow:  "do=43"   => [ ("do", Style::default().on(Yellow)) ]);
    test!(purple:  "re=45"   => [ ("re", Style::default().on(Purple)) ]);
    test!(cyan:    "mi=46"   => [ ("mi", Style::default().on(Cyan)) ]);

    // Bold and underline
    test!(bold:    "fa=1"    => [ ("fa", Style::default().bold()) ]);
    test!(under:   "so=4"    => [ ("so", Style::default().underline()) ]);
    test!(both:    "la=1;4"  => [ ("la", Style::default().bold().underline()) ]);

    // More and many
    test!(more:  "me=43;21;55;34:yu=1;4;1"  => [ ("me", Blue.on(Yellow)), ("yu", Style::default().bold().underline()) ]);
    test!(many:  "red=31:green=32:blue=34"  => [ ("red", Red.normal()), ("green", Green.normal()), ("blue", Blue.normal()) ]);
}
