[![CI](https://github.com/lpenz/ogle/actions/workflows/ci.yml/badge.svg)](https://github.com/lpenz/ogle/actions/workflows/ci.yml)
[![coveralls](https://coveralls.io/repos/github/lpenz/ogle/badge.svg?branch=main)](https://coveralls.io/github/lpenz/ogle?branch=main)
[![crates.io](https://img.shields.io/crates/v/ogle)](https://crates.io/crates/ogle)
[![packagecloud](https://img.shields.io/badge/deb-packagecloud.io-844fec.svg)](https://packagecloud.io/app/lpenz/debian/search?q=ogle)

# ogle

**ogle** is a program that runs the given command-line periodically,
showing the output only when it is different than the last. It also
shows the date+time of the last change, and a progress bar if the
process has taken more than 3s on the last execution.

ogle allows you to do all kinds of monitoring right from the command
line. It's a more flexible and modern take on *[watch (1)]*


## Demos

### Basics: `df` and `free`

Monitor the free space in `/tmp` by running `df -h /tmp`:

![df -h demo](demos/demo-1-df.gif)

Keeping an eye on the amount of free memory with `free -h` is
essentially the same thing. We can use the `--period` (`-p`) flag to
set the time to wait between executions. A counter is displayed when
`--period` is used:

![free -h demo](demos/demo-2-free.gif)

### Monitoring hosts: `ping` and `fping`

We can also monitor `ping` to see when a particular host gets
resolved. Note that ping is not very ogle-friendly, as its output
includes the RTT which tents to always be different between
executions:

![ping demo](demos/demo-3-ping.gif)

`fping` just tells us if a host is alive or dead, which works better
with ogle. We can use the `--until-success` (`-z`) to make ogle exit
when the host comes online, as `fping` returns a result code to the
shell. That let us know exactly when the host came back:

![fping demo](demos/demo-4-fping.gif)


### Long running commands: `date; sleep`

ogle also has a progress bar that is displayed when the command takes
more than 3s. The progress measurement uses the last execution
duration as the total time. An example using `date` and `sleep` with
the `--shell` (`-c`) flag that passes the argument to the shell:

![sleep demo](demos/demo-5-sleep.gif)


[watch (1)]: https://linux.die.net/man/1/watch

