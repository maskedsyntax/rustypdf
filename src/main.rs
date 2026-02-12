use gtk::prelude::*;
use gtk::{
    Button, FileChooserAction, FileChooserDialog, FileFilter, Orientation, ResponseType, Window,
    WindowType, Notebook,
};
use lopdf::{Document, Object, dictionary};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("RustyPDF - PDF Manager");
    window.set_default_size(500, 400);

    let notebook = Notebook::new();
    window.add(&notebook);

    // Merge Tab
    let merge_box = gtk::Box::new(Orientation::Vertical, 10);
    merge_box.set_border_width(10);
    let merge_label = gtk::Label::new(Some("Merge"));
    notebook.append_page(&merge_box, Some(&merge_label));

    let select_label = gtk::Label::new(Some("Select PDF files to merge"));
    merge_box.pack_start(&select_label, true, true, 0);

    let select_button = Button::with_label("Select PDFs");
    merge_box.pack_start(&select_button, false, false, 0);

    let merge_button = Button::with_label("Merge and Save");
    merge_button.set_sensitive(false);
    merge_box.pack_start(&merge_button, false, false, 0);

    let selected_files: Rc<RefCell<Vec<PathBuf>>> = Rc::new(RefCell::new(Vec::new()));

    let select_label_clone = select_label.clone();
    let merge_button_clone = merge_button.clone();
    let selected_files_clone = Rc::clone(&selected_files);
    let window_weak = window.downgrade();
    select_button.connect_clicked(move |_| {
        let window = match window_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let dialog = FileChooserDialog::with_buttons(
            Some("Select PDF Files"),
            Some(&window),
            FileChooserAction::Open,
            &[
                ("_Cancel", ResponseType::Cancel),
                ("_Open", ResponseType::Accept),
            ],
        );
        dialog.set_select_multiple(true);

        let filter = FileFilter::new();
        filter.add_pattern("*.pdf");
        filter.set_name(Some("PDF files"));
        dialog.add_filter(filter);

        if dialog.run() == ResponseType::Accept {
            let mut files = selected_files_clone.borrow_mut();
            *files = dialog.filenames();
            select_label_clone.set_text(&format!("{} files selected", files.len()));
            merge_button_clone.set_sensitive(!files.is_empty());
        }
        dialog.close();
    });

    let selected_files_clone = Rc::clone(&selected_files);
    let window_weak = window.downgrade();
    merge_button.connect_clicked(move |_| {
        let window = match window_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let files = selected_files_clone.borrow();
        if files.is_empty() {
            return;
        }

        let dialog = FileChooserDialog::with_buttons(
            Some("Save Merged PDF"),
            Some(&window),
            FileChooserAction::Save,
            &[
                ("_Cancel", ResponseType::Cancel),
                ("_Save", ResponseType::Accept),
            ],
        );
        dialog.set_current_name("merged.pdf");

        if dialog.run() == ResponseType::Accept {
            if let Some(output_path) = dialog.filename() {
                match merge_pdfs(&files, output_path) {
                    Ok(_) => {
                        show_message(&window, gtk::MessageType::Info, "Success", "PDFs merged successfully!");
                    }
                    Err(e) => {
                        show_message(&window, gtk::MessageType::Error, "Error", &format!("Error merging PDFs: {}", e));
                    }
                }
            }
        }
        dialog.close();
    });

    // Split Tab
    let split_box = gtk::Box::new(Orientation::Vertical, 10);
    split_box.set_border_width(10);
    let split_tab_label = gtk::Label::new(Some("Split"));
    notebook.append_page(&split_box, Some(&split_tab_label));

    let split_select_label = gtk::Label::new(Some("Select a PDF file to split"));
    split_box.pack_start(&split_select_label, true, true, 0);

    let split_select_button = Button::with_label("Select PDF");
    split_box.pack_start(&split_select_button, false, false, 0);

    let split_button = Button::with_label("Split into Pages");
    split_button.set_sensitive(false);
    split_box.pack_start(&split_button, false, false, 0);

    let split_file: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));

    let split_select_label_clone = split_select_label.clone();
    let split_button_clone = split_button.clone();
    let split_file_clone = Rc::clone(&split_file);
    let window_weak = window.downgrade();
    split_select_button.connect_clicked(move |_| {
        let window = match window_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let dialog = FileChooserDialog::with_buttons(
            Some("Select PDF File"),
            Some(&window),
            FileChooserAction::Open,
            &[
                ("_Cancel", ResponseType::Cancel),
                ("_Open", ResponseType::Accept),
            ],
        );

        let filter = FileFilter::new();
        filter.add_pattern("*.pdf");
        filter.set_name(Some("PDF files"));
        dialog.add_filter(filter);

        if dialog.run() == ResponseType::Accept {
            let mut file = split_file_clone.borrow_mut();
            *file = dialog.filename();
            if let Some(f) = &*file {
                split_select_label_clone.set_text(&format!("Selected: {}", f.file_name().unwrap().to_string_lossy()));
                split_button_clone.set_sensitive(true);
            }
        }
        dialog.close();
    });

    let split_file_clone = Rc::clone(&split_file);
    let window_weak = window.downgrade();
    split_button.connect_clicked(move |_| {
        let window = match window_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let file = split_file_clone.borrow();
        if let Some(input_path) = &*file {
            let dialog = FileChooserDialog::with_buttons(
                Some("Select Output Directory"),
                Some(&window),
                FileChooserAction::SelectFolder,
                &[
                    ("_Cancel", ResponseType::Cancel),
                    ("_Select", ResponseType::Accept),
                ],
            );

            if dialog.run() == ResponseType::Accept {
                if let Some(output_dir) = dialog.filename() {
                    match split_pdf(input_path, &output_dir) {
                        Ok(count) => {
                            show_message(&window, gtk::MessageType::Info, "Success", &format!("PDF split into {} pages.", count));
                        }
                        Err(e) => {
                            show_message(&window, gtk::MessageType::Error, "Error", &format!("Error splitting PDF: {}", e));
                        }
                    }
                }
            }
            dialog.close();
        }
    });

    // Compress Tab
    let compress_box = gtk::Box::new(Orientation::Vertical, 10);
    compress_box.set_border_width(10);
    let compress_tab_label = gtk::Label::new(Some("Compress"));
    notebook.append_page(&compress_box, Some(&compress_tab_label));

    let compress_select_label = gtk::Label::new(Some("Select a PDF file to compress"));
    compress_box.pack_start(&compress_select_label, true, true, 0);

    let compress_select_button = Button::with_label("Select PDF");
    compress_box.pack_start(&compress_select_button, false, false, 0);

    let compress_button = Button::with_label("Compress and Save");
    compress_button.set_sensitive(false);
    compress_box.pack_start(&compress_button, false, false, 0);

    let compress_file: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));

    let compress_select_label_clone = compress_select_label.clone();
    let compress_button_clone = compress_button.clone();
    let compress_file_clone = Rc::clone(&compress_file);
    let window_weak = window.downgrade();
    compress_select_button.connect_clicked(move |_| {
        let window = match window_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let dialog = FileChooserDialog::with_buttons(
            Some("Select PDF File"),
            Some(&window),
            FileChooserAction::Open,
            &[
                ("_Cancel", ResponseType::Cancel),
                ("_Open", ResponseType::Accept),
            ],
        );

        let filter = FileFilter::new();
        filter.add_pattern("*.pdf");
        filter.set_name(Some("PDF files"));
        dialog.add_filter(filter);

        if dialog.run() == ResponseType::Accept {
            let mut file = compress_file_clone.borrow_mut();
            *file = dialog.filename();
            if let Some(f) = &*file {
                compress_select_label_clone.set_text(&format!("Selected: {}", f.file_name().unwrap().to_string_lossy()));
                compress_button_clone.set_sensitive(true);
            }
        }
        dialog.close();
    });

    let compress_file_clone = Rc::clone(&compress_file);
    let window_weak = window.downgrade();
    compress_button.connect_clicked(move |_| {
        let window = match window_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let file = compress_file_clone.borrow();
        if let Some(input_path) = &*file {
            let dialog = FileChooserDialog::with_buttons(
                Some("Save Compressed PDF"),
                Some(&window),
                FileChooserAction::Save,
                &[
                    ("_Cancel", ResponseType::Cancel),
                    ("_Save", ResponseType::Accept),
                ],
            );
            dialog.set_current_name("compressed.pdf");

            if dialog.run() == ResponseType::Accept {
                if let Some(output_path) = dialog.filename() {
                    match compress_pdf(input_path, output_path) {
                        Ok(_) => {
                            show_message(&window, gtk::MessageType::Info, "Success", "PDF compressed successfully!");
                        }
                        Err(e) => {
                            show_message(&window, gtk::MessageType::Error, "Error", &format!("Error compressing PDF: {}", e));
                        }
                    }
                }
            }
            dialog.close();
        }
    });

    // Rotate Tab
    let rotate_box = gtk::Box::new(Orientation::Vertical, 10);
    rotate_box.set_border_width(10);
    let rotate_tab_label = gtk::Label::new(Some("Rotate"));
    notebook.append_page(&rotate_box, Some(&rotate_tab_label));

    let rotate_select_label = gtk::Label::new(Some("Select a PDF file to rotate"));
    rotate_box.pack_start(&rotate_select_label, true, true, 0);

    let rotate_select_button = Button::with_label("Select PDF");
    rotate_box.pack_start(&rotate_select_button, false, false, 0);

    let rotate_button = Button::with_label("Rotate 90° and Save");
    rotate_button.set_sensitive(false);
    rotate_box.pack_start(&rotate_button, false, false, 0);

    let rotate_file: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));

    let rotate_select_label_clone = rotate_select_label.clone();
    let rotate_button_clone = rotate_button.clone();
    let rotate_file_clone = Rc::clone(&rotate_file);
    let window_weak = window.downgrade();
    rotate_select_button.connect_clicked(move |_| {
        let window = match window_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let dialog = FileChooserDialog::with_buttons(
            Some("Select PDF File"),
            Some(&window),
            FileChooserAction::Open,
            &[
                ("_Cancel", ResponseType::Cancel),
                ("_Open", ResponseType::Accept),
            ],
        );

        let filter = FileFilter::new();
        filter.add_pattern("*.pdf");
        filter.set_name(Some("PDF files"));
        dialog.add_filter(filter);

        if dialog.run() == ResponseType::Accept {
            let mut file = rotate_file_clone.borrow_mut();
            *file = dialog.filename();
            if let Some(f) = &*file {
                rotate_select_label_clone.set_text(&format!("Selected: {}", f.file_name().unwrap().to_string_lossy()));
                rotate_button_clone.set_sensitive(true);
            }
        }
        dialog.close();
    });

    let rotate_file_clone = Rc::clone(&rotate_file);
    let window_weak = window.downgrade();
    rotate_button.connect_clicked(move |_| {
        let window = match window_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let file = rotate_file_clone.borrow();
        if let Some(input_path) = &*file {
            let dialog = FileChooserDialog::with_buttons(
                Some("Save Rotated PDF"),
                Some(&window),
                FileChooserAction::Save,
                &[
                    ("_Cancel", ResponseType::Cancel),
                    ("_Save", ResponseType::Accept),
                ],
            );
            dialog.set_current_name("rotated.pdf");

            if dialog.run() == ResponseType::Accept {
                if let Some(output_path) = dialog.filename() {
                    match rotate_pdf(input_path, output_path, 90) {
                        Ok(_) => {
                            show_message(&window, gtk::MessageType::Info, "Success", "PDF rotated successfully!");
                        }
                        Err(e) => {
                            show_message(&window, gtk::MessageType::Error, "Error", &format!("Error rotating PDF: {}", e));
                        }
                    }
                }
            }
            dialog.close();
        }
    });

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::glib::Propagation::Proceed
    });

    window.show_all();
    gtk::main();
}

fn show_message(parent: &Window, msg_type: gtk::MessageType, title: &str, message: &str) {
    let dialog = gtk::MessageDialog::new(
        Some(parent),
        gtk::DialogFlags::MODAL,
        msg_type,
        gtk::ButtonsType::Ok,
        message,
    );
    dialog.set_title(title);
    dialog.run();
    dialog.close();
}

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
            "Type" => "Pages",
            "Count" => 1,
            "Kids" => vec![Object::Reference(*target_page_id)],
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
    doc.compress();
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
