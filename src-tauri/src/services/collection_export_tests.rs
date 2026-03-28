use super::*;
use crate::models::{HttpMethod, KeyValuePair};
use crate::services::collection_import;
use uuid::Uuid;

fn make_test_collection() -> RequestCollection {
    RequestCollection {
        id: Uuid::new_v4(),
        name: "Test API".to_string(),
        requests: vec![SavedRequest {
            id: Uuid::new_v4(),
            name: "Create User".to_string(),
            method: HttpMethod::Post,
            url: "https://api.example.com/users".to_string(),
            query_params: vec![],
            headers: vec![KeyValuePair {
                id: Uuid::new_v4(),
                key: "Content-Type".to_string(),
                value: "application/json".to_string(),
                is_enabled: true,
                is_secret: false,
            }],
            body: RequestBody::Json(r#"{"name":"Alice"}"#.to_string()),
            auth: AuthConfig::BearerToken {
                token: "mytoken123".to_string(),
            },
            sort_order: 0,
            created_at: String::new(),
            updated_at: String::new(),
            response_extractions: vec![],
            timeout_secs: None,
        }],
        sort_order: 0,
        created_at: String::new(),
    }
}

#[test]
fn export_basic_collection_to_postman_v21() {
    let collection = make_test_collection();
    let json = export_postman_collection(&collection).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["info"]["name"], "Test API");
    assert_eq!(
        parsed["info"]["schema"],
        "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
    );
    assert_eq!(parsed["item"].as_array().unwrap().len(), 1);

    let item = &parsed["item"][0];
    assert_eq!(item["name"], "Create User");
    assert_eq!(item["request"]["method"], "POST");
    assert_eq!(item["request"]["url"], "https://api.example.com/users");
}

#[test]
fn export_preserves_auth_fields() {
    let collection = make_test_collection();
    let json = export_postman_collection(&collection).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    let auth = &parsed["item"][0]["request"]["auth"];
    assert_eq!(auth["type"], "bearer");
    assert_eq!(auth["bearer"][0]["key"], "token");
    assert_eq!(auth["bearer"][0]["value"], "mytoken123");
}

#[test]
fn export_preserves_headers_and_body() {
    let collection = make_test_collection();
    let json = export_postman_collection(&collection).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    let req = &parsed["item"][0]["request"];
    // Headers
    let headers = req["header"].as_array().unwrap();
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0]["key"], "Content-Type");
    assert_eq!(headers[0]["value"], "application/json");
    assert_eq!(headers[0]["disabled"], false);

    // Body
    assert_eq!(req["body"]["mode"], "raw");
    assert_eq!(req["body"]["raw"], r#"{"name":"Alice"}"#);
    assert_eq!(req["body"]["options"]["raw"]["language"], "json");
}

#[test]
fn roundtrip_import_export_preserves_key_data() {
    let collection = RequestCollection {
        id: Uuid::new_v4(),
        name: "Roundtrip Test".to_string(),
        requests: vec![
            SavedRequest {
                id: Uuid::new_v4(),
                name: "Get Items".to_string(),
                method: HttpMethod::Get,
                url: "https://example.com/items".to_string(),
                query_params: vec![],
                headers: vec![KeyValuePair {
                    id: Uuid::new_v4(),
                    key: "Accept".to_string(),
                    value: "application/json".to_string(),
                    is_enabled: true,
                    is_secret: false,
                }],
                body: RequestBody::None,
                auth: AuthConfig::BasicAuth {
                    username: "admin".to_string(),
                    password: "secret".to_string(),
                },
                sort_order: 0,
                created_at: String::new(),
                updated_at: String::new(),
                response_extractions: vec![],
                timeout_secs: None,
            },
            SavedRequest {
                id: Uuid::new_v4(),
                name: "Post Data".to_string(),
                method: HttpMethod::Post,
                url: "https://example.com/data".to_string(),
                query_params: vec![],
                headers: vec![],
                body: RequestBody::Json(r#"{"key":"value"}"#.to_string()),
                auth: AuthConfig::BearerToken {
                    token: "tok".to_string(),
                },
                sort_order: 1,
                created_at: String::new(),
                updated_at: String::new(),
                response_extractions: vec![],
                timeout_secs: None,
            },
        ],
        sort_order: 0,
        created_at: String::new(),
    };

    let exported = export_postman_collection(&collection).unwrap();
    let imported = collection_import::import_postman_collection(&exported).unwrap();

    assert_eq!(imported.name, "Roundtrip Test");
    assert_eq!(imported.requests.len(), 2);

    // First request
    assert_eq!(imported.requests[0].name, "Get Items");
    assert_eq!(imported.requests[0].method, HttpMethod::Get);
    assert_eq!(imported.requests[0].url, "https://example.com/items");
    assert_eq!(imported.requests[0].headers.len(), 1);
    assert_eq!(imported.requests[0].headers[0].key, "Accept");
    match &imported.requests[0].auth {
        AuthConfig::BasicAuth { username, password } => {
            assert_eq!(username, "admin");
            assert_eq!(password, "secret");
        }
        _ => panic!("Expected BasicAuth"),
    }

    // Second request
    assert_eq!(imported.requests[1].name, "Post Data");
    assert_eq!(imported.requests[1].method, HttpMethod::Post);
    match &imported.requests[1].body {
        RequestBody::Json(text) => assert_eq!(text, r#"{"key":"value"}"#),
        _ => panic!("Expected Json body"),
    }
    match &imported.requests[1].auth {
        AuthConfig::BearerToken { token } => assert_eq!(token, "tok"),
        _ => panic!("Expected BearerToken"),
    }
}
