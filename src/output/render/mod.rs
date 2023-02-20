mod blocks;
pub use self::blocks::Colours as BlocksColours;

mod filetype;
pub use self::filetype::Colours as FiletypeColours;

mod git;
pub use self::git::Colours as GitColours;

#[cfg(unix)]
mod groups;
#[cfg(unix)]
pub use self::groups::Colours as GroupColours;

mod inode;
// inode uses just one colour

mod links;
pub use self::links::Colours as LinksColours;

mod permissions;
pub use self::permissions::Colours as PermissionsColours;

mod size;
pub use self::size::Colours as SizeColours;

mod times;
pub use self::times::Render as TimeRender;
// times does too

#[cfg(unix)]
mod users;
#[cfg(unix)]
pub use self::users::Colours as UserColours;

mod octal;
// octal uses just one colour
