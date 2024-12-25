# reif

正規表現の `match` を高速で行えるマクロ

<sub>
注意: 
</sub>

<sub>
本気で高速化したいと思い作成したわけではなくマクロを書きたいために作成したため、
</sub>

<sub>
必ずしも意図した通りに動く、またはどのような場合にでも高速であることは保証できません。
</sub>

## 実行例

```rust
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
```

## 実行結果
```
----------------------------------------------------------------------------------------
regex_new_elp: 191.8µs
reif_new_elp: 0ns
----------------------------------------------------------------------------------------
https://discord.com/channels/1234567890123456789/1234567890123456789/1234567890123456789: true
Regex: 45.1µs
Reif: 400ns
https://discord.com/channels/9876543210987654321/9876543210987654321/9876543210987654321: true
Regex: 1.4µs
Reif: 300ns
https://discord.com/channels/9876543210987654321/9876543210987654321/9876543ABCDE7654321: false
Regex: 2.3µs
Reif: 400ns
----------------------------------------------------------------------------------------
```