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
    fn from_string(is_digit: bool, slice: &str) -> SortPart {
        if is_digit {
            // numbers too big for a u64 fall back into strings.
            match from_str::<u64>(slice) {
                Some(num) => Numeric(num),
                None => Stringular(slice.to_string()),
            }
        } else {
            Stringular(slice.to_ascii_lower())
        }
    }

    // The logic here is taken from my question at
    // http://stackoverflow.com/q/23969191/3484614

    pub fn split_into_parts(input: String) -> Vec<SortPart> {
        let mut parts = vec![];

        if input.is_empty() {
            return parts
        }

        let mut is_digit = input.as_slice().char_at(0).is_digit();
        let mut start = 0;

        for (i, c) in input.as_slice().char_indices() {
            if is_digit != c.is_digit() {
                parts.push(SortPart::from_string(is_digit, input.as_slice().slice(start, i)));
                is_digit = !is_digit;
                start = i;
            }
        }

        parts.push(SortPart::from_string(is_digit, input.as_slice().slice_from(start)));
        parts
    }
}

#[test]
fn test_numeric() {
    let bits = SortPart::split_into_parts("123456789".to_string());
    assert!(bits == vec![ Numeric(123456789) ]);
}


#[test]
fn test_stringular() {
    let bits = SortPart::split_into_parts("toothpaste".to_string());
    assert!(bits == vec![ Stringular("toothpaste".to_string()) ]);
}

#[test]
fn test_empty() {
    let bits = SortPart::split_into_parts("".to_string());
    assert!(bits == vec![]);
}

#[test]
fn test_one() {
    let bits = SortPart::split_into_parts("123abc123".to_string());
    assert!(bits == vec![ Numeric(123), Stringular("abc".to_string()), Numeric(123) ]);
}

#[test]
fn test_two() {
    let bits = SortPart::split_into_parts("final version 3.pdf".to_string());
    assert!(bits == vec![ Stringular("final version ".to_string()), Numeric(3), Stringular(".pdf".to_string()) ]);
}

#[test]
fn test_huge_number() {
    let bits = SortPart::split_into_parts("9999999999999999999999999999999999999999999999999999999".to_string());
    assert!(bits == vec![ Stringular("9999999999999999999999999999999999999999999999999999999".to_string()) ]);
}

#[test]
fn test_case() {
    let bits = SortPart::split_into_parts("123ABC123".to_string());
    assert!(bits == vec![ Numeric(123), Stringular("abc".to_string()), Numeric(123) ]);
}
