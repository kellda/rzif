extern crate rzif;
mod common;
use rzif::Cause;

fn main(file: Vec<u8>) {
    let (str, trans, err) =
        common::main(file, vec![("n".to_string(), '\n')].into_iter(), " ".chars());
    assert_eq!(trans, String::new());
    common::check(&str, include_str!("../zcode/strictz/strictz.out"), true);
    common::errors(err, vec![(Cause::BadObj, (0, 0)); 17]);
}

#[test]
fn v5() {
    main(include_bytes!("../zcode/strictz/strictz.z5").to_vec());
}

#[test]
fn v7() {
    main(include_bytes!("../zcode/strictz/strictz.z7").to_vec());
}

#[test]
fn v8() {
    main(include_bytes!("../zcode/strictz/strictz.z8").to_vec());
}
