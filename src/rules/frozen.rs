pub struct Region {
    pub content: String,
    pub frozen: bool,
}

/// Splits `source` into alternating formattable / frozen regions.
/// Frozen regions are delimited by `@fmt-off`/`@fmt-on` or the
/// PhpStorm-compatible `@formatter:off`/`@formatter:on` markers.
pub fn split_regions(source: &str) -> Vec<Region> {
    let mut regions: Vec<Region> = Vec::new();
    let mut buf = String::new();
    let mut frozen = false;

    for line in source.lines() {
        let t = line.trim();
        let is_off = t.contains("@fmt-off") || t.contains("@formatter:off");
        let is_on  = t.contains("@fmt-on")  || t.contains("@formatter:on");

        if !frozen && is_off {
            if !buf.is_empty() {
                regions.push(Region { content: std::mem::take(&mut buf), frozen: false });
            }
            frozen = true;
            buf.push_str(line);
            buf.push('\n');
        } else if frozen && is_on {
            buf.push_str(line);
            buf.push('\n');
            regions.push(Region { content: std::mem::take(&mut buf), frozen: true });
            frozen = false;
        } else {
            buf.push_str(line);
            buf.push('\n');
        }
    }

    if !buf.is_empty() {
        regions.push(Region { content: buf, frozen });
    }

    regions
}
