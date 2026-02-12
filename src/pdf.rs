use lopdf::{Document, Object, dictionary, Stream};
use lopdf::content::{Content, Operation};
use lopdf::encryption::{EncryptionState, EncryptionVersion, Permissions};
use std::collections::BTreeMap;
use std::path::PathBuf;
use image::GenericImageView;

pub fn merge_pdfs(files: &[PathBuf], output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut max_id = 1;
    let mut p_collect = Vec::new();
    let mut o_collect = BTreeMap::new();
    let mut catalog_id = None;

    for file in files {
        let mut doc = Document::load(file)?;
        doc.renumber_objects_with(max_id);
        for (id, object) in doc.objects.iter() { o_collect.insert(*id, object.clone()); }
        for (_, page_id) in doc.get_pages() { p_collect.push(page_id); }
        if catalog_id.is_none() { catalog_id = Some(doc.trailer.get(b"Root")?.as_reference()?); }
        max_id = doc.max_id + 1;
    }

    if let Some(catalog_id) = catalog_id {
        let mut out_doc = Document::with_version("1.5");
        out_doc.objects = o_collect;
        let pages_id = (max_id, 0);
        max_id += 1;
        let pages_dict = dictionary! {
            "Type" => "Pages",
            "Count" => p_collect.len() as i32,
            "Kids" => p_collect.into_iter().map(Object::Reference).collect::<Vec<_>>(),
        };
        out_doc.objects.insert(pages_id, Object::Dictionary(pages_dict));
        if let Ok(Object::Dictionary(catalog)) = out_doc.get_object_mut(catalog_id) {
            catalog.set("Pages", Object::Reference(pages_id));
        }
        out_doc.trailer.set("Root", Object::Reference(catalog_id));
        out_doc.max_id = max_id;
        out_doc.save(output)?;
    }
    Ok(())
}

pub fn split_pdf(input: &PathBuf, output_dir: &PathBuf) -> Result<usize, Box<dyn std::error::Error>> {
    let doc = Document::load(input)?;
    let pages = doc.get_pages();
    for (i, _) in pages.iter().enumerate() {
        let mut out_doc = doc.clone();
        let out_pages = out_doc.get_pages();
        let target_page_id = out_pages.get(&(i as u32 + 1)).ok_or("Page not found")?;
        let pages_id = out_doc.new_object_id();
        let pages_dict = dictionary! { "Type" => "Pages", "Count" => 1, "Kids" => vec![Object::Reference(*target_page_id)] };
        out_doc.objects.insert(pages_id, Object::Dictionary(pages_dict));
        let catalog_id = out_doc.trailer.get(b"Root")?.as_reference()?;
        if let Ok(Object::Dictionary(catalog)) = out_doc.get_object_mut(catalog_id) {
            catalog.set("Pages", Object::Reference(pages_id));
        }
        out_doc.save(output_dir.join(format!("page_{}.pdf", i + 1)))?;
    }
    Ok(pages.len())
}

pub fn compress_pdf(input: &PathBuf, output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::load(input)?;
    doc.trailer.remove(b"Info"); doc.trailer.remove(b"Metadata"); doc.trailer.remove(b"PieceInfo"); doc.trailer.remove(b"XMP");
    doc.decompress(); doc.compress(); doc.prune_objects(); doc.trailer.remove(b"Prev");
    doc.save(output)?;
    Ok(())
}

pub fn rotate_pdf(input: &PathBuf, output: PathBuf, degrees: i32) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::load(input)?;
    for (_, page_id) in doc.get_pages() {
        if let Ok(Object::Dictionary(page)) = doc.get_object_mut(page_id) {
            let current = page.get(b"Rotate").ok().and_then(|obj| obj.as_i64().ok()).unwrap_or(0);
            page.set("Rotate", (current + degrees as i64) % 360);
        }
    }
    doc.save(output)?;
    Ok(())
}

pub fn image_to_pdf(input: &PathBuf, output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(input)?;
    let (width, height) = img.dimensions();
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let content_id = doc.new_object_id();
    let page_id = doc.new_object_id();
    let image_id = doc.new_object_id();
    let catalog_id = doc.new_object_id();
    let mut content = Content { operations: Vec::new() };
    content.operations.push(Operation::new("q", vec![]));
    content.operations.push(Operation::new("cm", vec![width.into(), 0.into(), 0.into(), height.into(), 0.into(), 0.into()]));
    content.operations.push(Operation::new("Do", vec![Object::Name(b"Im0".to_vec())]));
    content.operations.push(Operation::new("Q", vec![]));
    let stream = Stream::new(dictionary! {}, content.encode()?);
    let rgb = img.to_rgb8();
    let image_dict = dictionary! { "Type" => "XObject", "Subtype" => "Image", "Width" => width, "Height" => height, "ColorSpace" => "DeviceRGB", "BitsPerComponent" => 8 };
    doc.objects.insert(image_id, Object::Stream(Stream::new(image_dict, rgb.into_raw())));
    doc.objects.insert(content_id, Object::Stream(stream));
    doc.objects.insert(page_id, Object::Dictionary(dictionary! { "Type" => "Page", "Parent" => pages_id, "MediaBox" => vec![0.into(), 0.into(), width.into(), height.into()], "Contents" => content_id, "Resources" => dictionary! { "XObject" => dictionary! { "Im0" => image_id } } }));
    doc.objects.insert(pages_id, Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => vec![page_id.into()], "Count" => 1 }));
    doc.objects.insert(catalog_id, Object::Dictionary(dictionary! { "Type" => "Catalog", "Pages" => pages_id }));
    doc.trailer.set("Root", catalog_id);
    doc.compress(); doc.save(output)?;
    Ok(())
}

pub fn delete_pages(input: &PathBuf, output: PathBuf, to_delete: Vec<u32>) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::load(input)?;
    let pages = doc.get_pages();
    let kept_ids: Vec<lopdf::ObjectId> = pages.iter().filter(|(n, _)| !to_delete.contains(n)).map(|(_, id)| *id).collect();
    if kept_ids.is_empty() { return Err("Cannot delete all pages.".into()); }
    update_pages_tree(&mut doc, kept_ids)?;
    doc.save(output)?;
    Ok(())
}

pub fn reorder_pages(input: &PathBuf, output: PathBuf, order: Vec<u32>) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::load(input)?;
    let pages = doc.get_pages();
    let new_ids: Vec<lopdf::ObjectId> = order.iter().filter_map(|n| pages.get(n)).copied().collect();
    if new_ids.is_empty() { return Err("Invalid page order.".into()); }
    update_pages_tree(&mut doc, new_ids)?;
    doc.save(output)?;
    Ok(())
}

pub fn insert_pages(base: &PathBuf, to_insert: &PathBuf, output: PathBuf, after_page: u32) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc_base = Document::load(base)?;
    let doc_to_insert = Document::load(to_insert)?;
    
    let mut doc_to_insert = doc_to_insert;
    let max_id = doc_base.max_id + 1;
    doc_to_insert.renumber_objects_with(max_id);
    
    for (id, object) in doc_to_insert.objects.iter() {
        doc_base.objects.insert(*id, object.clone());
    }
    
    let base_pages = doc_base.get_pages();
    let mut base_page_ids: Vec<lopdf::ObjectId> = base_pages.iter().map(|(_, id)| *id).collect();
    
    let insert_pages = doc_to_insert.get_pages();
    let insert_page_ids: Vec<lopdf::ObjectId> = insert_pages.iter().map(|(_, id)| *id).collect();
    
    let pos = if after_page as usize > base_page_ids.len() {
        base_page_ids.len()
    } else {
        after_page as usize
    };
    
    for (i, id) in insert_page_ids.into_iter().enumerate() {
        base_page_ids.insert(pos + i, id);
    }
    
    update_pages_tree(&mut doc_base, base_page_ids)?;
    doc_base.max_id = doc_to_insert.max_id;
    doc_base.save(output)?;
    Ok(())
}

pub fn encrypt_pdf(input: &PathBuf, output: PathBuf, password: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::load(input)?;
    let version = EncryptionVersion::V2 {
        document: &doc,
        owner_password: &password,
        user_password: &password,
        key_length: 128,
        permissions: Permissions::default(),
    };
    let state: EncryptionState = version.try_into()?;
    doc.encrypt(&state)?;
    doc.save(output)?;
    Ok(())
}

fn update_pages_tree(doc: &mut Document, page_ids: Vec<lopdf::ObjectId>) -> Result<(), Box<dyn std::error::Error>> {
    let catalog_id = doc.trailer.get(b"Root")?.as_reference()?;
    let pages_id = match doc.get_object(catalog_id)? {
        Object::Dictionary(cat) => cat.get(b"Pages")?.as_reference()?,
        _ => return Err("Invalid catalog.".into()),
    };
    let count = page_ids.len() as i32;
    let kids: Vec<Object> = page_ids.iter().map(|&id| Object::Reference(id)).collect();
    doc.objects.insert(pages_id, Object::Dictionary(dictionary! { "Type" => "Pages", "Count" => count, "Kids" => kids }));
    for id in page_ids {
        if let Ok(Object::Dictionary(page)) = doc.get_object_mut(id) { page.set("Parent", pages_id); }
    }
    Ok(())
}
