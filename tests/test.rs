mod common;

fn main(file: Vec<u8>) {
    let (str, trans, err) = common::main(file, Vec::new().into_iter(), Vec::new().into_iter());
    assert_eq!(trans, String::new());
    common::check(&str, include_str!("../zcode/test/test.out"), false);
    common::errors(err, Vec::new());
}

#[test]
fn v5() {
    main(include_bytes!("../zcode/test/test.z5").to_vec());
}

#[test]
fn v7() {
    main(include_bytes!("../zcode/test/test.z7").to_vec());
}

#[test]
fn v8() {
    main(include_bytes!("../zcode/test/test.z8").to_vec());
}
