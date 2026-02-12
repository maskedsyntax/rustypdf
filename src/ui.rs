use gtk::prelude::*;
use gtk::{
    Button, FileChooserAction, FileChooserDialog, FileFilter, Orientation, ResponseType, Window,
    Notebook, Entry, Label,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use crate::pdf;

pub fn create_merge_tab(notebook: &Notebook, window: &Window) {
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
            match pdf::merge_pdfs(&files, output) {
                Ok(_) => show_message(&window, gtk::MessageType::Info, "Success", "PDFs merged successfully!"),
                Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
            }
        }
    });
}

pub fn create_split_tab(notebook: &Notebook, window: &Window) {
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
                    match pdf::split_pdf(input, &output_dir) {
                        Ok(c) => show_message(&window, gtk::MessageType::Info, "Success", &format!("Split into {} pages.", c)),
                        Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
                    }
                }
            }
            dialog.close();
        }
    });
}

pub fn create_compress_tab(notebook: &Notebook, window: &Window) {
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
                match pdf::compress_pdf(input, output) {
                    Ok(_) => show_message(&window, gtk::MessageType::Info, "Success", "Compressed successfully!"),
                    Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
                }
            }
        }
    });
}

pub fn create_rotate_tab(notebook: &Notebook, window: &Window) {
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
                match pdf::rotate_pdf(input, output, 90) {
                    Ok(_) => show_message(&window, gtk::MessageType::Info, "Success", "Rotated successfully!"),
                    Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
                }
            }
        }
    });
}

pub fn create_tools_tab(notebook: &Notebook, window: &Window) {
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
                match pdf::image_to_pdf(input, output) {
                    Ok(_) => show_message(&window, gtk::MessageType::Info, "Success", "Converted successfully!"),
                    Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
                }
            }
        }
    });
}

pub fn create_organize_tab(notebook: &Notebook, window: &Window) {
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
                    match pdf::delete_pages(input, output, pages) {
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
                    match pdf::reorder_pages(input, output, order) {
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
                        match pdf::insert_pages(input, &to_insert, output, pos) {
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

pub fn create_security_tab(notebook: &Notebook, window: &Window) {
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
                match pdf::encrypt_pdf(input, output, password) {
                    Ok(_) => show_message(&window, gtk::MessageType::Info, "Success", "Password applied!"),
                    Err(e) => show_message(&window, gtk::MessageType::Error, "Error", &format!("Error: {}", e)),
                }
            }
        }
    });
}

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
