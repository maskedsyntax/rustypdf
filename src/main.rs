use gtk::prelude::*;
use gtk::{
    Button, FileChooserAction, FileChooserDialog, FileFilter, Orientation, ResponseType, Window,
    WindowType, Notebook, Entry, Label,
};
use lopdf::{Document, Object, dictionary, Stream};
use lopdf::content::{Content, Operation};
use lopdf::encryption::{EncryptionState, EncryptionVersion, Permissions};
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
    window.set_default_size(700, 600);

    let notebook = Notebook::new();
    window.add(&notebook);

    create_merge_tab(&notebook, &window);
    create_split_tab(&notebook, &window);
    create_compress_tab(&notebook, &window);
    create_rotate_tab(&notebook, &window);
    create_tools_tab(&notebook, &window);
    create_organize_tab(&notebook, &window);
    create_security_tab(&notebook, &window);

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
    let tab_label = Label::new(Some("Merge"));
    notebook.append_page(&box_container, Some(&tab_label));

    let label = Label::new(Some("Select PDF files to merge"));
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
    let tab_label = Label::new(Some("Split"));
    notebook.append_page(&box_container, Some(&tab_label));

    let label = Label::new(Some("Select a PDF to split"));
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
    let tab_label = Label::new(Some("Compress"));
    notebook.append_page(&box_container, Some(&tab_label));

    let label = Label::new(Some("Select a PDF to compress"));
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
    let tab_label = Label::new(Some("Rotate"));
    notebook.append_page(&box_container, Some(&tab_label));

    let label = Label::new(Some("Select a PDF to rotate"));
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
    let tab_label = Label::new(Some("Tools"));
    notebook.append_page(&box_container, Some(&tab_label));

    let label = Label::new(Some("Image to PDF Converter"));
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
    let tab_label = Label::new(Some("Organize"));
    notebook.append_page(&box_container, Some(&tab_label));

    let label = Label::new(Some("Select PDF for organization"));
    box_container.pack_start(&label, true, true, 0);

    let select_btn = Button::with_label("Select PDF");
    box_container.pack_start(&select_btn, false, false, 0);

    let sep1 = gtk::Separator::new(Orientation::Horizontal);
    box_container.pack_start(&sep1, false, false, 5);

    // Delete Section
    let del_label = Label::new(Some("Delete Pages (e.g. 1, 3)"));
    box_container.pack_start(&del_label, false, false, 0);
    let del_entry = Entry::new();
    box_container.pack_start(&del_entry, false, false, 0);
    let del_btn = Button::with_label("Delete Pages");
    del_btn.set_sensitive(false);
    box_container.pack_start(&del_btn, false, false, 0);

    let sep2 = gtk::Separator::new(Orientation::Horizontal);
    box_container.pack_start(&sep2, false, false, 5);

    // Reorder Section
    let re_label = Label::new(Some("Reorder Pages (e.g. 3, 1, 2)"));
    box_container.pack_start(&re_label, false, false, 0);
    let re_entry = Entry::new();
    box_container.pack_start(&re_entry, false, false, 0);
    let re_btn = Button::with_label("Reorder Pages");
    re_btn.set_sensitive(false);
    box_container.pack_start(&re_btn, false, false, 0);

    let sep3 = gtk::Separator::new(Orientation::Horizontal);
    box_container.pack_start(&sep3, false, false, 5);

    // Insert Section
    let ins_label = Label::new(Some("Insert PDF at Position"));
    box_container.pack_start(&ins_label, false, false, 0);
    let ins_pos_entry = Entry::new();
    ins_pos_entry.set_placeholder_text(Some("After page (e.g. 0 for start)"));
    box_container.pack_start(&ins_pos_entry, false, false, 0);
    let ins_btn = Button::with_label("Select PDF to Insert");
    ins_btn.set_sensitive(false);
    box_container.pack_start(&ins_btn, false, false, 0);

    let selected_file: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));
    let label_clone = label.clone();
    let del_btn_clone = del_btn.clone();
    let re_btn_clone = re_btn.clone();
    let ins_btn_clone = ins_btn.clone();
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
                del_btn_clone.set_sensitive(true);
                re_btn_clone.set_sensitive(true);
                ins_btn_clone.set_sensitive(true);
            }
        }
        dialog.close();
    });

    let file_clone = Rc::clone(&selected_file);
    let del_entry_clone = del_entry.clone();
    let window_weak = window.downgrade();
    del_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let file = file_clone.borrow();
        if let Some(input) = &*file {
            let text = del_entry_clone.text().to_string();
            let pages: Vec<u32> = text.split(',')
                .filter_map(|s| s.trim().parse::<u32>().ok())
                .collect();
            if !pages.is_empty() {
                if let Some(output) = save_dialog(&window, "modified.pdf") {
                    match delete_pages(input, output, pages) {
                        Ok(_) => show_message(&window, gtk::MessageType::Info, "Success", "Pages deleted!"),
                        Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
                    }
                }
            }
        }
    });

    let file_clone = Rc::clone(&selected_file);
    let re_entry_clone = re_entry.clone();
    let window_weak = window.downgrade();
    re_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let file = file_clone.borrow();
        if let Some(input) = &*file {
            let text = re_entry_clone.text().to_string();
            let order: Vec<u32> = text.split(',')
                .filter_map(|s| s.trim().parse::<u32>().ok())
                .collect();
            if !order.is_empty() {
                if let Some(output) = save_dialog(&window, "reordered.pdf") {
                    match reorder_pages(input, output, order) {
                        Ok(_) => show_message(&window, gtk::MessageType::Info, "Success", "Pages reordered!"),
                        Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
                    }
                }
            }
        }
    });

    let file_clone = Rc::clone(&selected_file);
    let ins_pos_entry_clone = ins_pos_entry.clone();
    let window_weak = window.downgrade();
    ins_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let base_file = file_clone.borrow();
        if let Some(input) = &*base_file {
            let pos = ins_pos_entry_clone.text().to_string().parse::<u32>().unwrap_or(0);
            let dialog = create_file_chooser(&window, FileChooserAction::Open, "Select PDF to Insert", false);
            if dialog.run() == ResponseType::Accept {
                if let Some(to_insert) = dialog.filename() {
                    dialog.close();
                    if let Some(output) = save_dialog(&window, "inserted.pdf") {
                        match insert_pages(input, &to_insert, output, pos) {
                            Ok(_) => show_message(&window, gtk::MessageType::Info, "Success", "PDF inserted successfully!"),
                            Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
                        }
                    }
                } else {
                    dialog.close();
                }
            } else {
                dialog.close();
            }
        }
    });
}

fn create_security_tab(notebook: &Notebook, window: &Window) {
    let box_container = gtk::Box::new(Orientation::Vertical, 10);
    box_container.set_border_width(10);
    let tab_label = Label::new(Some("Security"));
    notebook.append_page(&box_container, Some(&tab_label));

    let label = Label::new(Some("Add Password Protection"));
    box_container.pack_start(&label, true, true, 0);

    let select_btn = Button::with_label("Select PDF");
    box_container.pack_start(&select_btn, false, false, 0);

    let pass_label = Label::new(Some("Enter Password"));
    box_container.pack_start(&pass_label, false, false, 0);
    let pass_entry = Entry::new();
    pass_entry.set_visibility(false);
    box_container.pack_start(&pass_entry, false, false, 0);

    let action_btn = Button::with_label("Apply Password");
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
    let pass_entry_clone = pass_entry.clone();
    let window_weak = window.downgrade();
    action_btn.connect_clicked(move |_| {
        let window = match window_weak.upgrade() { Some(w) => w, None => return };
        let file = file_clone.borrow();
        if let Some(input) = &*file {
            let password = pass_entry_clone.text().to_string();
            if password.is_empty() {
                show_message(&window, gtk::MessageType::Warning, "Warning", "Password cannot be empty.");
                return;
            }
            if let Some(output) = save_dialog(&window, "protected.pdf") {
                match encrypt_pdf(input, output, password) {
                    Ok(_) => show_message(&window, gtk::MessageType::Info, "Success", "Password applied!"),
                    Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
                }
            }
        }
    });
}

// Helper Functions
fn create_file_chooser(window: &Window, action: FileChooserAction, title: &str, multiple: bool) -> FileChooserDialog {
    let dialog = FileChooserDialog::with_buttons(Some(title), Some(window), action,
        &[("_Cancel", ResponseType::Cancel), ("_Open", ResponseType::Accept)]);
    dialog.set_select_multiple(multiple);
    let filter = FileFilter::new();
    filter.add_pattern("*.pdf");
    filter.set_name(Some("PDF files"));
    dialog.add_filter(filter);
    dialog
}

fn save_dialog(window: &Window, default_name: &str) -> Option<PathBuf> {
    let dialog = FileChooserDialog::with_buttons(Some("Save File"), Some(window), FileChooserAction::Save,
        &[("_Cancel", ResponseType::Cancel), ("_Save", ResponseType::Accept)]);
    dialog.set_current_name(default_name);
    let res = if dialog.run() == ResponseType::Accept { dialog.filename() } else { None };
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

fn split_pdf(input: &PathBuf, output_dir: &PathBuf) -> Result<usize, Box<dyn std::error::Error>> {
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

fn compress_pdf(input: &PathBuf, output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::load(input)?;
    doc.trailer.remove(b"Info"); doc.trailer.remove(b"Metadata"); doc.trailer.remove(b"PieceInfo"); doc.trailer.remove(b"XMP");
    doc.decompress(); doc.compress(); doc.prune_objects(); doc.trailer.remove(b"Prev");
    doc.save(output)?;
    Ok(())
}

fn rotate_pdf(input: &PathBuf, output: PathBuf, degrees: i32) -> Result<(), Box<dyn std::error::Error>> {
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

fn image_to_pdf(input: &PathBuf, output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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

fn delete_pages(input: &PathBuf, output: PathBuf, to_delete: Vec<u32>) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::load(input)?;
    let pages = doc.get_pages();
    let kept_ids: Vec<lopdf::ObjectId> = pages.iter().filter(|(n, _)| !to_delete.contains(n)).map(|(_, id)| *id).collect();
    if kept_ids.is_empty() { return Err("Cannot delete all pages.".into()); }
    update_pages_tree(&mut doc, kept_ids)?;
    doc.save(output)?;
    Ok(())
}

fn reorder_pages(input: &PathBuf, output: PathBuf, order: Vec<u32>) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::load(input)?;
    let pages = doc.get_pages();
    let new_ids: Vec<lopdf::ObjectId> = order.iter().filter_map(|n| pages.get(n)).copied().collect();
    if new_ids.is_empty() { return Err("Invalid page order.".into()); }
    update_pages_tree(&mut doc, new_ids)?;
    doc.save(output)?;
    Ok(())
}

fn insert_pages(base: &PathBuf, to_insert: &PathBuf, output: PathBuf, after_page: u32) -> Result<(), Box<dyn std::error::Error>> {
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

fn encrypt_pdf(input: &PathBuf, output: PathBuf, password: String) -> Result<(), Box<dyn std::error::Error>> {
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
