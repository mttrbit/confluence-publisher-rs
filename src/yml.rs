use {
    serde::{Deserialize, Serialize},
    std::{
        fs::File,
        io::{prelude::*, BufReader},
    },
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    #[serde(rename = "spaceKey")]
    pub space_key: String,
    pub pages: Vec<Page>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Page {
    #[serde(skip_serializing)]
    pub title: String,
    #[serde(rename = "contentFilePath")]
    pub content_file_path: String,
    pub children: Vec<Page>,
    pub attachments: serde_json::map::Map<String, serde_json::Value>,
}

pub fn read_yml(path: &str) -> std::io::Result<Metadata> {
    match File::open(path) {
        Ok(file) => {
            let mut buf_reader = BufReader::new(file);
            let mut content = String::new();
            buf_reader.read_to_string(&mut content).unwrap();
            match serde_yaml::from_str(&content) {
                Ok(data) => Ok(data),
                Err(e) => {
                    let err = std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Could not parse yml file: {:?}", e),
                    );
                    Err(err.into())
                }
            }
        }
        Err(e) => Err(e.into()),
    }
}
