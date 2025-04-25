use book_core::BookCore;

#[test]
fn test_crypto_aes256_pkcs7() {
    let js = r#"
    function test(){
        let text = "Hello, World!";
        let key = "12345678901234567890123456789012";
        let iv = "0000000000000000";
        console.log(iv.toAscii())
        iv = iv.toAscii();
        let encrypt = new Encrypt({
            cipher_mode: "cbc",
            aes_type: "aes256",
            padding_type: "pkcs7",
            encoding: "base64",
            key: key,
            iv: new Array(16).fill(0x30),
        });
        return encrypt.value(text);
    }
    "#;
    let mut core = BookCore::init(js.to_string());
    let res = core.run_action("test".to_string()).unwrap();
    let res = res.as_str().unwrap_or_default();
    assert_eq!(res, "woHoG0gBYIR/Ia8D9FNGyA==");
}
