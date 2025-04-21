use book_core::BookCore;
use serde_json::json;

#[test]
fn test_envs() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(code.to_string());
    wk8.set_envs(json!({
      "name": "zsakvo",
      "age": 18,
    }))
    .unwrap();
    println!("----当前环境变量----");
    let res = wk8.get_envs().unwrap();
    println!("{}", res);
    println!("----用户名----");
    let res = wk8.get_env("name".to_string()).unwrap();
    println!("{}", res);
    println!("----年龄----");
    let res = wk8.get_env("age".to_string()).unwrap();
    println!("{}", res);
    println!("----清除环境变量----");
    wk8.clear_envs();
    let res = wk8.get_envs().unwrap();
    println!("{}", res);
    println!("----设置性别----");
    wk8.clear_envs();
    wk8.set_env("gender".to_string(), json!("female")).unwrap();
    let res = wk8.get_envs().unwrap();
    println!("{}", res);
}
