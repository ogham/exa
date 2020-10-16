#!/bin/bash
# This file contains the text fixtures — the known, constant data — that are
# used when setting up the environment that exa’s tests get run in.


# The directory that all the test files are created under.
export TEST_ROOT=/testcases


# Because the timestamps are formatted differently depending on whether
# they’re in the current year or not (see `details.rs`), we have to make
# sure that the files are created in the current year, so they get shown
# in the format we expect.
export CURRENT_YEAR=$(date "+%Y")
export FIXED_DATE="${CURRENT_YEAR}01011234.56"  # 1st January, 12:34:56


# We also need an UID and a GID that are guaranteed to not exist, to
# test what happen when they don’t.
export FIXED_BAD_UID=666
export FIXED_BAD_GID=616


# We create two users that own the test files.
#
# The first one just owns the ordinary ones, because we don’t want the
# test outputs to depend on “vagrant” or “ubuntu” existing.
#
# The second one has a long name, to test that the file owner column
# widens correctly. The benefit of Vagrant is that we don’t need to
# set this up on the *actual* system!
export FIXED_USER="cassowary"
export FIXED_LONG_USER="antidisestablishmentarienism"


# A couple of dates, for date-time testing.
export OLD_DATE='200303030000.00'
export MED_DATE='200606152314.29'   # the june gets used for fr_FR locale tests
export NEW_DATE='200912221038.53'   # and the december for ja_JP local tests
