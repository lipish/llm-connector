use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Attribute, DeriveInput, Expr, Lit, Meta, MetaNameValue, NestedMeta, Path,
};

/// Procedural macro for declarative provider definition
///
/// # Example
/// ```
/// #[derive(Provider)]
/// #[provider(name = "deepseek", base_url = "https://api.deepseek.com")]
/// #[supports_models("deepseek-chat", "deepseek-reasoner")]
/// pub struct DeepSeekConfig {
///     api_key: String,
///     timeout_ms: Option<u64>,
/// }
/// ```
///
/// This macro will automatically generate:
/// - Provider trait implementation
/// - Request/response conversion methods
/// - Error mapping logic
/// - Adapter implementation for GenericProvider
#[proc_macro_derive(Provider, attributes(provider, supports_models))]
pub fn derive_provider(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Extract provider attributes
    let provider_attrs = extract_provider_attributes(&input.attrs);
    let name = provider_attrs
        .get("name")
        .cloned()
        .unwrap_or_else(|| struct_name.to_string().to_lowercase());
    let base_url = provider_attrs.get("base_url").cloned();

    // Extract supported models
    let supported_models = extract_supported_models(&input.attrs);

    // Generate the adapter implementation
    let adapter_name = syn::Ident::new(&format!("{}Adapter", struct_name), struct_name.span());

    let expanded = quote! {
        /// Auto-generated adapter for #struct_name
        #[derive(Debug, Clone)]
        pub struct #adapter_name;

        impl #adapter_name {
            pub fn new() -> Self {
                Self
            }
        }

        #[async_trait::async_trait]
        impl crate::providers::traits::ProviderAdapter for #adapter_name {
            type RequestType = #struct_name;
            type ResponseType = crate::types::ChatResponse;
            type StreamResponseType = crate::types::StreamingResponse;
            type ErrorMapperType = crate::providers::errors::DefaultErrorMapper;

            fn name(&self) -> &str {
                #name
            }

            fn supported_models(&self) -> Vec<String> {
                vec![#(#supported_models.to_string()),*]
            }

            fn endpoint_url(&self, base_url: &Option<String>) -> String {
                base_url.as_deref().unwrap_or(#base_url).to_string()
            }

            fn build_request_data(&self, request: &crate::types::ChatRequest, stream: bool) -> Self::RequestType {
                // Convert ChatRequest to provider-specific request
                #struct_name {
                    // Default implementation - should be customized per provider
                    api_key: "".to_string(), // Should be set from config
                    timeout_ms: request.max_tokens,
                    // Add other fields as needed
                }
            }

            fn parse_response_data(&self, response: Self::ResponseType) -> crate::types::ChatResponse {
                response // Default implementation - should be customized
            }

            #[cfg(feature = "streaming")]
            fn parse_stream_response_data(&self, response: Self::StreamResponseType) -> crate::types::StreamingResponse {
                response // Default implementation - should be customized
            }
        }

        /// Auto-generated provider implementation for #struct_name
        impl crate::providers::base::Provider for #struct_name {
            fn name(&self) -> &str {
                #name
            }

            fn supported_models(&self) -> Vec<String> {
                vec![#(#supported_models.to_string()),*]
            }

            async fn chat(&self, request: &crate::types::ChatRequest) -> Result<crate::types::ChatResponse, crate::error::LlmConnectorError> {
                let adapter = #adapter_name::new();
                let generic_provider = crate::providers::generic::GenericProvider::new(
                    crate::config::ProviderConfig {
                        api_key: self.api_key.clone(),
                        base_url: #base_url.map(|s| s.to_string()),
                        timeout_ms: self.timeout_ms,
                        proxy: None,
                    },
                    adapter,
                )?;
                generic_provider.chat(request).await
            }

            #[cfg(feature = "streaming")]
            async fn chat_stream(
                &self,
                request: &crate::types::ChatRequest,
            ) -> Result<crate::types::ChatStream, crate::error::LlmConnectorError> {
                let adapter = #adapter_name::new();
                let generic_provider = crate::providers::generic::GenericProvider::new(
                    crate::config::ProviderConfig {
                        api_key: self.api_key.clone(),
                        base_url: #base_url.map(|s| s.to_string()),
                        timeout_ms: self.timeout_ms,
                        proxy: None,
                    },
                    adapter,
                )?;
                generic_provider.chat_stream(request).await
            }
        }
    };

    TokenStream::from(expanded)
}

/// Extract provider attributes from the struct
fn extract_provider_attributes(attrs: &[Attribute]) -> std::collections::HashMap<String, String> {
    let mut result = std::collections::HashMap::new();

    for attr in attrs {
        if attr.path.is_ident("provider") {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                for nested in meta_list.nested {
                    if let NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. })) =
                        nested
                    {
                        let key = path.get_ident().map(|i| i.to_string()).unwrap_or_default();
                        if let Lit::Str(lit_str) = lit {
                            result.insert(key, lit_str.value());
                        }
                    }
                }
            }
        }
    }

    result
}

/// Extract supported models from attributes
fn extract_supported_models(attrs: &[Attribute]) -> Vec<String> {
    let mut models = Vec::new();

    for attr in attrs {
        if attr.path.is_ident("supports_models") {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                for nested in meta_list.nested {
                    if let NestedMeta::Lit(Lit::Str(lit_str)) = nested {
                        models.push(lit_str.value());
                    }
                }
            }
        }
    }

    models
}

/// Default error mapper for providers
#[derive(Debug, Clone)]
pub struct DefaultErrorMapper;

impl crate::providers::errors::ErrorMapper for DefaultErrorMapper {
    fn map_http_error(status: u16, body: serde_json::Value) -> crate::error::LlmConnectorError {
        crate::error::LlmConnectorError::ProviderError(format!(
            "HTTP error {}: {}",
            status,
            body["error"]["message"].as_str().unwrap_or("Unknown error")
        ))
    }

    fn map_network_error(error: reqwest::Error) -> crate::error::LlmConnectorError {
        crate::error::LlmConnectorError::NetworkError(error.to_string())
    }

    fn is_retriable_error(_error: &crate::error::LlmConnectorError) -> bool {
        false
    }
}
