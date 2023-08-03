mod flow_test {
    use forum_api::dto::auth::Flow;

    #[test]
    fn test_gen_id() {
        let id = Flow::gen_id();
        assert_eq!(id.len(), 32);
    }
}
