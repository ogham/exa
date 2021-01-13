# exa › xtests

These are the **extended tests**. They are integration tests: they run the `exa` binary with select configurations of parameters and environment variables, and assert that the program prints the correct text to standard output and error, and exits with the correct status code.

They test things like:

- broken symlinks
- extended attributes
- file names with weird stuff like newlines or escapes in
- invalid UTF-8
- missing users and groups
- nested Git repositories

They are intended to be run from the Vagrant VM that has already had its environment set up — see the `devtools/dev-create-test-filesystem.sh` script for how the files are generated.


## Anatomy of the tests

The tests are run using [Specsheet](https://specsheet.software/). The TOML files define the  tests, and the files in `output/` contain the output that exa should produce.

For example, let’s look at one of the tests in `lines-view.toml`. This test checks that running exa does the right thing when running with the `-1` argument, and a directory full of files:

```toml
[[cmd]]
name = "‘exa -1’ displays file names, one on each line"
shell = "exa -1 /testcases/file-names"
stdout = { file = "outputs/names_lines.ansitxt" }
stderr = { empty = true }
status = 0
tags = [ 'oneline' ]
```

Here’s an explanation of each line:

1. The `[[cmd]]` line marks this test as a [cmd](https://specsheet.software/checks/command/cmd) check, which can run arbitrary commands. In this case, the commad is exa with some arguments.

2. The `name` field is a human-readable description of the feature of exa that’s under test. It gets printed to the screen as tests are run.

3. The `shell` field contains the shell script to execute. It should have `exa` in there somewhere.

4. The `stdout` field describes the [content](https://specsheet.software/docs/check-file-schema#content) that exa should print to standard output. In this case, the test asserts that the output of running the program should be identical to the contents of the file.

5. The `stderr` field describes the content of standard error. In this case, it asserts that nothing is printed to stderr.

6. The `status` field asserts that exa should exit with a status code of 0.

7. The `tags` field does not change the test at all, but can be used to filter which tests are run, instead of running all of them each time.
