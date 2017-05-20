use fs::fields as f;
use output::column::SizeFormat;
use output::cell::{TextCell, DisplayWidth};
use output::colours::Colours;
use locale;


impl f::Size {
    pub fn render(&self, colours: &Colours, size_format: SizeFormat, numerics: &locale::Numeric) -> TextCell {
        use number_prefix::{binary_prefix, decimal_prefix};
        use number_prefix::{Prefixed, Standalone, PrefixNames};

        let size = match *self {
            f::Size::Some(s)                     => s,
            f::Size::None                        => return TextCell::blank(colours.punctuation),
            f::Size::DeviceIDs { major, minor }  => return render_device_ids(colours, major, minor),
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

fn render_device_ids(colours: &Colours, major: u8, minor: u8) -> TextCell {
    let major = major.to_string();
    let minor = minor.to_string();

    TextCell {
        width: DisplayWidth::from(major.len() + 1 + minor.len()),
        contents: vec![
            colours.size.major.paint(major),
            colours.punctuation.paint(","),
            colours.size.minor.paint(minor),
        ].into(),
    }
}
