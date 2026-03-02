use llm_connector::types::EmbedRequest;
use llm_connector::providers::mock::MockProviderBuilder;

#[tokio::test]
async fn test_mock_embed() {
    let client = MockProviderBuilder::new().build_client();

    let req = EmbedRequest {
        model: "text-embedding-3-small".to_string(),
        input: vec!["Hello world".to_string(), "How are you?".to_string()],
        ..Default::default()
    };

    let resp = client.embed(&req).await.unwrap();

    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].embedding, vec![0.1, 0.2, 0.3]);
}
