use colours::Colours;
use file::File;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lines {
    pub colours: Colours,
}

/// The lines view literally just displays each file, line-by-line.
impl Lines {
    pub fn view(&self, files: &[File]) {
        for file in files {
            println!("{}", file.file_name_view(&self.colours));
        }
    }
}
