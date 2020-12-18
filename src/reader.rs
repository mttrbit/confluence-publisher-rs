use {
    crate::publisher::*, crate::util::read_file_content, crate::yml::*,
    confluence::model::UploadAttachmentRequest,
};

pub fn read_metadata_yml(svc: &Publisher, path_to_file: &str) -> std::io::Result<()> {
    let metadata: Metadata = read_yml(path_to_file)?;
    for page in metadata.pages {
        let content = read_file_content(&page.content_file_path)?;
        let create_page =
            CreateOrUpdatePageRequest::new(&page.title, &metadata.space_key, &content);
        if let Ok((_, _, Some(data))) = svc.create_or_update_page(&create_page) {
            println!(
                "Processed id={:?} space={:?} title={:?}",
                data.id, data.space.key, data.title
            );
            for c in page.children {
                let content = read_file_content(&c.content_file_path)?;
                let req =
                    CreateChildPageRequest::new(&data.id, &c.title, &metadata.space_key, &content);

                if let (_, _, Some(data)) = svc.create_or_update_child_page(&req).unwrap() {
                    println!(
                        "Processed id={:?} space={:?} title={:?}",
                        data.id, data.space.key, data.title
                    );
                    for (k, v) in c.attachments {
                        let r =
                            UploadAttachmentRequest::new(&data.id, &k, &v.as_str().unwrap(), "");
                        let _ = svc.upload_or_update_attachment(&r);
                    }
                }
            }
        }
    }

    Ok(())
}
