# RustyPDF
## Lightweight and Efficient PDF Management

![Rust](https://img.shields.io/badge/Rust-Toolkit-orange)
![GTK](https://img.shields.io/badge/GTK-Linux-green)
![Cross-Platform](https://img.shields.io/badge/Platform-Linux%20|%20Windows-blue)

RustyPDF is a high-performance, open-source PDF management tool built with Rust and GTK. It provides a clean, native interface for all your essential PDF tasks without the bloat of traditional editors or the privacy concerns of online tools.

---

## Key Features

### PDF Management
- Merge: Seamlessly combine multiple PDF files into a single document.
- Split: Extract individual pages into separate files.
- Compress: Reduce file size by optimizing internal streams and removing redundant metadata.
- Rotate: Quickly fix orientation by rotating all pages 90 degrees.
- Delete Pages: Remove unwanted pages by specifying page numbers.
- Reorder Pages: Change the sequence of pages within a document.
- Insert Pages: Add pages from another PDF at a specific position.

### Conversion
- Images to PDF: Convert JPG and PNG images into high-quality PDF documents instantly.

### Security
- Password Protection: Add password protection and encryption to your sensitive files.

---

## How to Use

1. Install Dependencies: Ensure you have GTK 3 installed on your system.
2. Run the App: 
   ```bash
   cargo run --release
   ```
3. Select your tool: Use the tabs at the top to navigate between Merge, Split, Compress, and more.

---

## Future Vision
- OCR Support: Extract text from scanned documents using a native Rust engine.
- Word/PPT to PDF: Native conversion for common office formats.

---

## License
This project is licensed under the MIT License.
