use super::common::*;

macro_rules! dump {
    ($name:ident, $src:expr, $w:expr, $h:expr) => {
        #[test]
        fn $name() {
            let buf = render_to_buffer($src, $w, $h);
            let actual = buffer_to_string(&buf);
            eprintln!("=== {} ===", stringify!($name));
            for (i, line) in actual.split('\n').enumerate() {
                eprintln!("{:02}: |{}|", i, line);
            }
            eprintln!();
        }
    };
}

dump!(d_cache_flow, "graph TD
    A{Cache Hit?} -->|No| B[Compute Result]
    B --> C[Update Cache]
    A -->|Yes| D[Return Cached]
    C --> D
    D --> E[Response]", 80, 25);

dump!(d_cache_yes_branch, "graph TD
    A{Cache Hit?} -->|Yes| B[Return Cached]
    A -->|No| C[Compute]", 80, 12);
