static METRIC_PREFIXES: &'static [&'static str] = &[
    "", "K", "M", "G", "T", "P", "E", "Z", "Y"
];

static IEC_PREFIXES: &'static [&'static str] = &[
    "", "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi", "Yi"
];

fn formatBytes(mut amount: u64, kilo: u64, prefixes: &[&str]) -> String {
    let mut prefix = 0;
    while amount > kilo {
        amount /= kilo;
        prefix += 1;
    }
    format!("{}{}", amount, prefixes[prefix])
}

pub fn format_IEC_bytes(amount: u64) -> String {
    formatBytes(amount, 1024, IEC_PREFIXES)
}

pub fn format_metric_bytes(amount: u64) -> String {
    formatBytes(amount, 1000, METRIC_PREFIXES)
}
