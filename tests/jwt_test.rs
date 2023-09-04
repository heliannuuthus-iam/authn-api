mod test_jwt {
    use chrono::Duration;
    use forum_api::common::jwt::Claims;
    use strum::IntoEnumIterator;

    #[test]
    fn test_generate_key() {
        let algs = forum_api::common::jwt::JwtAlgorithm::iter();
        for alg in algs {
<<<<<<< HEAD
            println!("start: {:?}", alg);
            let key = forum_api::common::jwt::genrate_key(alg).unwrap();
            let mut claims = Claims::new("subject".to_string(), Duration::minutes(3));
            claims.aud = vec![String::from("heiannuuthus")];
            let token = forum_api::common::jwt::generate_jws(&claims, &key).unwrap();
=======
            let key: forum_api::common::jwt::JwKPair<'_> =
                forum_api::common::jwt::genrate_key(&alg, 256).unwrap();
            let mut claims = Claims::new("subject".to_string(), Duration::minutes(3));
            claims.aud = vec![String::from("heiannuuthus")];
            let token = forum_api::common::jwt::generate_jws(&claims, &key).unwrap();

>>>>>>> fbd0014 (format)
            let validation = forum_api::common::jwt::validation(&key, claims.aud.clone()).unwrap();
            let verifed_claims =
                forum_api::common::jwt::verify_jws(&token, &key, validation).unwrap();
            assert_eq!(verifed_claims.sub, verifed_claims.sub)
        }
    }
}
