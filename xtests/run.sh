#!/bin/bash
set +xe


# The exa binary we want to run
exa="$HOME/target/debug/exa --colour=always"

# Directory containing our awkward testcase files
testcases=~/testcases

# Directory containing existing test results to compare against
results=/vagrant/xtests


# Long view tests
$exa $testcases/files -l   | diff -q - $results/files_l     || exit 1
$exa $testcases/files -lh  | diff -q - $results/files_lh    || exit 1
$exa $testcases/files -lhb | diff -q - $results/files_lhb   || exit 1
$exa $testcases/files -lhB | diff -q - $results/files_lhb2  || exit 1

# Grid view tests
COLUMNS=40  $exa $testcases/files | diff -q - $results/files_40   || exit 1
COLUMNS=80  $exa $testcases/files | diff -q - $results/files_80   || exit 1
COLUMNS=120 $exa $testcases/files | diff -q - $results/files_120  || exit 1
COLUMNS=160 $exa $testcases/files | diff -q - $results/files_160  || exit 1
COLUMNS=200 $exa $testcases/files | diff -q - $results/files_200  || exit 1

# Long grid view tests
COLUMNS=40  $exa $testcases/files -lG | diff -q - $results/files_lG_40   || exit 1
COLUMNS=80  $exa $testcases/files -lG | diff -q - $results/files_lG_80   || exit 1
COLUMNS=120 $exa $testcases/files -lG | diff -q - $results/files_lG_120  || exit 1
COLUMNS=160 $exa $testcases/files -lG | diff -q - $results/files_lG_160  || exit 1
COLUMNS=200 $exa $testcases/files -lG | diff -q - $results/files_lG_200  || exit 1

# Attributes
$exa $testcases/attributes -l@T | diff -q - $results/attributes  || exit 1

# UIDs and GIDs
$exa $testcases/passwd -lgh | diff -q - $results/passwd  || exit 1

# Permissions
$exa $testcases/permissions -lghR 2>&1 | diff -q - $results/permissions  || exit 1


echo "All the tests passed!"
