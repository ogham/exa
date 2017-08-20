use std::collections::HashMap;

use ansi_term::Style;
use ansi_term::Colour::*;


pub struct LSColors<'var> {
	contents: HashMap<&'var str, &'var str>
}

impl<'var> LSColors<'var> {
    fn parse(input: &'var str) -> LSColors<'var> {
        let contents = input.split(":")
                            .flat_map(|mapping| {

            let bits = mapping.split("=")
                              .take(3)
                              .collect::<Vec<_>>();

            if bits.len() == 2 { Some((bits[0], bits[1])) }
                          else { None }
        }).collect();
        LSColors { contents }
    }

    fn get(&self, facet_name: &str) -> Option<Style> {
        self.contents.get(facet_name).and_then(ansi_to_style)
    }
}

fn ansi_to_style(ansi: &&str) -> Option<Style> {
    match *ansi {
        "31" => Some(Red.normal()),
        "34" => Some(Blue.normal()),
        _ => None,
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_empty() {
        let lsc = LSColors::parse("");
        assert_eq!(lsc.get("di"), None);
        assert_eq!(lsc.get(""), None);
    }

    #[test]
    fn parse_gibberish() {
        let lsc = LSColors::parse("gibberish");
        assert_eq!(lsc.get("di"), None);
        assert_eq!(lsc.get("gibberish"), None);
    }


    #[test]
    fn parse_one() {
        let lsc = LSColors::parse("di=34");
        assert_eq!(lsc.get("di"), Some(Blue.normal()));
        assert_eq!(lsc.get("ln"), None);
    }
    
    #[test]
    fn parse_and_ignore_one() {
        let lsc = LSColors::parse("di=34=56");
        assert_eq!(lsc.get("di"), None);
    }

    #[test]
    fn parse_and_ignore_again() {
        let lsc = LSColors::parse("di=");
        assert_eq!(lsc.get("di"), None);
    }
    
    #[test]
    fn parse_and_ignore_other() {
        let lsc = LSColors::parse("=id");
        assert_eq!(lsc.get("di"), None);
    }


    #[test]
    fn parse_and_ignore_equals() {
        let lsc = LSColors::parse("=");
        assert_eq!(lsc.get("di"), None);
    }

    
    #[test]
    fn parse_two() {
        let lsc = LSColors::parse("di=34:ln=31");
        assert_eq!(lsc.get("di"), Some(Blue.normal()));
        assert_eq!(lsc.get("ln"), Some(Red.normal()));
        assert_eq!(lsc.get("cd"), None);
    }
}
