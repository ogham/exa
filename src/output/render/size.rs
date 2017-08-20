use fs::fields as f;
use output::cell::{TextCell, DisplayWidth};
use output::colours::Colours;
use output::table::SizeFormat;
use locale;


impl f::Size {
    pub fn render(&self, colours: &Colours, size_format: SizeFormat, numerics: &locale::Numeric) -> TextCell {
        use number_prefix::{binary_prefix, decimal_prefix};
        use number_prefix::{Prefixed, Standalone, PrefixNames};

        let size = match *self {
            f::Size::Some(s)             => s,
            f::Size::None                => return TextCell::blank(colours.punctuation),
            f::Size::DeviceIDs(ref ids)  => return ids.render(colours),
        };

        let result = match size_format {
            SizeFormat::DecimalBytes  => decimal_prefix(size as f64),
            SizeFormat::BinaryBytes   => binary_prefix(size as f64),
            SizeFormat::JustBytes     => {
                let string = numerics.format_int(size);
                return TextCell::paint(colours.file_size(size), string);
            },
        };

        let (prefix, n) = match result {
            Standalone(b)  => return TextCell::paint(colours.file_size(b as u64), b.to_string()),
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
                colours.file_size(size).paint(number),
                colours.size.unit.paint(symbol),
            ].into(),
        }
    }
}

impl f::DeviceIDs {
    fn render(&self, colours: &Colours) -> TextCell {
        let major = self.major.to_string();
        let minor = self.minor.to_string();

        TextCell {
            width: DisplayWidth::from(major.len() + 1 + minor.len()),
            contents: vec![
                colours.size.major.paint(major),
                colours.punctuation.paint(","),
                colours.size.minor.paint(minor),
            ].into(),
        }
    }
}


#[cfg(test)]
pub mod test {
    use output::colours::Colours;
    use output::cell::{TextCell, DisplayWidth};
    use output::table::SizeFormat;
    use fs::fields as f;

    use locale;
    use ansi_term::Colour::*;


    #[test]
    fn directory() {
        let mut colours = Colours::default();
        colours.punctuation = Green.italic();

        let directory = f::Size::None;
        let expected = TextCell::blank(Green.italic());
        assert_eq!(expected, directory.render(&colours, SizeFormat::JustBytes, &locale::Numeric::english()))
    }


    #[test]
    fn file_decimal() {
        let mut colours = Colours::default();
        colours.size.numbers = Blue.on(Red);
        colours.size.unit    = Yellow.bold();

        let directory = f::Size::Some(2_100_000);
        let expected = TextCell {
            width: DisplayWidth::from(4),
            contents: vec![
                Blue.on(Red).paint("2.1"),
                Yellow.bold().paint("M"),
            ].into(),
        };

        assert_eq!(expected, directory.render(&colours, SizeFormat::DecimalBytes, &locale::Numeric::english()))
    }


    #[test]
    fn file_binary() {
        let mut colours = Colours::default();
        colours.size.numbers = Blue.on(Red);
        colours.size.unit    = Yellow.bold();

        let directory = f::Size::Some(1_048_576);
        let expected = TextCell {
            width: DisplayWidth::from(5),
            contents: vec![
                Blue.on(Red).paint("1.0"),
                Yellow.bold().paint("Mi"),
            ].into(),
        };

        assert_eq!(expected, directory.render(&colours, SizeFormat::BinaryBytes, &locale::Numeric::english()))
    }


    #[test]
    fn file_bytes() {
        let mut colours = Colours::default();
        colours.size.numbers = Blue.on(Red);

        let directory = f::Size::Some(1048576);
        let expected = TextCell {
            width: DisplayWidth::from(9),
            contents: vec![
                Blue.on(Red).paint("1,048,576"),
            ].into(),
        };

        assert_eq!(expected, directory.render(&colours, SizeFormat::JustBytes, &locale::Numeric::english()))
    }


    #[test]
    fn device_ids() {
        let mut colours = Colours::default();
        colours.size.major = Blue.on(Red);
        colours.punctuation = Green.italic();
        colours.size.minor = Cyan.on(Yellow);

        let directory = f::Size::DeviceIDs(f::DeviceIDs { major: 10, minor: 80 });
        let expected = TextCell {
            width: DisplayWidth::from(5),
            contents: vec![
                Blue.on(Red).paint("10"),
                Green.italic().paint(","),
                Cyan.on(Yellow).paint("80"),
            ].into(),
        };

        assert_eq!(expected, directory.render(&colours, SizeFormat::JustBytes, &locale::Numeric::english()))
    }
}
