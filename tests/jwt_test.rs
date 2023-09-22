mod test_jwt {
    use chrono::Duration;
    use forum_api::{common::jwt::IdToken, dto::user::UserProfile};
    use strum::IntoEnumIterator;

    #[test]
    fn test_id_token_to_jwt() {
        let algs = forum_api::common::jwt::JwtAlgorithm::iter();
        for alg in algs {
            println!("start: {:?}", alg);
            let key = forum_api::common::jwt::genrate_key(alg).unwrap();
            let mut user_profile: UserProfile = UserProfile::default();
            user_profile.openid = "openid".to_string();
            user_profile.avatar = "pickture".to_string();
            let id_token = IdToken::new(
                "client_id",
                &user_profile,
                Duration::minutes(3).to_std().unwrap(),
            );

            let token = forum_api::common::jwt::generate_jws(&id_token, &key).unwrap();
            let validation =
                forum_api::common::jwt::validation(&key, vec!["client_id".to_string()]).unwrap();
            let verifed_claims =
                forum_api::common::jwt::verify_jws::<IdToken>(&token, &key, validation).unwrap();
            assert_eq!(verifed_claims.token.sub, "openid")
        }
    }
}
