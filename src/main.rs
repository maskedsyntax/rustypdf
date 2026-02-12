use gtk::prelude::*;
use gtk::{
    Button, FileChooserAction, FileChooserDialog, FileFilter, Orientation, ResponseType, Window,
    WindowType, Notebook, Entry,
};
use lopdf::{Document, Object, dictionary, Stream};
use lopdf::content::{Content, Operation};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use image::GenericImageView;

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("RustyPDF - PDF Manager");
    window.set_default_size(600, 500);

    let notebook = Notebook::new();
    window.add(&notebook);

    // Merge Tab
    create_merge_tab(&notebook, &window);
    
    // Split Tab
    create_split_tab(&notebook, &window);
    
    // Compress Tab
    create_compress_tab(&notebook, &window);
    
    // Rotate Tab
    create_rotate_tab(&notebook, &window);

    // Tools Tab (Image to PDF)
    create_tools_tab(&notebook, &window);

    // Organize Tab (Delete Pages)
    create_organize_tab(&notebook, &window);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::glib::Propagation::Proceed
    });

    window.show_all();
    gtk::main();
}

fn create_merge_tab(notebook: &Notebook, window: &Window) {
    let box_container = gtk::Box::new(Orientation::Vertical, 10);
    box_container.set_border_width(10);
    let tab_label = gtk::Label::new(Some("Merge"));
    notebook.append_page(&box_container, Some(&tab_label));

    let label = gtk::Label::new(Some("Select PDF files to merge"));
    box_container.pack_start(&label, true, true, 0);

    let select_btn = Button::with_label("Select PDFs");
    box_container.pack_start(&select_btn, false, false, 0);

    let action_btn = Button::with_label("Merge and Save");
    action_btn.set_sensitive(false);
    box_container.pack_start(&action_btn, false, false, 0);

    let selected_files: Rc<RefCell<Vec<PathBuf>>> = Rc::new(RefCell::new(Vec::new()));
    let label_clone = label.clone();
    let action_btn_clone = action_btn.clone();
    let files_clone = Rc::clone(&selected_files);
    let window_weak = window.downgrade();

    select_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let dialog = create_file_chooser(&window, FileChooserAction::Open, "Select PDF Files", true);
        if dialog.run() == ResponseType::Accept {
            let mut files = files_clone.borrow_mut();
            *files = dialog.filenames();
            label_clone.set_text(&format!("{} files selected", files.len()));
            action_btn_clone.set_sensitive(!files.is_empty());
        }
        dialog.close();
    });

    let files_clone = Rc::clone(&selected_files);
    let window_weak = window.downgrade();
    action_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let files = files_clone.borrow();
        if let Some(output) = save_dialog(&window, "merged.pdf") {
            match merge_pdfs(&files, output) {
                Ok(_) => show_message(&window, gtk::MessageType::Info, "Success", "PDFs merged successfully!"),
                Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
            }
        }
    });
}

fn create_split_tab(notebook: &Notebook, window: &Window) {
    let box_container = gtk::Box::new(Orientation::Vertical, 10);
    box_container.set_border_width(10);
    let tab_label = gtk::Label::new(Some("Split"));
    notebook.append_page(&box_container, Some(&tab_label));

    let label = gtk::Label::new(Some("Select a PDF to split"));
    box_container.pack_start(&label, true, true, 0);

    let select_btn = Button::with_label("Select PDF");
    box_container.pack_start(&select_btn, false, false, 0);

    let action_btn = Button::with_label("Split All Pages");
    action_btn.set_sensitive(false);
    box_container.pack_start(&action_btn, false, false, 0);

    let selected_file: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));
    let label_clone = label.clone();
    let action_btn_clone = action_btn.clone();
    let file_clone = Rc::clone(&selected_file);
    let window_weak = window.downgrade();

    select_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let dialog = create_file_chooser(&window, FileChooserAction::Open, "Select PDF File", false);
        if dialog.run() == ResponseType::Accept {
            let mut file = file_clone.borrow_mut();
            *file = dialog.filename();
            if let Some(f) = &*file {
                label_clone.set_text(&format!("Selected: {}", f.file_name().unwrap().to_string_lossy()));
                action_btn_clone.set_sensitive(true);
            }
        }
        dialog.close();
    });

    let file_clone = Rc::clone(&selected_file);
    let window_weak = window.downgrade();
    action_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let file = file_clone.borrow();
        if let Some(input) = &*file {
            let dialog = FileChooserDialog::with_buttons(
                Some("Select Output Folder"), Some(&window), FileChooserAction::SelectFolder,
                &[("_Cancel", ResponseType::Cancel), ("_Select", ResponseType::Accept)]
            );
            if dialog.run() == ResponseType::Accept {
                if let Some(output_dir) = dialog.filename() {
                    match split_pdf(input, &output_dir) {
                        Ok(c) => show_message(&window, gtk::MessageType::Info, "Success", &format!("Split into {} pages.", c)),
                        Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
                    }
                }
            }
            dialog.close();
        }
    });
}

fn create_compress_tab(notebook: &Notebook, window: &Window) {
    let box_container = gtk::Box::new(Orientation::Vertical, 10);
    box_container.set_border_width(10);
    let tab_label = gtk::Label::new(Some("Compress"));
    notebook.append_page(&box_container, Some(&tab_label));

    let label = gtk::Label::new(Some("Select a PDF to compress"));
    box_container.pack_start(&label, true, true, 0);

    let select_btn = Button::with_label("Select PDF");
    box_container.pack_start(&select_btn, false, false, 0);

    let action_btn = Button::with_label("Compress");
    action_btn.set_sensitive(false);
    box_container.pack_start(&action_btn, false, false, 0);

    let selected_file: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));
    let label_clone = label.clone();
    let action_btn_clone = action_btn.clone();
    let file_clone = Rc::clone(&selected_file);
    let window_weak = window.downgrade();

    select_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let dialog = create_file_chooser(&window, FileChooserAction::Open, "Select PDF File", false);
        if dialog.run() == ResponseType::Accept {
            let mut file = file_clone.borrow_mut();
            *file = dialog.filename();
            if let Some(f) = &*file {
                label_clone.set_text(&format!("Selected: {}", f.file_name().unwrap().to_string_lossy()));
                action_btn_clone.set_sensitive(true);
            }
        }
        dialog.close();
    });

    let file_clone = Rc::clone(&selected_file);
    let window_weak = window.downgrade();
    action_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let file = file_clone.borrow();
        if let Some(input) = &*file {
            if let Some(output) = save_dialog(&window, "compressed.pdf") {
                match compress_pdf(input, output) {
                    Ok(_) => show_message(&window, gtk::MessageType::Info, "Success", "Compressed successfully!"),
                    Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
                }
            }
        }
    });
}

fn create_rotate_tab(notebook: &Notebook, window: &Window) {
    let box_container = gtk::Box::new(Orientation::Vertical, 10);
    box_container.set_border_width(10);
    let tab_label = gtk::Label::new(Some("Rotate"));
    notebook.append_page(&box_container, Some(&tab_label));

    let label = gtk::Label::new(Some("Select a PDF to rotate"));
    box_container.pack_start(&label, true, true, 0);

    let select_btn = Button::with_label("Select PDF");
    box_container.pack_start(&select_btn, false, false, 0);

    let action_btn = Button::with_label("Rotate 90°");
    action_btn.set_sensitive(false);
    box_container.pack_start(&action_btn, false, false, 0);

    let selected_file: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));
    let label_clone = label.clone();
    let action_btn_clone = action_btn.clone();
    let file_clone = Rc::clone(&selected_file);
    let window_weak = window.downgrade();

    select_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let dialog = create_file_chooser(&window, FileChooserAction::Open, "Select PDF File", false);
        if dialog.run() == ResponseType::Accept {
            let mut file = file_clone.borrow_mut();
            *file = dialog.filename();
            if let Some(f) = &*file {
                label_clone.set_text(&format!("Selected: {}", f.file_name().unwrap().to_string_lossy()));
                action_btn_clone.set_sensitive(true);
            }
        }
        dialog.close();
    });

    let file_clone = Rc::clone(&selected_file);
    let window_weak = window.downgrade();
    action_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let file = file_clone.borrow();
        if let Some(input) = &*file {
            if let Some(output) = save_dialog(&window, "rotated.pdf") {
                match rotate_pdf(input, output, 90) {
                    Ok(_) => show_message(&window, gtk::MessageType::Info, "Success", "Rotated successfully!"),
                    Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
                }
            }
        }
    });
}

fn create_tools_tab(notebook: &Notebook, window: &Window) {
    let box_container = gtk::Box::new(Orientation::Vertical, 10);
    box_container.set_border_width(10);
    let tab_label = gtk::Label::new(Some("Tools"));
    notebook.append_page(&box_container, Some(&tab_label));

    let label = gtk::Label::new(Some("Image to PDF Converter"));
    box_container.pack_start(&label, true, true, 0);

    let select_btn = Button::with_label("Select Image (JPG/PNG)");
    box_container.pack_start(&select_btn, false, false, 0);

    let action_btn = Button::with_label("Convert to PDF");
    action_btn.set_sensitive(false);
    box_container.pack_start(&action_btn, false, false, 0);

    let selected_file: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));
    let label_clone = label.clone();
    let action_btn_clone = action_btn.clone();
    let file_clone = Rc::clone(&selected_file);
    let window_weak = window.downgrade();

    select_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let dialog = FileChooserDialog::with_buttons(
            Some("Select Image"), Some(&window), FileChooserAction::Open,
            &[("_Cancel", ResponseType::Cancel), ("_Open", ResponseType::Accept)]
        );
        let filter = FileFilter::new();
        filter.add_pattern("*.jpg");
        filter.add_pattern("*.jpeg");
        filter.add_pattern("*.png");
        filter.set_name(Some("Images"));
        dialog.add_filter(filter);

        if dialog.run() == ResponseType::Accept {
            let mut file = file_clone.borrow_mut();
            *file = dialog.filename();
            if let Some(f) = &*file {
                label_clone.set_text(&format!("Selected: {}", f.file_name().unwrap().to_string_lossy()));
                action_btn_clone.set_sensitive(true);
            }
        }
        dialog.close();
    });

    let file_clone = Rc::clone(&selected_file);
    let window_weak = window.downgrade();
    action_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let file = file_clone.borrow();
        if let Some(input) = &*file {
            if let Some(output) = save_dialog(&window, "image.pdf") {
                match image_to_pdf(input, output) {
                    Ok(_) => show_message(&window, gtk::MessageType::Info, "Success", "Converted successfully!"),
                    Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
                }
            }
        }
    });
}

fn create_organize_tab(notebook: &Notebook, window: &Window) {
    let box_container = gtk::Box::new(Orientation::Vertical, 10);
    box_container.set_border_width(10);
    let tab_label = gtk::Label::new(Some("Organize"));
    notebook.append_page(&box_container, Some(&tab_label));

    let label = gtk::Label::new(Some("Delete Pages from PDF"));
    box_container.pack_start(&label, true, true, 0);

    let select_btn = Button::with_label("Select PDF");
    box_container.pack_start(&select_btn, false, false, 0);

    let pages_entry = Entry::new();
    pages_entry.set_placeholder_text(Some("Enter page numbers to delete (e.g., 1, 3, 5)"));
    box_container.pack_start(&pages_entry, false, false, 0);

    let action_btn = Button::with_label("Delete Pages");
    action_btn.set_sensitive(false);
    box_container.pack_start(&action_btn, false, false, 0);

    let selected_file: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));
    let label_clone = label.clone();
    let action_btn_clone = action_btn.clone();
    let file_clone = Rc::clone(&selected_file);
    let window_weak = window.downgrade();

    select_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let dialog = create_file_chooser(&window, FileChooserAction::Open, "Select PDF File", false);
        if dialog.run() == ResponseType::Accept {
            let mut file = file_clone.borrow_mut();
            *file = dialog.filename();
            if let Some(f) = &*file {
                label_clone.set_text(&format!("Selected: {}", f.file_name().unwrap().to_string_lossy()));
                action_btn_clone.set_sensitive(true);
            }
        }
        dialog.close();
    });

    let file_clone = Rc::clone(&selected_file);
    let pages_entry_clone = pages_entry.clone();
    let window_weak = window.downgrade();
    action_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let file = file_clone.borrow();
        if let Some(input) = &*file {
            let text = pages_entry_clone.text().to_string();
            let pages: Vec<u32> = text.split(',')
                .filter_map(|s| s.trim().parse::<u32>().ok())
                .collect();
            
            if pages.is_empty() {
                 show_message(&window, gtk::MessageType::Warning, "Warning", "Please enter valid page numbers.");
                 return;
            }

            if let Some(output) = save_dialog(&window, "modified.pdf") {
                match delete_pages(input, output, pages) {
                    Ok(_) => show_message(&window, gtk::MessageType::Info, "Success", "Pages deleted successfully!"),
                    Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
                }
            }
        }
    });
}

// Helper Functions

fn create_file_chooser(window: &Window, action: FileChooserAction, title: &str, multiple: bool) -> FileChooserDialog {
    let dialog = FileChooserDialog::with_buttons(
        Some(title), Some(window), action,
        &[("_Cancel", ResponseType::Cancel), ("_Open", ResponseType::Accept)]
    );
    dialog.set_select_multiple(multiple);
    let filter = FileFilter::new();
    filter.add_pattern("*.pdf");
    filter.set_name(Some("PDF files"));
    dialog.add_filter(filter);
    dialog
}

fn save_dialog(window: &Window, default_name: &str) -> Option<PathBuf> {
    let dialog = FileChooserDialog::with_buttons(
        Some("Save File"), Some(window), FileChooserAction::Save,
        &[("_Cancel", ResponseType::Cancel), ("_Save", ResponseType::Accept)]
    );
    dialog.set_current_name(default_name);
    let res = if dialog.run() == ResponseType::Accept {
        dialog.filename()
    } else {
        None
    };
    dialog.close();
    res
}

fn show_message(parent: &Window, msg_type: gtk::MessageType, title: &str, message: &str) {
    let dialog = gtk::MessageDialog::new(Some(parent), gtk::DialogFlags::MODAL, msg_type, gtk::ButtonsType::Ok, message);
    dialog.set_title(title);
    dialog.run();
    dialog.close();
}

// Logic Functions

fn merge_pdfs(files: &[PathBuf], output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut max_id = 1;
    let mut p_collect = Vec::new();
    let mut o_collect = BTreeMap::new();
    let mut catalog_id = None;

    for file in files {
        let mut doc = Document::load(file)?;
        doc.renumber_objects_with(max_id);
        for (id, object) in doc.objects.iter() {
            o_collect.insert(*id, object.clone());
        }
        for (_, page_id) in doc.get_pages() {
            p_collect.push(page_id);
        }
        if catalog_id.is_none() {
            catalog_id = Some(doc.trailer.get(b"Root")?.as_reference()?);
        }
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

fn split_pdf(input: &PathBuf, output_dir: &PathBuf) -> Result<usize, Box<dyn std::error::Error>> {
    let doc = Document::load(input)?;
    let pages = doc.get_pages();
    let count = pages.len();
    for (i, _) in pages.iter().enumerate() {
        let mut out_doc = doc.clone();
        let out_pages = out_doc.get_pages();
        let target_page_id = out_pages.get(&(i as u32 + 1)).ok_or("Page not found")?;
        let pages_id = out_doc.new_object_id();
        let pages_dict = dictionary! {
            "Type" => "Pages", "Count" => 1, "Kids" => vec![Object::Reference(*target_page_id)],
        };
        out_doc.objects.insert(pages_id, Object::Dictionary(pages_dict));
        let catalog_id = out_doc.trailer.get(b"Root")?.as_reference()?;
        if let Ok(Object::Dictionary(catalog)) = out_doc.get_object_mut(catalog_id) {
            catalog.set("Pages", Object::Reference(pages_id));
        }
        let output_path = output_dir.join(format!("page_{}.pdf", i + 1));
        out_doc.save(output_path)?;
    }
    Ok(count)
}

fn compress_pdf(input: &PathBuf, output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::load(input)?;
    doc.trailer.remove(b"Info");
    doc.trailer.remove(b"Metadata");
    doc.trailer.remove(b"PieceInfo");
    doc.trailer.remove(b"XMP");
    doc.decompress();
    doc.compress();
    doc.prune_objects();
    doc.trailer.remove(b"Prev");
    doc.save(output)?;
    Ok(())
}

fn rotate_pdf(input: &PathBuf, output: PathBuf, degrees: i32) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::load(input)?;
    let pages = doc.get_pages();
    for (_, page_id) in pages {
        if let Ok(Object::Dictionary(page)) = doc.get_object_mut(page_id) {
            let current_rotate = page.get(b"Rotate").ok().and_then(|obj| obj.as_i64().ok()).unwrap_or(0);
            page.set("Rotate", (current_rotate + degrees as i64) % 360);
        }
    }
    doc.save(output)?;
    Ok(())
}

fn image_to_pdf(input: &PathBuf, output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(input)?;
    let (width, height) = img.dimensions();
    
    // Convert to PDF 1.5
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let content_id = doc.new_object_id();
    let page_id = doc.new_object_id();
    let image_id = doc.new_object_id();
    let catalog_id = doc.new_object_id();

    let mut content = Content { operations: Vec::new() };
    content.operations.push(Operation::new("q", vec![]));
    // Scale image to page size
    content.operations.push(Operation::new("cm", vec![
        width.into(), 0.into(), 0.into(), height.into(), 0.into(), 0.into()
    ]));
    content.operations.push(Operation::new("Do", vec![Object::Name(b"Im0".to_vec())]));
    content.operations.push(Operation::new("Q", vec![]));

    let stream = Stream::new(dictionary! {}, content.encode()?);

    // Get raw RGB bytes
    let rgb = img.to_rgb8();
    
    let image_dict = dictionary! {
        "Type" => "XObject",
        "Subtype" => "Image",
        "Width" => width,
        "Height" => height,
        "ColorSpace" => "DeviceRGB",
        "BitsPerComponent" => 8,
    };
    
    // Use the raw bytes
    let image_stream = Stream::new(image_dict, rgb.into_raw());

    doc.objects.insert(image_id, Object::Stream(image_stream));
    doc.objects.insert(content_id, Object::Stream(stream));

    let page_dict = dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), width.into(), height.into()],
        "Contents" => content_id,
        "Resources" => dictionary! {
            "XObject" => dictionary! {
                "Im0" => image_id,
            },
        },
    };
    doc.objects.insert(page_id, Object::Dictionary(page_dict));

    let pages_dict = dictionary! {
        "Type" => "Pages",
        "Kids" => vec![page_id.into()],
        "Count" => 1,
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));
    
    let catalog_dict = dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    };
    doc.objects.insert(catalog_id, Object::Dictionary(catalog_dict));
    
    doc.trailer.set("Root", catalog_id);
    
    // Compress the streams (important for images)
    doc.compress();

    doc.save(output)?;
    Ok(())
}

fn delete_pages(input: &PathBuf, output: PathBuf, pages_to_delete: Vec<u32>) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::load(input)?;
    
    // `get_pages` returns a BTreeMap<u32, ObjectId>
    let pages = doc.get_pages();
    let mut pages_to_keep_ids = Vec::new();
    
    // Pages are 1-indexed in get_pages
    for (page_num, object_id) in pages.iter() {
        if !pages_to_delete.contains(page_num) {
            pages_to_keep_ids.push(*object_id);
        }
    }
    
    if pages_to_keep_ids.is_empty() {
        return Err("Cannot delete all pages.".into());
    }

    let catalog_id = doc.trailer.get(b"Root")?.as_reference()?;
    let mut pages_id = doc.new_object_id(); 
    
    if let Ok(Object::Dictionary(cat)) = doc.get_object(catalog_id) {
         if let Ok(pid) = cat.get(b"Pages") {
             pages_id = pid.as_reference()?;
         }
    }
    
    let count = pages_to_keep_ids.len() as i32;
    let kids: Vec<Object> = pages_to_keep_ids.iter().map(|&id| Object::Reference(id)).collect();
    
    let pages_dict = dictionary! {
        "Type" => "Pages",
        "Count" => count,
        "Kids" => kids,
    };
    
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));
    
    for id in pages_to_keep_ids {
        if let Ok(Object::Dictionary(page)) = doc.get_object_mut(id) {
            page.set("Parent", pages_id);
        }
    }

    doc.save(output)?;
    Ok(())
}
