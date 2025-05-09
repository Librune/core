use std::cell::RefCell;

use book_core::BookCore;
use serde_json::Value;

thread_local! {
    static BKS: RefCell<Option<BookCore>> = RefCell::new(None);
}

#[test]
fn t() {
    BKS.with(|bks| {
        let mut bks = bks.borrow_mut();
        *bks = Some(BookCore::init(include_str!("./wk8.js").to_string()));
        let bks = bks.as_mut().unwrap();
        let res = bks.eval::<Value>("test();".to_string());
        match res {
            Ok(value) => println!("{}", value),
            Err(err) => println!("Error: {:?}", err),
        }
        // Clear the BookCore before the thread ends to avoid GC issues
    });
    BKS.with(|bks| {
        let mut bks = bks.borrow_mut();
        let bks = bks.as_mut().unwrap();
        match bks.get_metadata() {
            Ok(metadata) => println!("Metadata: {:?}", metadata),
            Err(err) => println!("Error getting metadata: {:?}", err),
        }
    });
    BKS.with(|bks| {
        let mut bks = bks.borrow_mut();
        let bks = bks.as_mut().unwrap();
        match bks.get_forms() {
            Ok(form) => println!("Form: {:?}", form),
            Err(err) => println!("Error getting form: {:?}", err),
        }
    });

    BKS.with(|bks| {
        *bks.borrow_mut() = None;
    });
}
