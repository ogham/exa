mod blocks;
pub use self::blocks::Colours as BlocksColours;

mod filekind;
pub use self::filekind::Colours as FilekindColours;

mod git;
pub use self::git::Colours as GitColours;

mod filetype;
pub use self::filetype::Colours as FiletypeColours;

mod groups;
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

mod users;
pub use self::users::Colours as UserColours;

mod octal;
// octal uses just one colour
