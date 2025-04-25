use book_core::BookCore;

#[test]
fn test_crypto_sha224() {
    let js = r#"
    function test(){
        let text = "1234";
        console.log(text.toSha224())
    }
    "#;
    let mut core = BookCore::init(js.to_string());
    core.run_action("test".to_string()).unwrap();
}

#[test]
fn test_crypto_sha256() {
    let js = r#"
    function test(){
        let text = "1234";
        console.log(text.toSha256("a90f3731745f1c30ee77cb13fc00005a"))
    }
    "#;
    let mut core = BookCore::init(js.to_string());
    core.run_action("test".to_string()).unwrap();
}
