use ansi_term::Style;

pub trait Colours {
    fn temp(&self) -> Style;
    fn build(&self) -> Style;
    fn image(&self) -> Style;
    fn video(&self) -> Style;
    fn music(&self) -> Style;
    fn lossless(&self) -> Style;
    fn crypto(&self) -> Style;
    fn document(&self) -> Style;
    fn compressed(&self) -> Style;
    fn compiled(&self) -> Style;
}