#!/bin/bash
# This script creates a bunch of awkward test case files. It gets
# automatically run as part of Vagrant provisioning.
trap 'exit' ERR

if [[ ! -d "/vagrant" ]]; then
    echo "This script should be run in the Vagrant environment"
    exit 1
fi

source "/vagrant/devtools/dev-fixtures.sh"


# Delete old testcases if they exist already, then create a
# directory to house new ones.
if [[ -d "$TEST_ROOT" ]]; then
    echo -e "\033[1m[ 0/13]\033[0m Deleting existing test cases directory"
    sudo rm -rf "$TEST_ROOT"
fi

sudo mkdir "$TEST_ROOT"
sudo chmod 777 "$TEST_ROOT"
sudo mkdir "$TEST_ROOT/empty"


# Awkward file size testcases.
# This needs sudo to set the files’ users at the very end.
mkdir "$TEST_ROOT/files"
echo -e "\033[1m[ 1/13]\033[0m Creating file size testcases"
for i in {1..13}; do
  fallocate -l "$i" "$TEST_ROOT/files/$i"_bytes
  fallocate -l "$i"KiB "$TEST_ROOT/files/$i"_KiB
  fallocate -l "$i"MiB "$TEST_ROOT/files/$i"_MiB
done

touch -t $FIXED_DATE "$TEST_ROOT/files/"*
touch -t $FIXED_DATE "$TEST_ROOT/files/"
chmod 644 "$TEST_ROOT/files/"*
sudo chown $FIXED_USER:$FIXED_USER "$TEST_ROOT/files/"*


# File name extension testcases.
# These aren’t tested in details view, but we set timestamps on them to
# test that various sort options work.
mkdir "$TEST_ROOT/file-names-exts"
echo -e "\033[1m[ 2/13]\033[0m Creating file name extension testcases"

touch "$TEST_ROOT/file-names-exts/Makefile"

touch "$TEST_ROOT/file-names-exts/IMAGE.PNG"
touch "$TEST_ROOT/file-names-exts/image.svg"

touch "$TEST_ROOT/file-names-exts/VIDEO.AVI"
touch "$TEST_ROOT/file-names-exts/video.wmv"

touch "$TEST_ROOT/file-names-exts/music.mp3"
touch "$TEST_ROOT/file-names-exts/MUSIC.OGG"

touch "$TEST_ROOT/file-names-exts/lossless.flac"
touch "$TEST_ROOT/file-names-exts/lossless.wav"

touch "$TEST_ROOT/file-names-exts/crypto.asc"
touch "$TEST_ROOT/file-names-exts/crypto.signature"

touch "$TEST_ROOT/file-names-exts/document.pdf"
touch "$TEST_ROOT/file-names-exts/DOCUMENT.XLSX"

touch "$TEST_ROOT/file-names-exts/COMPRESSED.ZIP"
touch "$TEST_ROOT/file-names-exts/compressed.tar.gz"
touch "$TEST_ROOT/file-names-exts/compressed.tgz"
touch "$TEST_ROOT/file-names-exts/compressed.tar.xz"
touch "$TEST_ROOT/file-names-exts/compressed.txz"
touch "$TEST_ROOT/file-names-exts/compressed.deb"

touch "$TEST_ROOT/file-names-exts/backup~"
touch "$TEST_ROOT/file-names-exts/#SAVEFILE#"
touch "$TEST_ROOT/file-names-exts/file.tmp"

touch "$TEST_ROOT/file-names-exts/compiled.class"
touch "$TEST_ROOT/file-names-exts/compiled.o"
touch "$TEST_ROOT/file-names-exts/compiled.js"
touch "$TEST_ROOT/file-names-exts/compiled.coffee"


# File name testcases.
# bash really doesn’t want you to create a file with escaped characters
# in its name, so we have to resort to the echo builtin and touch!
mkdir "$TEST_ROOT/file-names"
echo -e "\033[1m[ 3/13]\033[0m Creating file names testcases"

echo -ne "$TEST_ROOT/file-names/ascii: hello" | xargs -0 touch
echo -ne "$TEST_ROOT/file-names/emoji: [🆒]"  | xargs -0 touch
echo -ne "$TEST_ROOT/file-names/utf-8: pâté"  | xargs -0 touch

echo -ne "$TEST_ROOT/file-names/bell: [\a]"         | xargs -0 touch
echo -ne "$TEST_ROOT/file-names/backspace: [\b]"    | xargs -0 touch
echo -ne "$TEST_ROOT/file-names/form-feed: [\f]"    | xargs -0 touch
echo -ne "$TEST_ROOT/file-names/new-line: [\n]"     | xargs -0 touch
echo -ne "$TEST_ROOT/file-names/return: [\r]"       | xargs -0 touch
echo -ne "$TEST_ROOT/file-names/tab: [\t]"          | xargs -0 touch
echo -ne "$TEST_ROOT/file-names/vertical-tab: [\v]" | xargs -0 touch

echo -ne "$TEST_ROOT/file-names/escape: [\033]"               | xargs -0 touch
echo -ne "$TEST_ROOT/file-names/ansi: [\033[34mblue\033[0m]" | xargs -0 touch

echo -ne "$TEST_ROOT/file-names/invalid-utf8-1: [\xFF]"                | xargs -0 touch
echo -ne "$TEST_ROOT/file-names/invalid-utf8-2: [\xc3\x28]"           | xargs -0 touch
echo -ne "$TEST_ROOT/file-names/invalid-utf8-3: [\xe2\x82\x28]"      | xargs -0 touch
echo -ne "$TEST_ROOT/file-names/invalid-utf8-4: [\xf0\x28\x8c\x28]" | xargs -0 touch

echo -ne "$TEST_ROOT/file-names/new-line-dir: [\n]"                | xargs -0 mkdir
echo -ne "$TEST_ROOT/file-names/new-line-dir: [\n]/subfile"        | xargs -0 touch
echo -ne "$TEST_ROOT/file-names/new-line-dir: [\n]/another: [\n]" | xargs -0 touch
echo -ne "$TEST_ROOT/file-names/new-line-dir: [\n]/broken"         | xargs -0 touch

mkdir "$TEST_ROOT/file-names/links"
ln -s "$TEST_ROOT/file-names/new-line-dir"*/* "$TEST_ROOT/file-names/links"

echo -ne "$TEST_ROOT/file-names/new-line-dir: [\n]/broken" | xargs -0 rm


# Special file testcases.
mkdir "$TEST_ROOT/specials"
echo -e "\033[1m[ 4/13]\033[0m Creating special file kind testcases"

sudo mknod "$TEST_ROOT/specials/block-device" b  3 60
sudo mknod "$TEST_ROOT/specials/char-device"  c 14 40
sudo mknod "$TEST_ROOT/specials/named-pipe"   p

sudo touch -t $FIXED_DATE "$TEST_ROOT/specials/"*


# Awkward symlink testcases.
mkdir "$TEST_ROOT/links"
echo -e "\033[1m[ 5/13]\033[0m Creating symlink testcases"

ln -s /            "$TEST_ROOT/links/root"
ln -s /usr         "$TEST_ROOT/links/usr"
ln -s nowhere      "$TEST_ROOT/links/broken"
ln -s /proc/1/root "$TEST_ROOT/links/forbidden"

touch "$TEST_ROOT/links/some_file"
ln -s "$TEST_ROOT/links/some_file" "$TEST_ROOT/links/some_file_absolute"
(cd "$TEST_ROOT/links"; ln -s "some_file" "some_file_relative")
(cd "$TEST_ROOT/links"; ln -s "."         "current_dir")
(cd "$TEST_ROOT/links"; ln -s ".."        "parent_dir")
(cd "$TEST_ROOT/links"; ln -s "itself"    "itself")


# Awkward passwd testcases.
# sudo is needed for these because we technically aren’t a member
# of the groups (because they don’t exist), and chown and chgrp
# are smart enough to disallow it!
mkdir "$TEST_ROOT/passwd"
echo -e "\033[1m[ 6/13]\033[0m Creating user and group testcases"

touch -t $FIXED_DATE                  "$TEST_ROOT/passwd/unknown-uid"
chmod 644                             "$TEST_ROOT/passwd/unknown-uid"
sudo chown $FIXED_BAD_UID:$FIXED_USER "$TEST_ROOT/passwd/unknown-uid"

touch -t $FIXED_DATE                  "$TEST_ROOT/passwd/unknown-gid"
chmod 644                             "$TEST_ROOT/passwd/unknown-gid"
sudo chown $FIXED_USER:$FIXED_BAD_GID "$TEST_ROOT/passwd/unknown-gid"


# Awkward permission testcases.
# Differences in the way ‘chmod’ handles setting ‘setuid’ and ‘setgid’
# when you don’t already own the file mean that we need to use ‘sudo’
# to change permissions to those.
mkdir "$TEST_ROOT/permissions"
echo -e "\033[1m[ 7/13]\033[0m Creating file permission testcases"

mkdir                              "$TEST_ROOT/permissions/forbidden-directory"
chmod 000                          "$TEST_ROOT/permissions/forbidden-directory"
touch -t $FIXED_DATE               "$TEST_ROOT/permissions/forbidden-directory"
sudo chown $FIXED_USER:$FIXED_USER "$TEST_ROOT/permissions/forbidden-directory"

for perms in 000 001 002 004 010 020 040 100 200 400 644 755 777 1000 1001 2000 2010 4000 4100 7666 7777; do
    touch                              "$TEST_ROOT/permissions/$perms"
    sudo chown $FIXED_USER:$FIXED_USER "$TEST_ROOT/permissions/$perms"
    sudo chmod $perms                  "$TEST_ROOT/permissions/$perms"
    sudo touch -t $FIXED_DATE          "$TEST_ROOT/permissions/$perms"
done


# Awkward date and time testcases.
mkdir "$TEST_ROOT/dates"
echo -e "\033[1m[ 8/13]\033[0m Creating date and time testcases"

# created dates
# there’s no way to touch the created date of a file...
# so we have to do this the old-fashioned way!
# (and make sure these don't actually get listed)
touch -t $FIXED_OLD_DATE    "$TEST_ROOT/dates/peach";  sleep 1
touch -t $FIXED_MED_DATE    "$TEST_ROOT/dates/plum";   sleep 1
touch -t $FIXED_NEW_DATE    "$TEST_ROOT/dates/pear"

# modified dates
touch -t $FIXED_OLD_DATE -m "$TEST_ROOT/dates/pear"
touch -t $FIXED_MED_DATE -m "$TEST_ROOT/dates/peach"
touch -t $FIXED_NEW_DATE -m "$TEST_ROOT/dates/plum"

# accessed dates
touch -t $FIXED_OLD_DATE -a "$TEST_ROOT/dates/plum"
touch -t $FIXED_MED_DATE -a "$TEST_ROOT/dates/pear"
touch -t $FIXED_NEW_DATE -a "$TEST_ROOT/dates/peach"

sudo chown $FIXED_USER:$FIXED_USER -R "$TEST_ROOT/dates"

mkdir "$TEST_ROOT/far-dates"
touch -t $FIXED_PAST_DATE    "$TEST_ROOT/far-dates/the-distant-past"
touch -t $FIXED_FUTURE_DATE  "$TEST_ROOT/far-dates/beyond-the-future"


# Awkward extended attribute testcases.
# We need to test combinations of various numbers of files *and*
# extended attributes in directories. Turns out, the easiest way to
# do this is to generate all combinations of files with “one-xattr”
# or “two-xattrs” in their name and directories with “empty” or
# “one-file” in their name, then just give the right number of
# xattrs and children to those.
mkdir "$TEST_ROOT/attributes"
echo -e "\033[1m[ 9/13]\033[0m Creating extended attribute testcases"

mkdir "$TEST_ROOT/attributes/files"
touch "$TEST_ROOT/attributes/files/"{no-xattrs,one-xattr,two-xattrs}{,_forbidden}

mkdir "$TEST_ROOT/attributes/dirs"
mkdir "$TEST_ROOT/attributes/dirs/"{no-xattrs,one-xattr,two-xattrs}_{empty,one-file,two-files}{,_forbidden}

setfattr -n user.greeting         -v hello "$TEST_ROOT/attributes"/**/*{one-xattr,two-xattrs}*
setfattr -n user.another_greeting -v hi    "$TEST_ROOT/attributes"/**/*two-xattrs*

for dir in "$TEST_ROOT/attributes/dirs/"*one-file*; do
    touch $dir/file-in-question
done

for dir in "$TEST_ROOT/attributes/dirs/"*two-files*; do
    touch $dir/this-file
    touch $dir/that-file
done

find "$TEST_ROOT/attributes" -exec touch {} -t $FIXED_DATE \;

# I want to use the following to test,
# but it only works on macos:
#chmod +a "$FIXED_USER deny readextattr" "$TEST_ROOT/attributes"/**/*_forbidden

sudo chmod 000                        "$TEST_ROOT/attributes"/**/*_forbidden
sudo chown $FIXED_USER:$FIXED_USER -R "$TEST_ROOT/attributes"


# A sample Git repository
# This uses cd because it's easier than telling Git where to go each time
echo -e "\033[1m[10/13]\033[0m Creating Git testcases (1/3)"
mkdir "$TEST_ROOT/git"
cd    "$TEST_ROOT/git"
git init >/dev/null

mkdir edits additions moves

echo "original content" | tee edits/{staged,unstaged,both} >/dev/null
echo "this file gets moved" > moves/hither

git add edits moves
git config --global user.email "exa@exa.exa"
git config --global user.name "Exa Exa"
git commit -m "Automated test commit" >/dev/null

echo "modifications!" | tee edits/{staged,both} >/dev/null
touch additions/{staged,edited}
mv moves/{hither,thither}

git add edits moves additions
echo "more modifications!" | tee edits/unstaged edits/both additions/edited >/dev/null
touch additions/unstaged

find "$TEST_ROOT/git" -exec touch {} -t $FIXED_DATE \;
sudo chown $FIXED_USER:$FIXED_USER -R "$TEST_ROOT/git"


# A second Git repository
# for testing two at once
echo -e "\033[1m[11/13]\033[0m Creating Git testcases (2/3)"
mkdir -p "$TEST_ROOT/git2/deeply/nested/directory"
cd       "$TEST_ROOT/git2"
git init >/dev/null

touch "deeply/nested/directory/upd8d"
git add "deeply/nested/directory/upd8d"
git commit -m "Automated test commit" >/dev/null

echo "Now with contents" > "deeply/nested/directory/upd8d"
touch "deeply/nested/directory/l8st"

echo -e "target\n*.mp3" > ".gitignore"
mkdir "ignoreds"
touch "ignoreds/music.mp3"
touch "ignoreds/music.m4a"
mkdir "ignoreds/nested"
touch "ignoreds/nested/70s grove.mp3"
touch "ignoreds/nested/funky chicken.m4a"
mkdir "ignoreds/nested2"
touch "ignoreds/nested2/ievan polkka.mp3"

mkdir "target"
touch "target/another ignored file"

mkdir "deeply/nested/repository"
cd    "deeply/nested/repository"
git init >/dev/null
touch subfile
# This file, ‘subfile’, should _not_ be marked as a new file by exa, because
# it’s in the sub-repository but hasn’t been added to it. Were the sub-repo not
# present, it would be marked as a new file, as the top-level repo knows about
# the ‘deeply’ directory.

find "$TEST_ROOT/git2" -exec touch {} -t $FIXED_DATE \;
sudo chown $FIXED_USER:$FIXED_USER -R "$TEST_ROOT/git2"


# A third Git repository
# Regression test for https://github.com/ogham/exa/issues/526
echo -e "\033[1m[12/13]\033[0m Creating Git testcases (3/3)"
mkdir -p "$TEST_ROOT/git3"
cd       "$TEST_ROOT/git3"
git init >/dev/null

# Create symbolic links pointing to non-existing files
ln -s aaa/aaa/a b
ln -s /aaa

# This normally fails with:
find "$TEST_ROOT/git3" -exec touch {} -h -t $FIXED_DATE \;
sudo chown $FIXED_USER:$FIXED_USER -R "$TEST_ROOT/git3"


# Hidden and dot file testcases.
# We need to set the permissions of `.` and `..` because they actually
# get displayed in the output here, so this has to come last.
echo -e "\033[1m[13/13]\033[0m Creating hidden and dot file testcases"
shopt -u dotglob
GLOBIGNORE=".:.."

mkdir "$TEST_ROOT/hiddens"
cd    "$TEST_ROOT/hiddens"
touch "$TEST_ROOT/hiddens/visible"
touch "$TEST_ROOT/hiddens/.hidden"
touch "$TEST_ROOT/hiddens/..extra-hidden"

# ./hiddens/
touch -t $FIXED_DATE               "$TEST_ROOT/hiddens/"*
chmod 644                          "$TEST_ROOT/hiddens/"*
sudo chown $FIXED_USER:$FIXED_USER "$TEST_ROOT/hiddens/"*

# .
touch -t $FIXED_DATE               "$TEST_ROOT/hiddens"
chmod 755                          "$TEST_ROOT/hiddens"
sudo chown $FIXED_USER:$FIXED_USER "$TEST_ROOT/hiddens"

# ..
sudo touch -t $FIXED_DATE          "$TEST_ROOT"
sudo chmod 755                     "$TEST_ROOT"
sudo chown $FIXED_USER:$FIXED_USER "$TEST_ROOT"
