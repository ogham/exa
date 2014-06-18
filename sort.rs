use std::ascii::StrAsciiExt;

// This is an implementation of "natural sort order". See
// http://blog.codinghorror.com/sorting-for-humans-natural-sort-order/
// for more information and examples. It tries to sort "9" before
// "10", which makes sense to those regular human types.

// It works by splitting an input string into several parts, and then
// comparing based on those parts. A SortPart derives TotalOrd, so a
// Vec<SortPart> will automatically have natural sorting.

#[deriving(Eq, Ord, PartialEq, PartialOrd)]
pub enum SortPart {
    Numeric(u64),
    Stringular(String),
}

impl SortPart {
    pub fn from_string(is_digit: bool, slice: &str) -> SortPart {
        if is_digit {
            Numeric(from_str::<u64>(slice).expect(slice))
        } else {
            Stringular(slice.to_ascii_lower())
        }
    }

    // The logic here is taken from my question at
    // http://stackoverflow.com/q/23969191/3484614

    pub fn split_into_parts(input: &str) -> Vec<SortPart> {
        let mut parts = vec![];

        if input.is_empty() {
            return parts
        }

        let mut is_digit = input.char_at(0).is_digit();
        let mut start = 0;

        for (i, c) in input.char_indices() {
            if is_digit != c.is_digit() {
                parts.push(SortPart::from_string(is_digit, input.slice(start, i)));
                is_digit = !is_digit;
                start = i;
            }
        }

        parts.push(SortPart::from_string(is_digit, input.slice_from(start)));
        parts
    }
}
