use std::{cell::RefCell, collections::HashMap};

use book_core::BookCore;

thread_local! {
    static BKS: RefCell<Option<BookCore>> = RefCell::new(None);
}

#[test]
fn t() {
    BKS.with(|bks| {
        let mut bks = bks.borrow_mut();
        *bks = Some(BookCore::init(
            include_str!("./wk8.js").to_string(),
            Some(HashMap::new()),
        ));
        let bks = bks.as_mut().unwrap();
        let res = bks.eval("test();".to_string());
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
        match bks.get_form() {
            Ok(form) => println!("Form: {:?}", form),
            Err(err) => println!("Error getting form: {:?}", err),
        }
    });

    BKS.with(|bks| {
        *bks.borrow_mut() = None;
    });
}
