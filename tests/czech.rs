mod common;

fn main(file: Vec<u8>, exept: &str) {
    let (str, trans, err) = common::main(file, Vec::new().into_iter(), Vec::new().into_iter());
    assert_eq!(trans, String::new());
    common::check(&str, exept, true);
    common::errors(err, Vec::new());
}

#[test]
fn v3() {
    main(
        include_bytes!("../zcode/czech/czech.z3").to_vec(),
        include_str!("../zcode/czech/czech.out3"),
    );
}

#[test]
fn v4() {
    main(
        include_bytes!("../zcode/czech/czech.z4").to_vec(),
        include_str!("../zcode/czech/czech.out4"),
    );
}

#[test]
fn v5() {
    main(
        include_bytes!("../zcode/czech/czech.z5").to_vec(),
        include_str!("../zcode/czech/czech.out5"),
    );
}

#[test]
fn v7() {
    main(
        include_bytes!("../zcode/czech/czech.z7").to_vec(),
        include_str!("../zcode/czech/czech.out7"),
    );
}

#[test]
fn v8() {
    main(
        include_bytes!("../zcode/czech/czech.z8").to_vec(),
        include_str!("../zcode/czech/czech.out8"),
    );
}
