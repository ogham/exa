use std::path::PathBuf;

use fs::File;


impl<'a> File<'a> {

    /// For this file, return a vector of alternate file paths that, if any of
    /// them exist, mean that *this* file should be coloured as “compiled”.
    ///
    /// The point of this is to highlight compiled files such as `foo.o` when
    /// their source file `foo.c` exists in the same directory. It's too
    /// dangerous to highlight *all* compiled, so the paths in this vector
    /// are checked for existence first: for example, `foo.js` is perfectly
    /// valid without `foo.coffee`.
    pub fn get_source_files(&self) -> Vec<PathBuf> {
        if let Some(ref ext) = self.ext {
            match &ext[..] {
                "class" => vec![self.path.with_extension("java")],  // Java
                "css"   => vec![self.path.with_extension("sass"),   self.path.with_extension("less")],  // SASS, Less
                "elc"   => vec![self.path.with_extension("el")],    // Emacs Lisp
                "hi"    => vec![self.path.with_extension("hs")],    // Haskell
                "js"    => vec![self.path.with_extension("coffee"), self.path.with_extension("ts")],  // CoffeeScript, TypeScript
                "o"     => vec![self.path.with_extension("c"),      self.path.with_extension("cpp")], // C, C++
                "pyc"   => vec![self.path.with_extension("py")],    // Python

                "aux" => vec![self.path.with_extension("tex")],  // TeX: auxiliary file
                "bbl" => vec![self.path.with_extension("tex")],  // BibTeX bibliography file
                "blg" => vec![self.path.with_extension("tex")],  // BibTeX log file
                "lof" => vec![self.path.with_extension("tex")],  // TeX list of figures
                "log" => vec![self.path.with_extension("tex")],  // TeX log file
                "lot" => vec![self.path.with_extension("tex")],  // TeX list of tables
                "toc" => vec![self.path.with_extension("tex")],  // TeX table of contents

                _ => vec![],  // No source files if none of the above
            }
        }
        else {
            vec![]  // No source files if there's no extension, either!
        }
    }
}
