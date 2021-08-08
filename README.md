[![CI](https://github.com/lpenz/ogle/actions/workflows/ci.yml/badge.svg)](https://github.com/lpenz/ogle/actions/workflows/ci.yml)
[![coveralls](https://coveralls.io/repos/github/lpenz/ogle/badge.svg?branch=main)](https://coveralls.io/github/lpenz/ogle?branch=main)
[![crates.io](https://img.shields.io/crates/v/ogle)](https://crates.io/crates/ogle)
[![packagecloud](https://img.shields.io/badge/deb-packagecloud.io-844fec.svg)](https://packagecloud.io/app/lpenz/debian/search?q=ogle)

# ogle

**ogle** is a program that runs the given command-line periodically,
showing the output only when it is different than the last.

The simplest way to show most of the features of *ogle* is by asking
it to run `date; sleep` in a shell, with a waiting period of 3s:

![demo](demos/demo-sleep.gif)

Lines that were written by ogle all start with `=>`. On the first
execution, ogle shows a spinner while the command is running. On the
next executions, ogle shows a progress bar, where the total
corresponds to the duration of the previous execution. The sleep time
is also shown, as a countdown. If the command returns an error to the
shell, the error value is displayed.


## Installation

If you're a **Rust programmer**, ogle can be installed with `cargo`:

```
$ cargo install ogle
```

If you're a **Debian** user, ogle is available in
[packagecloud](https://packagecloud.io/app/lpenz/debian/search?q=ogle). Follow
these
[instruction](https://packagecloud.io/lpenz/debian/install#manual) to
use the package repository.


[watch (1)]: https://linux.die.net/man/1/watch

