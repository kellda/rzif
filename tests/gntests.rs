mod common;

fn main(file: Vec<u8>) {
    let (str, trans, err) = common::main(file, Vec::new().into_iter(), "123!\"#$%&'()*+,-./0123456789:;<=>@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ 5 0".chars());
    assert_eq!(trans, String::new());
    common::check(&str, include_str!("../zcode/gntests/gntests.out"), true);
    common::errors(err, Vec::new());
}

#[test]
fn v5() {
    main(include_bytes!("../zcode/gntests/gntests.z5").to_vec());
}

#[test]
fn v7() {
    main(include_bytes!("../zcode/gntests/gntests.z7").to_vec());
}

#[test]
fn v8() {
    main(include_bytes!("../zcode/gntests/gntests.z8").to_vec());
}
