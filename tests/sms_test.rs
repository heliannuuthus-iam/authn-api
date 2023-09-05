mod sms_test {
    use forum_api::dto::sms;

    #[test]
    fn test_sms_test() {
        let sms_config = sms::SmsConfig {
            id: 1,
            name: String::from("title"),
            template: String::from(r#"have a {{prefix}} {{suffix}}!"#),
            template_type: String::from("verify_code"),
        };
        let mut s = sms::SmsContext::from(sms_config);
        s.context.insert("prefix", "good");
        s.context.insert("suffix", "day");
        assert_eq!("have a good day!", s.render().unwrap())
    }
}
