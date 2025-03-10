// Jackson Coxson

pub mod decode;
pub mod encode;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut d = plist::Dictionary::new();
        let a = plist::Value::Array(vec!["a".into(), 1.into()]);
        d.insert("asdf".into(), "qwer".into());
        d.insert("reeeeee".to_string(), a);

        let mut d_d = plist::Dictionary::new();
        d_d.insert("fortnite".into(), plist::Value::Boolean(false));
        d.insert("good games".into(), d_d.into());

        let b = encode::encode_to_bytes(d.clone().into()).unwrap();
        let d1 = decode::from_bytes(&b).unwrap();

        println!("{d:#?}");
        println!("{d1:#?}");
        assert_eq!(plist::Value::Dictionary(d), d1);

        let s = plist::Value::String("asdf".into());
        let b = encode::encode_to_bytes(s.clone()).unwrap();
        let s1 = decode::from_bytes(&b).unwrap();

        println!("{s:#?}");
        println!("{s1:#?}");
        assert_eq!(s, s1);
    }
}
