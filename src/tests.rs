use crate::{replace, ReplacementMap};
use std::collections::HashMap;

#[test]
fn test_convert() {
    let html = r#"<div><a href="bonk.com">bonk</a><p>hewwo</p></div>"#;

    let repl_map: ReplacementMap = HashMap::from([
        ("a", HashMap::from([("class", "text-white")])),
        ("p", HashMap::from([("class", "text-blue-500")])),
    ]);

    let res = replace(&repl_map, html);

    let desired = r#"<div><a href="bonk.com" class="text-white">bonk</a><p class="text-blue-500">hewwo</p></div>"#;
    assert_eq!(res, desired);
}
