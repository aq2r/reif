use regex::Regex;

macro_rules! time_measure {
    ($expr:expr) => {{
        let now = std::time::Instant::now();
        let expr_result = $expr;
        let elp = now.elapsed();

        (expr_result, elp)
    }};
}

fn main() {
    // discord msg url
    let mut input_text_1 = String::new();
    input_text_1 +=
        "https://discord.com/channels/1234567890123456789/1234567890123456789/1234567890123456789";

    let mut input_text_2 = String::new();
    input_text_2 +=
        "https://discord.com/channels/9876543210987654321/9876543210987654321/9876543210987654321";

    let mut input_text_3 = String::new();
    input_text_3 +=
        "https://discord.com/channels/9876543210987654321/9876543210987654321/9876543ABCDE7654321";

    println!("{}", "-".repeat(100));

    let (regex, regex_new_elp) = time_measure! {
        Regex::new(r"^https?://discord\.com/channels(/[0-9]+){3}/?$").unwrap()
    };

    let (reif, reif_new_elp) = time_measure! {
        reif::new!(r"^https?://discord\.com/channels(/[0-9]+){3}/?$")
    };

    println!("regex_new_elp: {regex_new_elp:?}");
    println!("reif_new_elp: {reif_new_elp:?}");

    println!("{}", "-".repeat(100));

    for i in [input_text_1, input_text_2, input_text_3] {
        let (regex_result, regex_match_elp) = time_measure!({ regex.is_match(&i) });
        let (reif_result, reif_match_elp) = time_measure!({ reif.is_match(&i) });

        assert_eq!(reif_result, regex_result);
        println!("{i}: {reif_result}");
        println!("Regex: {regex_match_elp:?}");
        println!("Reif: {reif_match_elp:?}");
    }

    println!("{}", "-".repeat(100));
}

#[test]
fn test_re() {
    let reif = reif::new!("abc");
    let results = [("abc", true), ("abcd", true), ("ab", false), ("def", false)];
    for (input, b) in results {
        assert_eq!(reif.is_match(input), b)
    }

    let reif = reif::new!("a.c");
    let results = [("abc", true), ("axc", true), ("a c", true), ("ac", false)];
    for (input, b) in results {
        assert_eq!(reif.is_match(input), b)
    }

    let reif = reif::new!("a*");
    let results = [
        ("", true),
        ("a", true),
        ("aa", true),
        ("b", true),
        ("ba", true),
    ];
    for (input, b) in results {
        assert_eq!(reif.is_match(input), b)
    }

    let reif = reif::new!("a+");
    let results = [
        ("", false),
        ("a", true),
        ("aa", true),
        ("b", false),
        ("ba", true),
    ];
    for (input, b) in results {
        assert_eq!(reif.is_match(input), b)
    }

    let reif = reif::new!("a?");
    let results = [
        ("", true),
        ("a", true),
        ("aa", true),
        ("b", true),
        ("ba", true),
    ];
    for (input, b) in results {
        assert_eq!(reif.is_match(input), b)
    }

    let reif = reif::new!("^abc$");
    let results = [
        ("abc", true),
        ("abcd", false),
        ("ab", false),
        ("def", false),
    ];
    for (input, b) in results {
        assert_eq!(reif.is_match(input), b)
    }

    let reif = reif::new!("[abc]");
    let results = [("a", true), ("b", true), ("c", true), ("d", false)];
    for (input, b) in results {
        assert_eq!(reif.is_match(input), b)
    }

    let reif = reif::new!("\\d+");
    let results = [("123", true), ("abc", false), ("123abc", true)];
    for (input, b) in results {
        assert_eq!(reif.is_match(input), b)
    }

    let reif = reif::new!("\\w+");
    let results = [
        ("abc", true),
        ("123", true),
        ("abc_123", true),
        ("abc", true),
        (" ", false),
    ];
    for (input, b) in results {
        assert_eq!(reif.is_match(input), b)
    }

    let reif = reif::new!("\\s+");
    let results = [(" ", true), ("\t", true), ("\n", true), ("abc", false)];
    for (input, b) in results {
        assert_eq!(reif.is_match(input), b)
    }

    let reif = reif::new!("(a)(b)");
    let results = [("ab", true), ("ac", false)];
    for (input, b) in results {
        assert_eq!(reif.is_match(input), b)
    }

    let reif = reif::new!("a{2,4}");
    let results = [
        ("aa", true),
        ("aaa", true),
        ("aaaa", true),
        ("aaaaa", true),
        ("a", false),
    ];
    for (input, b) in results {
        assert_eq!(reif.is_match(input), b)
    }

    let reif = reif::new!("ひらがな|カタカナ");
    let results = [
        ("ひらがな", true),
        ("カタカナ", true),
        ("漢字", false),
        ("alphabet", false),
    ];
    for (input, b) in results {
        assert_eq!(reif.is_match(input), b)
    }
}
