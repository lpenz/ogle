[![CI](https://github.com/lpenz/ogle/actions/workflows/ci.yml/badge.svg)](https://github.com/lpenz/ogle/actions/workflows/ci.yml)
[![coveralls](https://coveralls.io/repos/github/lpenz/ogle/badge.svg?branch=main)](https://coveralls.io/github/lpenz/ogle?branch=main)
[![dependency status](https://deps.rs/repo/github/lpenz/ogle/status.svg)](https://deps.rs/repo/github/lpenz/ogle)
[![crates.io](https://img.shields.io/crates/v/ogle)](https://crates.io/crates/ogle)
[![packagecloud](https://img.shields.io/badge/deb-packagecloud.io-844fec.svg)](https://packagecloud.io/app/lpenz/debian/search?q=ogle)

# `<O>` ogle

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

ogle also supports limited interactive control with one-character
commands followed by ENTER:
- `q`: quit after when the process is no longer running.

## Installation

If you're a **Rust programmer**, ogle can be installed with `cargo`:

```bash
$ cargo install ogle
```

If you're a **Debian** user, ogle is available in
[packagecloud](https://packagecloud.io/app/lpenz/debian/search?q=ogle). Follow
these
[instruction](https://packagecloud.io/lpenz/debian/install#manual) to
use the package repository.


## Internals

To make it fully testable, it uses a layered architecture based on
tokio streams which ends up being similar to how we use pipes in a
shell. We can divide it in the following layers:
- wrappers: we have 3 wrapper modules that abtract external
  libraries to provide us simpler types or types that provide that
  `impl` traits we need. They also make it easier to replate the
  underlying implementation in the future, if necessary. Namely:
  - [`process_wrapper`]: wraps process instantiation and I/O, and
    provides an [`Item`](process_wrapper::Item) that implements
    `Eq` so that we can use it in tests.
  - [`term_wrapper`]: implements terminal functions, mostly for
    output. As we are currently wrapping [`console`] and its
    functions require a [`console::Term`] object, we end up using
    a mutex here to abstract the singleton.
  - [`user_wrapper`]: abstract user interaction. At the moment, we
    just monitor `stdin` in line mode, and ogle exits gracefully
    when that's detected.
  - [`time_wrapper`]: home of the
    [`Instant`](time_wrapper::Instant) and
    [`Duration`](time_wrapper::Duration) types, which use types
    from [`chrono`] at the moment.
- [`sys`]: most of the ogle code doesn't really call functions
  that interact with the host system - we have the `sys` module
  for that. The module does that by providing a [`sys::SysApi`]
  trait that is then implemented by both the [`sys::SysReal`]
  type, which calls the system functions; and by the
  [`sys::SysVirtual`] type, which can be used to mock these calls
  in various ways.

```no_compile
sys -> engine -> view -> output
```

[watch (1)]: https://linux.die.net/man/1/watch
[`process_wrapper`]: https://docs.rs/ogle/latest/ogle/process_wrapper/index.html
[`Item`]: https://docs.rs/ogle/latest/ogle/process_wrapper/enum.Item.html
[`term_wrapper`]: https://docs.rs/ogle/latest/ogle/term_wrapper/index.html
[`console`]: https://docs.rs/console/latest/console/index.html
[`console::Term`]: https://docs.rs/console/latest/console/term/struct.Term.html
[`user_wrapper`]: https://docs.rs/ogle/latest/ogle/user_wrapper/index.html
[`time_wrapper`]: https://docs.rs/ogle/latest/ogle/time_wrapper/index.html
[`Instant`]: https://docs.rs/ogle/latest/ogle/time_wrapper/struct.Instant.html
[`Duration`]: https://docs.rs/ogle/latest/ogle/time_wrapper/struct.Duration.html
[`chrono`]: https://docs.rs/chrono/latest/chrono/index.html
[`sys`]: https://docs.rs/ogle/latest/ogle/sys/index.html
[`sys::SysApi`]: https://docs.rs/ogle/latest/ogle/sys/trait.SysApi.html
[`sys::SysReal`]: https://docs.rs/ogle/latest/ogle/sys/struct.SysReal.html
[`sys::SysVirtual`]: https://docs.rs/ogle/latest/ogle/sys/struct.SysVirtual.html

