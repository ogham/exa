static METRIC_PREFIXES: &'static [&'static str] = &[
    "", "K", "M", "G", "T", "P", "E", "Z", "Y"
];

static IEC_PREFIXES: &'static [&'static str] = &[
    "", "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi", "Yi"
];

fn format_bytes(mut amount: f64, kilo: f64, prefixes: &[&str]) -> (String, String) {
    let mut prefix = 0;
    while amount >= kilo {
        amount /= kilo;
        prefix += 1;
    }

    if amount < 10.0 && prefix != 0 {
        (format!("{:.1}", amount), prefixes[prefix].to_string())
    }
    else {
        (format!("{:.0}", amount), prefixes[prefix].to_string())
    }
}

#[allow(non_snake_case_functions)]
pub fn format_IEC_bytes(amount: u64) -> (String, String) {
    format_bytes(amount as f64, 1024.0, IEC_PREFIXES)
}

pub fn format_metric_bytes(amount: u64) -> (String, String) {
    format_bytes(amount as f64, 1000.0, METRIC_PREFIXES)
}

#[test]
fn test_0() {
    let kk = format_metric_bytes(0);
    assert!(kk == ("0".to_string(), "".to_string()));
}

#[test]
fn test_999() {
    let kk = format_metric_bytes(999);
    assert!(kk == ("999".to_string(), "".to_string()));
}

#[test]
fn test_1000() {
    let kk = format_metric_bytes(1000);
    assert!(kk == ("1.0".to_string(), "K".to_string()));
}

#[test]
fn test_1030() {
    let kk = format_metric_bytes(1030);
    assert!(kk == ("1.0".to_string(), "K".to_string()));
}

#[test]
fn test_1100() {
    let kk = format_metric_bytes(1100);
    assert!(kk == ("1.1".to_string(), "K".to_string()));
}

#[test]
fn test_1111() {
    let kk = format_metric_bytes(1111);
    assert!(kk == ("1.1".to_string(), "K".to_string()));
}

#[test]
fn test_104857() {
    let kk = format_IEC_bytes(126456);
    assert!(kk == ("123".to_string(), "Ki".to_string()));
}

#[test]
fn test_1048576() {
    let kk = format_IEC_bytes(1048576);
    assert!(kk == ("1.0".to_string(), "Mi".to_string()));
}

