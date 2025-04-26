use book_core::BookCore;

#[test]
fn test_hmac() {
    let js = r#"
    function test(){
        let text = "1234";
        const hmac = new Hmac({
            hash: "md5",
            key: "r14Wn1207vnZ72DN3zg9z1sBAuPAjZKD",
            encoding: "base64"
        });
        const res = hmac.update(text);
        console.log("res");
        return res;
    }
    "#;
    let mut core = BookCore::init(js.to_string());
    let res = core.run_action("test".to_string()).unwrap();
    let res = res.as_str().unwrap_or_default();
    assert_eq!(res, "khEfgERKXw9rSxVY7UyoUA==");
}
