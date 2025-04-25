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
        let crypto = new AesCrypto({
            cipher_mode: "cbc",
            aes_type: "aes256",
            padding_type: "pkcs7",
            encoding: "base64",
            key: key,
            iv
        });
        console.log(text);
        console.log(crypto.encrypt(text));
        console.log(crypto.decrypt(crypto.encrypt(text)));
        return crypto.encrypt(text);
    }
    "#;
    let mut core = BookCore::init(js.to_string());
    let res = core.run_action("test".to_string()).unwrap();
    let res = res.as_str().unwrap_or_default();
    assert_eq!(res, "woHoG0gBYIR/Ia8D9FNGyA==");
}
