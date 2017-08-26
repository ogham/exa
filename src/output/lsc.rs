#![allow(dead_code)]

use std::collections::HashMap;

use ansi_term::Style;
use ansi_term::Colour::*;


pub struct LSColors<'var> {
    contents: HashMap<&'var str, &'var str>
}

impl<'var> LSColors<'var> {
    pub fn parse(input: &'var str) -> LSColors<'var> {
        let contents = input.split(":")
                            .flat_map(|mapping| {

            let bits = mapping.split("=")
                              .take(3)
                              .collect::<Vec<_>>();

            if bits.len() != 2 || bits[0].is_empty() || bits[1].is_empty() { None }
            else { Some( (bits[0], bits[1]) ) }
        }).collect();
        LSColors { contents }
    }

    pub fn get(&self, facet_name: &str) -> Option<Style> {
        self.contents.get(facet_name).map(ansi_to_style)
    }
}

fn ansi_to_style(ansi: &&str) -> Style {
    let mut style = Style::default();

    for num in ansi.split(";") {
        match num {

            // Bold and italic
            "1"  => style = style.bold(),
            "4"  => style = style.underline(),

            // Foreground colours
            "30" => style = style.fg(Black),
            "31" => style = style.fg(Red),
            "32" => style = style.fg(Green),
            "33" => style = style.fg(Yellow),
            "34" => style = style.fg(Blue),
            "35" => style = style.fg(Purple),
            "36" => style = style.fg(Cyan),
            "37" => style = style.fg(White),

            // Background colours
            "40" => style = style.on(Black),
            "41" => style = style.on(Red),
            "42" => style = style.on(Green),
            "43" => style = style.on(Yellow),
            "44" => style = style.on(Blue),
            "45" => style = style.on(Purple),
            "46" => style = style.on(Cyan),
            "47" => style = style.on(White),
             _    => {/* ignore the error and do nothing */},
        }
    }

    style
}


#[cfg(test)]
mod ansi_test {
    use super::*;
    use ansi_term::Style;

    macro_rules! test {
        ($name:ident: $input:expr => $result:expr) => {
            #[test]
            fn $name() {
                assert_eq!(ansi_to_style(&$input), $result);
            }
        };
    }

    // Styles
    test!(bold:  "1"         => Style::default().bold());
    test!(under: "4"         => Style::default().underline());
    test!(both:  "1;4"       => Style::default().bold().underline());
    test!(fg:    "31"        => Red.normal());
    test!(bg:    "43"        => Style::default().on(Yellow));
    test!(bfg:   "31;43"     => Red.on(Yellow));
    test!(all:   "43;31;1;4" => Red.on(Yellow).bold().underline());
    test!(again: "1;1;1;1;1" => Style::default().bold());

    // Failure cases
    test!(empty: ""          => Style::default());
    test!(semis: ";;;;;;"    => Style::default());
    test!(nines: "99999999"  => Style::default());
    test!(word:  "GREEN"     => Style::default());
}



#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test {
        ($name:ident: $input:expr, $facet:expr => $result:expr) => {
            #[test]
            fn $name() {
                let lsc = LSColors::parse($input);
                assert_eq!(lsc.get($facet), $result.into());
                assert_eq!(lsc.get(""), None);
            }
        };
    }

    // Bad parses
    test!(empty:    "",       "di" => None);
    test!(jibber:   "blah",   "di" => None);

    test!(equals:     "=",    "di" => None);
    test!(starts:     "=di",  "di" => None);
    test!(ends:     "id=",    "id" => None);

    // Foreground colours
    test!(red:     "di=31",   "di" => Red.normal());
    test!(green:   "cb=32",   "cb" => Green.normal());
    test!(blue:    "la=34",   "la" => Blue.normal());

    // Background colours
    test!(yellow:  "do=43",   "do" => Style::default().on(Yellow));
    test!(purple:  "re=45",   "re" => Style::default().on(Purple));
    test!(cyan:    "mi=46",   "mi" => Style::default().on(Cyan));

    // Bold and underline
    test!(bold:    "fa=1",    "fa" => Style::default().bold());
    test!(under:   "so=4",    "so" => Style::default().underline());
    test!(both:    "la=1;4",  "la" => Style::default().bold().underline());

    // More and many
    test!(more_1:  "me=43;21;55;34:yu=1;4;1", "me" => Blue.on(Yellow));
    test!(more_2:  "me=43;21;55;34:yu=1;4;1", "yu" => Style::default().bold().underline());

    test!(many_1:  "red=31:green=32:blue=34", "red"   => Red.normal());
    test!(many_2:  "red=31:green=32:blue=34", "green" => Green.normal());
    test!(many_3:  "red=31:green=32:blue=34", "blue"  => Blue.normal());
}
