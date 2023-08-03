mod test_jwt {
    use chrono::Duration;
    use forum_api::common::jwt::Claims;
    use strum::IntoEnumIterator;

    #[test]
    fn test_generate_key() {
        let algs = forum_api::common::jwt::Algorithm::iter();
        for alg in algs {
            let key = forum_api::common::jwt::genrate_key(&alg, 256).unwrap();
            let claims = Claims::new("subject".to_string(), Duration::minutes(3));
            let _token = forum_api::common::jwt::generate_jws(&claims, &key);
        }
    }
}
