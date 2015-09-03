#!/bin/bash

# This is a script to generate "awkward" files and directories as test cases,
# to check that exa can actually handle them: symlinks that point at
# themselves, directories that you aren't allowed to view, files with strange
# extended attributes, that sort of thing.

## -- configuration --

# Directory that the files should be generated in.
DIR=testcases

if [[ -e "$DIR" ]]
then
    echo "'$DIR' already exists - aborting" >&2
    exit 2
fi

# You! Yes, you, the name of the user running this script.
YOU=`whoami`

# Someone with *higher* privileges than yourself, such as root.
ROOT=root

# A UID that doesn't map to any user on the system.
INVALID_UID=666

# A GID that doesn't map to any group on the system.
INVALID_GID=616

# List commands as they are run
# set -x

# Abort on any error!
abort() { echo 'Hit an error - aborting' >&2; exit 1; }
trap 'abort' ERR

# Get confirmation from the user before running.

# echo "This script will generate files into the $DIR directory."
# echo "It requires sudo for the '$ROOT' user."
# echo "You probably want to edit this file before running it."
# read -r -p "Continue? [y/N] " response
# if [[ ! $response =~ ^([yY][eE][sS]|[yY])$ ]]
# then
#     exit 2
# fi

mkdir "$DIR"


## -- links --

mkdir "$DIR/links"
ln -s / "$DIR/links/root"
ln -s /usr "$DIR/links/usr"
ln -s nowhere "$DIR/links/broken"


## -- users and groups --

mkdir "$DIR/passwd"

# sudo is needed for these because we technically aren't a member of the
# groups (because they don't exist), and chown and chgrp are smart enough to
# disallow it!

touch "$DIR/passwd/unknown-uid"
sudo -u "$ROOT" chown $INVALID_UID "$DIR/passwd/unknown-uid"

touch "$DIR/passwd/unknown-gid"
sudo -u "$ROOT" chgrp $INVALID_GID "$DIR/passwd/unknown-gid"


## -- permissions --

mkdir "$DIR/permissions"

touch "$DIR/permissions/all-permissions"
chmod 777 "$DIR/permissions/all-permissions"

touch "$DIR/permissions/no-permissions"
chmod 000 "$DIR/permissions/no-permissions"

mkdir "$DIR/permissions/forbidden-directory"
chmod 000 "$DIR/permissions/forbidden-directory"


## -- extended attributes --

mkdir "$DIR/attributes"

touch "$DIR/attributes/none"

touch "$DIR/attributes/one"
xattr -w greeting hello "$DIR/attributes/one"

touch "$DIR/attributes/two"
xattr -w greeting hello "$DIR/attributes/two"
xattr -w another_greeting hi "$DIR/attributes/two"

touch "$DIR/attributes/forbidden"
xattr -w greeting hello "$DIR/attributes/forbidden"
chmod +a "$YOU deny readextattr" "$DIR/attributes/forbidden"

mkdir "$DIR/attributes/dirs"

mkdir "$DIR/attributes/dirs/empty-with-attribute"
xattr -w greeting hello "$DIR/attributes/dirs/empty-with-attribute"

mkdir "$DIR/attributes/dirs/full-with-attribute"
touch "$DIR/attributes/dirs/full-with-attribute/file"
xattr -w greeting hello "$DIR/attributes/dirs/full-with-attribute"

mkdir "$DIR/attributes/dirs/full-but-forbidden"
touch "$DIR/attributes/dirs/full-but-forbidden/file"
xattr -w greeting hello "$DIR/attributes/dirs/full-but-forbidden"
chmod 000 "$DIR/attributes/dirs/full-but-forbidden"
chmod +a "$YOU deny readextattr" "$DIR/attributes/dirs/full-but-forbidden"