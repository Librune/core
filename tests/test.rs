use std::collections::HashMap;

use book_core::BookCore;

#[test]
fn search_books() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(
        code.to_string(),
        Some(HashMap::from_iter(vec![("a".to_string(), "b".to_string())])),
    );
    let res = wk8.search_books("国王的求婚".to_string(), 1, 10);
    println!("{}", res.unwrap());
}

#[test]
fn get_book_detail() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(
        code.to_string(),
        Some(HashMap::from_iter(vec![("a".to_string(), "b".to_string())])),
    );
    let res = wk8.get_book_detail("3067".to_string());
    println!("{}", res.unwrap());
}

#[test]
fn get_catalog() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(
        code.to_string(),
        Some(HashMap::from_iter(vec![("a".to_string(), "b".to_string())])),
    );
    let res = wk8.get_catalog("3067".to_string());
    println!("{}", res.unwrap());
}

#[test]
fn get_chapter() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(
        code.to_string(),
        Some(HashMap::from_iter(vec![("a".to_string(), "b".to_string())])),
    );
    let res = wk8.get_chapter("3067".to_string(), "126119".to_string());
    println!("{:?}", res.unwrap());
}

#[test]
fn get_metadata() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(
        code.to_string(),
        Some(HashMap::from_iter(vec![("a".to_string(), "b".to_string())])),
    );
    let res = wk8.get_metadata();
    println!("{}", res.unwrap());
}

#[test]
fn get_form() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(
        code.to_string(),
        Some(HashMap::from_iter(vec![("a".to_string(), "b".to_string())])),
    );
    let res = wk8.get_form();
    println!("{}", res.unwrap());
}

#[test]
fn test_envs() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(
        code.to_string(),
        Some(HashMap::from_iter(vec![("a".to_string(), "b".to_string())])),
    );
    let res = wk8.eval("console.log(__ENVS__)".to_string());
    println!("{}", res.unwrap());
    // wk8.set_envs(HashMap::from_iter(vec![(
    //     "name".to_string(),
    //     json!("zsakvo"),
    // )]))
    // .unwrap();
    // let res = wk8.get_envs();
    // println!("{}", res.unwrap());
}

#[test]
fn test() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(
        code.to_string(),
        Some(HashMap::from_iter(vec![("a".to_string(), "b".to_string())])),
    );
    let res = wk8.eval("test();".to_string());
    println!("{}", res.unwrap());
}
