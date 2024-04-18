use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct File {
    #[serde(rename = "@path")]
    pub Path: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct MagicFolder {
    #[serde(rename = "@excludeFolders")]
    pub exclude: String,
    #[serde(rename = "@filter")]
    pub filter: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@path")]
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub File: Option<Vec<File>>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Project {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@path")]
    pub Path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub File: Option<Vec<File>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub MagicFolder: Option<Vec<MagicFolder>>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Workspace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub File: Option<Vec<File>>,
    pub Project: Vec<Project>
}

impl Workspace {
    pub fn new(projects: Vec<Project>, files: Option<Vec<File>>) -> Self {
        Workspace {
            File: files,
            Project: projects,
        }
    }
}