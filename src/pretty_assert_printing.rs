enum ByteMatch {
    Same(u8),
    Diff { left: Option<u8>, right: Option<u8> },
}

fn check_sameness(a: &[u8], b: &[u8]) -> Vec<ByteMatch> {
    let mut result = Vec::new();

    for i in 0..a.len().max(b.len()) {
        let left = if a.len() <= i { None } else { Some(a[i]) };
        let right = if b.len() <= i { None } else { Some(b[i]) };
        match (left, right) {
            (Some(left), Some(right)) if left == right => result.push(ByteMatch::Same(left)),
            _ => result.push(ByteMatch::Diff { left, right }),
        }
    }

    result
}

const COLOR_RESET: &str = "\x1b[0m";
const COLOR_RED: &str = "\x1b[31m";
const COLOR_GREEN: &str = "\x1b[32m";

pub fn soft_assert(title: &str, actual: &[u8], expected: &[u8]) {
    let bytematch_result = check_sameness(actual, expected);

    let all_match = bytematch_result
        .iter()
        .all(|b| matches!(b, ByteMatch::Same(_)));

    println!("{} {}", if all_match { "✅" } else { "❌" }, title);
    print!("   Expected: [");
    println!(
        "{}]",
        expected
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ")
    );

    fn hex_colored(b: &u8, color: &str) -> String {
        format!("{}{:02x}{}", color, b, COLOR_RESET)
    }

    let actual_colored: Vec<_> = bytematch_result
        .iter()
        .map(|b| match b {
            ByteMatch::Same(b) => hex_colored(b, COLOR_GREEN),
            ByteMatch::Diff {
                left: Some(left),
                right: _,
            } => hex_colored(left, COLOR_RED),
            _ => format!("{}__{}", COLOR_RED, COLOR_RESET),
        })
        .collect();

    println!("   Actual  : [{}]", actual_colored.join(" "));
}
