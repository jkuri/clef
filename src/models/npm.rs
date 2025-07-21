use rocket::serde::{Deserialize, Serialize};
use serde_json::Value;

// NPM publish request models
#[derive(Deserialize, Debug)]
pub struct NpmPublishRequest {
    pub _id: String,
    pub name: String,
    pub description: Option<String>,
    pub private: Option<bool>,
    pub versions: std::collections::HashMap<String, NpmPackageVersion>,
    pub _attachments: std::collections::HashMap<String, NpmAttachment>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NpmPackageVersion {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub main: Option<String>,
    pub scripts: Option<std::collections::HashMap<String, String>>,
    pub dependencies: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<std::collections::HashMap<String, String>>,
    pub keywords: Option<Vec<String>>,
    pub author: Option<Value>,
    pub license: Option<String>,
    pub dist: NpmDist,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NpmDist {
    pub shasum: String,
    pub tarball: String,
}

#[derive(Deserialize, Debug)]
pub struct NpmAttachment {
    pub content_type: String,
    pub data: String, // base64 encoded tarball
    pub length: u64,
}

#[derive(Serialize, Debug)]
pub struct NpmPublishResponse {
    pub ok: bool,
    pub id: String,
    pub rev: String,
}

// Security audit models
#[derive(Deserialize, Debug)]
pub struct SecurityAdvisoriesRequest {
    // The request can contain various fields, but we'll proxy it as-is
    #[serde(flatten)]
    pub data: Value,
}

#[derive(Serialize, Debug)]
pub struct SecurityAdvisoriesResponse {
    // The response structure from npm registry
    #[serde(flatten)]
    pub data: Value,
}

#[derive(Deserialize, Debug)]
pub struct SecurityAuditsRequest {
    // The request can contain various fields, but we'll proxy it as-is
    #[serde(flatten)]
    pub data: Value,
}

#[derive(Serialize, Debug)]
pub struct SecurityAuditsResponse {
    // The response structure from npm registry
    #[serde(flatten)]
    pub data: Value,
}
