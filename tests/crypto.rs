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

#[test]
fn test_crypto_aes128_pkcs7() {
    let js = r#"
    function test(){
        let text = "Hello, World!";
        let key = "12345678901234567890123456789012";
        let iv = "0000000000000000";
        console.log(iv.toAscii())
        iv = iv.toAscii();
        let crypto = new AesCrypto({
            cipher_mode: "cbc",
            aes_type: "aes128",
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
    assert_eq!(res, "f/4/tfxZOUz6mxgRQ6hZ3w==");
}

#[test]
fn test_crypto_aes192_pkcs7() {
    let js = r#"
    function test(){
        let text = "2333";
        let key = "12345678901234567890123456789012";
        let iv = "0000000000000000";
        console.log(iv.toAscii())
        iv = iv.toAscii();
        const aes = new Aes({
            cipherMode:"cbc",
            aesType:'aes256',
            paddingType:'pkcs7',
            encoding:'base64',
            key:"c86289176564123bc9d76717629bf1c01864e1b153038ff020e3a1f06630db4b",
            iv
        })
        console.log(text);
        console.log(aes.encrypt(text));
        console.log(aes.decrypt(aes.encrypt(text)));
        return aes.encrypt(text);
    }
    "#;
    let mut core = BookCore::init(js.to_string());
    let res = core.run_action("test".to_string()).unwrap();
    let res = res.as_str().unwrap_or_default();
    assert_eq!(res, "VVvt6UCIXhf/6wHutWzroQ==");
}
