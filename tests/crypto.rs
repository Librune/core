use book_core::BookCore;

#[test]
fn test_crypto_aes256_pkcs7() {
    let js = r#"
    function test(){
        let text = "IT+LcNazRBcK54/p1lMtcyRwpZ01VQ4tFr6GBslpnwOWvTT4T9Ob4WdByE3ijG2RLR0B6phAlo7wuQTJMpAFFyzSX0b6GhpzFuYbVx4Ul/r3a91PLFhACufvapfEMnKKwZPpaAz5JzgVWEOUbZXXujR6tliyT2TsgqVKPfq3u27DRCMhV8G7/2lTyai7/kEB1uP0+314snnS+h6BW+Hv+QaT0Cd/bDHjimYNCHpTsG96EytH0h21HUzlVEu2xtuU62CkguPn7/0uSsjsr/CRpj0J9tUeu3ptL8g2VwZXoB5C06KUKM5Ap1OH+/ug366ZoHwR/lJENfZQNiL5GD66jhyDP5BT7w1reR1kySYXyrM8EAyJXKPQhRoMgYcRuNwXzO6Hmo9RX+KVjRQU0lo3yahCuHCtMlco4x3mHLt3CtuOs/17uSBXV24Zwv6xrAIeBjNRy48iCoQdTOO3FpJ1MDKbHwyj8IhQnw9Kl84eKdmnXsYdsDxtpxVYGH6xbMOyT5YeoGeHGjtLSweMVIWh6/0hbGvCyv+GbeayYFYroFM99GSXCxEQjjXzoSAZQ+r/b/kM9Nbsh6l2Ui5MEQmfDU21oiHVQI5O74XljXuiajdQjdkrSJbEmY1AytFgEe17O3dm/XEp4sa4EovGOJ+o6oSoIpOhdYK3DIrmCreWi8YXBGh4vRBMF4hlj6WZqzySCj7zkUe7t7PDIYZq7IRUAhwE51dvXmaysC4NzQyfPPvGp8+4980huS8HgmpVoCTWT7dlw7If4DS9Ynh0SCztYXkuO9yjiucvowIvjC6wOMw8tgUVF/5eQNZPm+iVIDo9h8R6UDWaKg6hqgINAEvhjMpCDjYtIHID7vy6gcANIgYzHIQdNih0rjW4HQeJ2psTR+1dUoTnXkdeF/0IJXGp/CBYoQEc9zqBl3rvv/QV8M7KnxhyR+WKEc5BGFvFJGblud8DuCyB4Y7qqosOLi3rQOeLuKCZYG8cqoW6uVTrUZ0Za1QAkGbYcVax5MgnANIjA5+LHEH1D6zWw6i/eS5pznkCG1FYzX/XJoo26b2HrNg7dHiNlZKTwm4k9PStxQJGtyCN3iSz0A+YfOL0nj5JbQNBRAkHhx0yRGWPFneL3RU80cLaaie3axJq5PPfMzE0tpuuNQfo1XYRdB4MWqn63AHj/dod5CrDU5LCBwbpprNUzU6C33x18W9dRMA9u/eMes51xJpJwuSmEnzC96f83nhz+vHA8XbDsWR8xEIhTR19Q0F4W4TuKKp4qjYioEPw7BsmW4QB+1YkVJi702jLp/SZ8FoeiM9KaH9Wv4FwCV9HsuQZ6S7E9ooc76UKrpsmJig8ZXpCOwN4tekuE0v6CWGpacl8wEe+N38Haca8OPdFGzY2XX53HkOnqOE9X6rQro/3pYc3Sd9jew5bXG8lsg==";
        let key = "00000000000000000";
        console.log(key.toSha("256"))
        let iv = new Array(16).fill(0);
        let aes = new Aes({
            cipherMode: "cbc",
            aesType: "aes256",
            paddingType: "pkcs7",
            encoding: "base64",
            key: key.toSha("256"),
            iv
        });
        // console.log(text);
        // console.log(crypto.encrypt(text));
        console.log(aes.decrypt(text));
        return aes.decrypt(text);
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
