mod common;

fn main(file: Vec<u8>) {
    let (str, trans, err) = common::main(file, Vec::new().into_iter(), "   ".chars());
    println!("\n======================\nChecking output...");
    common::check(&str, include_str!("../zcode/scriptbit/scriptbit.out"), true);
    println!("\n======================\nChecking transcript...");
    common::check(
        &trans,
        include_str!("../zcode/scriptbit/scriptbit.trans"),
        true,
    );
    common::errors(err, Vec::new());
}

#[test]
fn v5() {
    main(include_bytes!("../zcode/scriptbit/scriptbit.z5").to_vec());
}

#[test]
fn v7() {
    main(include_bytes!("../zcode/scriptbit/scriptbit.z7").to_vec());
}

#[test]
fn v8() {
    main(include_bytes!("../zcode/scriptbit/scriptbit.z8").to_vec());
}
