mod common;

fn main(file: Vec<u8>) {
    let (str, trans, err) = common::main(
        file,
        vec![
            ("3".to_string(), '\n'),
            ("6".to_string(), '\n'),
            ("7".to_string(), '\n'),
            ("8".to_string(), '\n'),
            ("13".to_string(), '\n'),
            ("14".to_string(), '\n'),
        ]
        .into_iter(),
        ".äöüÄÖÜß»«ëïÿËÏáéíóúýÁÉÍÓÚÝàèìòùÀÈÌÒÙâêîôûÂÊÎÔÛåÅøØãñõÃÑÕæÆçÇþðÞÐ£œŒ¡¿€.  . ".chars(),
    );
    assert_eq!(trans, String::new());
    common::check(&str, include_str!("../zcode/etude/etude.out"), true);
    common::errors(err, Vec::new());
}

#[test]
fn v5() {
    main(include_bytes!("../zcode/etude/etude.z5").to_vec());
}

#[test]
fn v7() {
    main(include_bytes!("../zcode/etude/etude.z7").to_vec());
}

#[test]
fn v8() {
    main(include_bytes!("../zcode/etude/etude.z8").to_vec());
}
