use book_core::BookCore;

#[test]
fn search_books() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(code.to_string());
    let res = wk8.search_books("国王的求婚".to_string(), 1, 10).unwrap();
    println!("{:?}", res);
}

#[test]
fn get_book_detail() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(code.to_string());
    let res = wk8.get_book_detail("3067".to_string()).unwrap();
    println!("{:?}", res);
}

#[test]
fn get_catalog() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(code.to_string());
    let res = wk8.get_catalog("3067".to_string()).unwrap();
    println!("{:?}", res);
}

#[test]
fn get_chapter() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(code.to_string());
    let res = wk8
        .get_chapter("3067".to_string(), "126119".to_string())
        .unwrap();
    println!("{:?}", res);
}

#[test]
fn get_metadata() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(code.to_string());
    let res = wk8.get_metadata().unwrap();
    println!("{:?}", res);
}

#[test]
fn get_form() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(code.to_string());
    let res = wk8.get_forms().unwrap();
    println!("{:?}", res);
}

#[test]
fn action() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(code.to_string());
    let res = wk8.run_action("test".to_string()).unwrap();
    println!("{:?}", res);
}
