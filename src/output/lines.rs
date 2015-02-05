use file::File;

/// The lines view literally just displays each file, line-by-line.
pub fn lines_view(files: &[File]) {
    for file in files {
        println!("{}", file.file_name_view());
    }
}
