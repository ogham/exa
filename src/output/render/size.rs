use ansi_term::Style;
use locale::Numeric as NumericLocale;

use fs::fields as f;
use output::cell::{TextCell, DisplayWidth};
use output::table::SizeFormat;



impl f::Size {
    pub fn render<C: Colours>(&self, colours: &C, size_format: SizeFormat, numerics: &NumericLocale) -> TextCell {
        use number_prefix::{binary_prefix, decimal_prefix};
        use number_prefix::{Prefixed, Standalone, PrefixNames};

        let size = match *self {
            f::Size::Some(s)             => s,
            f::Size::None                => return TextCell::blank(colours.no_size()),
            f::Size::DeviceIDs(ref ids)  => return ids.render(colours),
        };

        let result = match size_format {
            SizeFormat::DecimalBytes  => decimal_prefix(size as f64),
            SizeFormat::BinaryBytes   => binary_prefix(size as f64),
            SizeFormat::JustBytes     => {
                let string = numerics.format_int(size);
                return TextCell::paint(colours.size(size), string);
            },
        };

        let (prefix, n) = match result {
            Standalone(b)  => return TextCell::paint(colours.size(b as u64), b.to_string()),
            Prefixed(p, n) => (p, n)
        };

        let symbol = prefix.symbol();
        let number = if n < 10f64 { numerics.format_float(n, 1) }
                             else { numerics.format_int(n as isize) };

        // The numbers and symbols are guaranteed to be written in ASCII, so
        // we can skip the display width calculation.
        let width = DisplayWidth::from(number.len() + symbol.len());

        TextCell {
            width:    width,
            contents: vec![
                colours.size(size).paint(number),
                colours.unit().paint(symbol),
            ].into(),
        }
    }
}


impl f::DeviceIDs {
    fn render<C: Colours>(&self, colours: &C) -> TextCell {
        let major = self.major.to_string();
        let minor = self.minor.to_string();

        TextCell {
            width: DisplayWidth::from(major.len() + 1 + minor.len()),
            contents: vec![
                colours.major().paint(major),
                colours.comma().paint(","),
                colours.minor().paint(minor),
            ].into(),
        }
    }
}


pub trait Colours {
	fn size(&self, size: u64) -> Style;
	fn unit(&self) -> Style;
	fn no_size(&self) -> Style;

	fn major(&self) -> Style;
	fn comma(&self) -> Style;
	fn minor(&self) -> Style;
}


#[cfg(test)]
pub mod test {
	use super::Colours;
    use output::cell::{TextCell, DisplayWidth};
    use output::table::SizeFormat;
    use fs::fields as f;

    use locale::Numeric as NumericLocale;
    use ansi_term::Colour::*;
    use ansi_term::Style;
    
    
    struct TestColours;
    
    impl Colours for TestColours {
    	fn size(&self, _size: u64) -> Style { Fixed(66).normal() }
    	fn unit(&self)             -> Style { Fixed(77).bold() }
    	fn no_size(&self)          -> Style { Black.italic() }
    	
    	fn major(&self) -> Style { Blue.on(Red) }
    	fn comma(&self) -> Style { Green.italic() }
		fn minor(&self) -> Style { Cyan.on(Yellow) }
    }


    #[test]
    fn directory() {
        let directory = f::Size::None;
        let expected = TextCell::blank(Black.italic());
        assert_eq!(expected, directory.render(&TestColours, SizeFormat::JustBytes, &NumericLocale::english()))
    }


    #[test]
    fn file_decimal() {
        let directory = f::Size::Some(2_100_000);
        let expected = TextCell {
            width: DisplayWidth::from(4),
            contents: vec![
                Fixed(66).paint("2.1"),
                Fixed(77).bold().paint("M"),
            ].into(),
        };

        assert_eq!(expected, directory.render(&TestColours, SizeFormat::DecimalBytes, &NumericLocale::english()))
    }


    #[test]
    fn file_binary() {
        let directory = f::Size::Some(1_048_576);
        let expected = TextCell {
            width: DisplayWidth::from(5),
            contents: vec![
                Fixed(66).paint("1.0"),
                Fixed(77).bold().paint("Mi"),
            ].into(),
        };

        assert_eq!(expected, directory.render(&TestColours, SizeFormat::BinaryBytes, &NumericLocale::english()))
    }


    #[test]
    fn file_bytes() {
        let directory = f::Size::Some(1048576);
        let expected = TextCell {
            width: DisplayWidth::from(9),
            contents: vec![
                Fixed(66).paint("1,048,576"),
            ].into(),
        };

        assert_eq!(expected, directory.render(&TestColours, SizeFormat::JustBytes, &NumericLocale::english()))
    }


    #[test]
    fn device_ids() {
        let directory = f::Size::DeviceIDs(f::DeviceIDs { major: 10, minor: 80 });
        let expected = TextCell {
            width: DisplayWidth::from(5),
            contents: vec![
                Blue.on(Red).paint("10"),
                Green.italic().paint(","),
                Cyan.on(Yellow).paint("80"),
            ].into(),
        };

        assert_eq!(expected, directory.render(&TestColours, SizeFormat::JustBytes, &NumericLocale::english()))
    }
}
