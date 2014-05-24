fn formatBytes(mut amount: u64, kilo: u64, prefixes: ~[&str]) -> StrBuf {
    let mut prefix = 0;
    while amount > kilo {
        amount /= kilo;
        prefix += 1;
    }
    return format!("{:4}{}", amount, prefixes[prefix]);
}

pub fn formatBinaryBytes(amount: u64) -> StrBuf {
    formatBytes(amount, 1024, ~[ "B  ", "KiB", "MiB", "GiB", "TiB" ])
}

pub fn formatDecimalBytes(amount: u64) -> StrBuf {
    formatBytes(amount, 1000, ~[ "B ", "KB", "MB", "GB", "TB" ])
}
