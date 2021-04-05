NEXT
====

Changes for the next release.

Enhancements:

* Add a man page.
* Add standard CHANGELOG.md and AUTHORS files.

1.3.1
=====

Fixes:

* Don't exit with error on `--help` and `--version`.
* Don't let sleep progress overflow the terminal width.

1.3.0
=====

Enhancements:

* Add a line with the exit code of the process if it's not 0.
* Make progress bar proportional to process running/sleep time.

Fixes:

* Fix handling of executions that output fewer lines than the previous one.
* Fix progress bar blinking.

1.2.0
=====

Enhancements:

* Add a status bar with progress indication and/or a spinner.

1.1.0
=====

Enhancements:

* Add stderr monitoring.

1.0.1
=====

* Re-release that fixes the name of the binary.

1.0.0
=====

* First working release, prints command output when a change is
  detected. Supports `--shell`.
