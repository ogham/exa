# Exa Color Explanation

exa provides its own built\-in set of file extension mappings that cover a large range of common file extensions, including documents, archives, media, and temporary files.

Any mappings in the environment variables will override this default set: running exa with `LS_COLORS="*.zip=32"` will turn zip files green but leave the colours of other compressed files alone.

You can also disable this built\-in set entirely by including a
`reset` entry at the beginning of `EXA_COLORS`.
So setting `EXA_COLORS="reset:*.txt=31"` will highlight only text
files; setting `EXA_COLORS="reset"` will highlight nothing.

## Examples

- Disable the "current user" highlighting: `EXA_COLORS="uu=0:gu=0"`
- Turn the date column green: `EXA_COLORS="da=32"`
- Highlight Vagrantfiles: `EXA_COLORS="Vagrantfile=1;4;33"`
- Override the existing zip colour: `EXA_COLORS="*.zip=38;5;125"`
- Markdown files a shade of green, log files a shade of grey:
`EXA_COLORS="*.md=38;5;121:*.log=38;5;248"`

## BUILT\-IN EXTENSIONS

"Immediate" files are the files you should look at when downloading and building a project for the first time: READMEs, Makefiles, Cargo.toml, and others.
They are highlighted in _yellow_ and _underlined_.

- Images (png, jpeg, gif) are purple.
- Videos (mp4, ogv, m2ts) are a slightly purpler purple.
- Music (mp3, m4a, ogg) is a deeper purple.
- Lossless music (flac, alac, wav) is deeper than *that* purple. In general, most media files are some shade of purple.
- Cryptographic files (asc, enc, p12) are a faint blue.
- Documents (pdf, doc, dvi) are a less faint blue.
- Compressed files (zip, tgz, Z) are red.
- Temporary files (tmp, swp, ~) are grey.
- Compiled files (class, o, pyc) are faint orange. A file is also counted as compiled if it uses a common extension and is
in the same directory as one of its source files: styles.css will count as compiled when next to styles.less or styles.sass, and scripts.js when next to scripts.ts or scripts.coffee.


## See also

- [exa.1.md](exa.1.md)
- [exa_colors.5.md](exa_colors.5.md)
