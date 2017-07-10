//! The “info” module contains routines that aren’t about probing the
//! filesystem nor displaying output to the user, but are internal “business
//! logic” routines that are performed on a file’s already-read metadata.
//! (This counts the file name as metadata.)

pub mod filetype;
mod sources;
