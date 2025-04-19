// use std::collections::HashMap;

// use book_core;

// const TEST_JS: &str = include_str!("./test.js");

// #[test]
// fn print_script() {
//     // 测试代码
//     println!("{:?}", TEST_JS);
// }

// #[test]
// fn add_js() {
//     let uuid = book_core::add_script(TEST_JS.to_string());
//     println!("uuid: {}", uuid);
// }

// #[test]
// fn search_books() {
//     let uuid = book_core::add_script(TEST_JS.to_string());
//     let books = book_core::search_books(uuid, "国王的求婚".to_string(), 0, 10);
//     println!("books: {:?}", books);
// }

// #[test]
// fn get_book_detail() {
//     let uuid = book_core::add_script(TEST_JS.to_string());
//     let book = book_core::get_book_detail(uuid, "3067".to_string());
//     println!("book: {:?}", book);
// }

// #[test]
// fn get_catalog() {
//     let uuid = book_core::add_script(TEST_JS.to_string());
//     let catalog = book_core::get_catalog(uuid, "3067".to_string());
//     println!("catalog: {:?}", catalog);
// }

// #[test]
// fn get_chapter() {
//     let uuid = book_core::add_script(TEST_JS.to_string());
//     let chapter = book_core::get_chapter(uuid, "3067".to_string(), "161589".to_string());
//     println!("chapter: {:?}", chapter);
// }

// #[test]
// fn test_env() {
//     let uuid = book_core::add_script(TEST_JS.to_string());
//     book_core::set_env(
//         uuid.clone(),
//         HashMap::from([
//             ("key1".to_string(), "value1".to_string()),
//             ("key2".to_string(), "value2".to_string()),
//         ]),
//     );
//     let envs = book_core::run_code(uuid, "__ENVS__;".to_string());
//     println!("envs: {:?}", envs);
// }
