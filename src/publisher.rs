use {
    crate::Result,
    confluence::{
        client::{Confluence, Executor},
        content::model::*,
        model::*,
    },
    reqwest::{
        blocking::multipart::Form, blocking::multipart::Part, header::HeaderMap, StatusCode,
    },
    std::{
        fs::File,
        io::{prelude::*, BufReader},
        rc::Rc,
    },
};

#[derive(Clone)]
pub struct CreateOrUpdatePageRequest {
    pub title: String,
    pub space: String,
    pub body: String,
}

impl From<CreateOrUpdatePageRequest> for CreatePageRequest {
    fn from(req: CreateOrUpdatePageRequest) -> Self {
        Self {
            title: req.title,
            space: confluence::model::Space::new(&req.space),
            body: Body::new(Storage::new(&req.body, "storage")),
            type_name: "page".to_string(),
            ancestors: None,
        }
    }
}

impl CreateOrUpdatePageRequest {
    pub fn new(title: &str, space: &str, body: &str) -> Self {
        Self {
            title: title.to_string(),
            space: space.to_string(),
            body: body.to_string(),
        }
    }

    pub fn to_update_request(&self, content_id: &str, version: &u64) -> UpdatePageRequest {
        UpdatePageRequest {
            id: content_id.to_string(),
            title: self.title.clone(),
            space: confluence::model::Space::new(&self.space.clone()),
            body: Body::new(Storage::new(&self.body.clone(), "storage")),
            type_name: "page".to_string(),
            ancestors: None,
            version: Version::new(*version + 1),
        }
    }
}

impl From<CreateOrUpdatePageRequest> for FindPageRequest {
    fn from(req: CreateOrUpdatePageRequest) -> Self {
        Self {
            title: req.title.clone().replace(" ", "+"),
            space: req.space.clone(),
            expand: "version".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct CreateChildPageRequest {
    pub content_id: String,
    pub title: String,
    pub space: String,
    pub body: String,
}

impl CreateChildPageRequest {
    pub fn new(content_id: &str, title: &str, space: &str, body: &str) -> Self {
        Self {
            content_id: content_id.to_string(),
            title: title.to_string(),
            space: space.to_string(),
            body: body.to_string(),
        }
    }

    fn to_update_request(&self, content_id: &str, version: &u64) -> UpdatePageRequest {
        UpdatePageRequest {
            id: content_id.to_string(),
            title: self.title.clone(),
            space: confluence::model::Space::new(&self.space),
            body: Body::new(Storage::new(&self.body, "storage")),
            type_name: "page".to_string(),
            ancestors: None,
            version: Version::new(*version + 1),
        }
    }
}

impl From<CreateChildPageRequest> for FindChildPageRequest {
    fn from(req: CreateChildPageRequest) -> Self {
        Self {
            content_id: req.content_id,
            title: req.title.clone(),
        }
    }
}

impl From<CreateChildPageRequest> for CreatePageRequest {
    fn from(req: CreateChildPageRequest) -> Self {
        Self {
            title: req.title,
            space: confluence::model::Space::new(&req.space),
            body: Body::new(Storage::new(&req.body, "storage")),
            type_name: "page".to_string(),
            ancestors: Some(vec![Ancestor::new(&req.content_id)]),
        }
    }
}

pub struct Publisher {
    client: Rc<Confluence>,
}

impl Publisher {
    pub fn new(client: Confluence) -> Self {
        let rc = Rc::new(client);
        Self { client: rc }
    }

    pub fn find_page(
        &self,
        request: FindPageRequest,
    ) -> Result<(HeaderMap, StatusCode, Option<Results<Content>>)> {
        self.client
            .get()
            .content()
            .space_key(&request.space)
            .title(&request.title)
            .expand(&request.expand)
            .execute()
    }

    pub fn list_child_pages(
        &self,
        content_id: &str,
    ) -> Result<(HeaderMap, StatusCode, Option<ChildContentServiceResponse>)> {
        self.client
            .get()
            .content()
            .content_id(content_id)
            .child()
            .expand("page.version")
            .execute()
    }

    pub fn find_child_page(
        &self,
        request: FindChildPageRequest,
    ) -> Result<(HeaderMap, StatusCode, Option<ContentData>)> {
        match self.list_child_pages(&request.content_id) {
            Ok((headers, status, Some(d))) => {
                match d.page.results.iter().find(|e| e.title == request.title) {
                    Some(page) => Ok((headers, status, Some(page.into()))),
                    None => Ok((headers, status, None)),
                }
            }
            Ok((_, _, None)) => {
                let err = std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No page for this content_id found!",
                );
                Err(err.into())
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_page(
        &self,
        request: &CreatePageRequest,
    ) -> Result<(HeaderMap, StatusCode, Option<ContentServiceResponse>)> {
        self.client.post(request).content().execute()
    }

    pub fn update_page(
        &self,
        request: &UpdatePageRequest,
    ) -> Result<(HeaderMap, StatusCode, Option<ContentServiceResponse>)> {
        self.client
            .put(request)
            .content()
            .content_id(&request.id.clone())
            .execute()
    }

    pub fn create_or_update_page(
        &self,
        request: &CreateOrUpdatePageRequest,
    ) -> Result<(HeaderMap, StatusCode, Option<ContentServiceResponse>)> {
        if let Ok((_, _, Some(d))) = self.find_page(request.clone().into()) {
            if d.size == 0 {
                self.create_page(&request.clone().into())
            } else {
                let c = d.results.get(0).unwrap();
                let v = match &c.version {
                    Some(v) => v.number,
                    None => 1,
                };

                self.update_page(&request.to_update_request(&c.id, &v))
            }
        } else {
            let err = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No child page for this content_id found!",
            );
            Err(err.into())
        }
    }

    pub fn create_or_update_child_page(
        &self,
        request: &CreateChildPageRequest,
    ) -> Result<(HeaderMap, StatusCode, Option<ContentServiceResponse>)> {
        let fpr: FindChildPageRequest = request.clone().into();

        match self.find_child_page(fpr) {
            Ok((_, _, None)) => self.create_page(&request.clone().into()),
            Ok((_, _, Some(content))) => {
                self.update_page(&request.to_update_request(&content.id, &content.version))
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn find_attachment_by_name(
        &self,
        content_id: &str,
        filename: &str,
    ) -> Result<(HeaderMap, StatusCode, Option<AttachmentData>)> {
        let res = self
            .client
            .get()
            .content()
            .content_id(content_id)
            .child()
            .attachment()
            .filename(filename)
            .execute::<Results<Content>>();
        match res {
            Ok((headers, status, Some(data))) => {
                match data.results.iter().find(|e| e.title == filename) {
                    Some(content) => Ok((headers, status, Some(content.into()))),
                    None => Ok((headers, status, None)),
                }
            }
            Ok((_, _, None)) => {
                let err = std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No page for this content_id found!",
                );
                Err(err.into())
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn upload_attachment(
        &self,
        request: &UploadAttachmentRequest,
    ) -> Result<(HeaderMap, StatusCode, Option<serde_json::Value>)> {
        let file = File::open(&request.file).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut content = Vec::new();
        buf_reader.read_to_end(&mut content).unwrap();

        let part = Part::bytes(content)
            .file_name(request.name.clone())
            .mime_str("image/png")
            .unwrap();

        let form = Form::new().part("file", part);
        self.client
            .post(request.clone())
            .content()
            .content_id(&request.content_id)
            .child()
            .attachment(form)
            .execute()
    }

    pub fn update_attachment(
        &self,
        request: &UpdateAttachmentRequest,
    ) -> Result<(HeaderMap, StatusCode, Option<serde_json::Value>)> {
        let file = File::open(&request.file).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut content = Vec::new();
        buf_reader.read_to_end(&mut content).unwrap();

        let part = Part::bytes(content)
            .file_name(request.name.clone())
            .mime_str("image/png")
            .unwrap();

        let form = Form::new().part("file", part);
        self.client
            .post(request)
            .content()
            .content_id(&request.content_id)
            .child()
            .attachment(form)
            .attachment_id(&request.attachment_id)
            .data()
            .execute()
    }

    pub fn upload_or_update_attachment(
        &self,
        request: &UploadAttachmentRequest,
    ) -> Result<(HeaderMap, StatusCode, Option<serde_json::Value>)> {
        match self.find_attachment_by_name(&request.content_id, &request.name) {
            Ok((_, _, Some(data))) => {
                let req = request.to_update_attachment_request(data);
                self.update_attachment(&req)
            }
            Ok((_, _, None)) => self.upload_attachment(request),
            Err(e) => Err(e.into()),
        }
    }
}
