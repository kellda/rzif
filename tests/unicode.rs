mod common;

fn main(file: Vec<u8>) {
    let (str, trans, err) = common::main(file, Vec::new().into_iter(), "\x1b".chars());
    assert_eq!(trans, String::new());
    common::check(&str, include_str!("../zcode/unicode/unicode.out"), true);
    common::errors(err, Vec::new());
}

#[test]
fn v5() {
    main(include_bytes!("../zcode/unicode/unicode.z5").to_vec());
}

#[test]
fn v7() {
    main(include_bytes!("../zcode/unicode/unicode.z7").to_vec());
}

#[test]
fn v8() {
    main(include_bytes!("../zcode/unicode/unicode.z8").to_vec());
}
