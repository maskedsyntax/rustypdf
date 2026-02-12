mod pdf;
mod ui;

use gtk::prelude::*;
use gtk::{Notebook, Window, WindowType};

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

    ui::create_merge_tab(&notebook, &window);
    ui::create_split_tab(&notebook, &window);
    ui::create_compress_tab(&notebook, &window);
    ui::create_rotate_tab(&notebook, &window);
    ui::create_tools_tab(&notebook, &window);
    ui::create_organize_tab(&notebook, &window);
    ui::create_security_tab(&notebook, &window);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::glib::Propagation::Proceed
    });

    window.show_all();
    gtk::main();
}
