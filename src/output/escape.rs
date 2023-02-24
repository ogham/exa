use ansi_term::{ANSIString, Style};


pub fn escape(string: String, bits: &mut Vec<ANSIString<'_>>, good: Style, bad: Style) {
    let needs_quotes = string.contains(' ') || string.contains('\'');
    let quote_bit = good.paint(if string.contains('\'') { "\"" } else { "\'" });

    if string.chars().all(|c| c >= 0x20 as char && c != 0x7f as char) {
        bits.push(good.paint(string));
    }
    else {
        for c in string.chars() {
            // The `escape_default` method on `char` is *almost* what we want here, but
            // it still escapes non-ASCII UTF-8 characters, which are still printable.

            if c >= 0x20 as char && c != 0x7f as char {
                // TODO: This allocates way too much,
                // hence the `all` check above.
                let mut s = String::new();
                s.push(c);
                bits.push(good.paint(s));
            }
            else {
                let s = c.escape_default().collect::<String>();
                bits.push(bad.paint(s));
            }
        }
    }

    if needs_quotes {
        bits.insert(0, quote_bit.clone());
        bits.push(quote_bit);
    }
}
