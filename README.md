# confluence-publisher-rs
A Rust libary for publishing content to Confluence.

By design this project is very oppinionated and does not offer much flexibility at this point. 

# Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
confluence-publisher = { git = "https://github.com/mttrbit/confluence-publisher-rs", branch = "main"}
```

Vip is built with Rust 1.48.

# Usage

```rust,ignore
extern crate confluence;
extern crate confluence_publisher;

// As the confluence client does not support basic auth or api tokens yet you may need to
// make sure that you stored a valid session cookie in your client's session store.
let client = reqwest::blocking::Client::builder()
    .cookie_store(true)
    .build()
    .unwrap();
let rc_client = std::rc::Rc::new(client);

let publisher = Publisher::new(Confluence::with_client(
    rc_client,
    "https://path.to.your.confluence.com/rest/api",
));

// path := absolute path to your metadata file (yml)
match read_metadata_yml(&publisher, path) {
    Ok(_) => println!("Done"),
    Err(e) => println!("Error {:?}", e)
}
```

# Metadata file
```yml
---
spaceKey: "KEY"
ancestorId: "0"
pages:
- title: "Image REST API"
  contentFilePath: "/absolute/path/to/index.html"
  children:
  - title: "Swagger Definition"
    contentFilePath: "/absolute/path/to/api_spec.html"
    children: []
    attachments: {}
    labels: []
  - title: "Changelog"
    contentFilePath: "/absolute/path/to/release_notes.html"
    children: []
    attachments: {}
    labels: []
  - title: "UML"
    contentFilePath: "/absolute/path/to/modelling.html"
    children: []
    attachments:
      sequence.png: "/absolute/path/to/assets/sequence.png"
    labels: []
  - title: "Api Description"
    contentFilePath: "/absolute/path/to/auto_docs.html"
    children: []
    attachments: {}
    labels: []
  attachments: {}
  labels: []

```
