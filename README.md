# OSC Cue monitor

## What's this?

This is a small utility that will receive messages using the [OSC protocol](https://en.wikipedia.org/wiki/Open_Sound_Control)
and, when it receives messages matching a configurable pattern, it will print a part of that address in a window.


## ... but why would one want to do that?

We've used this in a theatre production, where we had the light board send OSC messages each time a cue was fired. This utility
was developed and used to display the most recently activated cue on a big screen, so that throughout the tech rehearsal process,
everyone in the room knew which cue we're currently in.


## How to use

### Installation

Currently this utility is distributed as source code only; to run it, you'll need to have [Rust installed](https://www.rust-lang.org/tools/install)
and then compile it using `cargo build`, or just `cargo run` it.

### Configuration

The utility will look for a configuration file named `osc-cue-monitor.ini`. The configuration file included in this repo
should work out of the box, but will receive OSC messages only from localhost. In our case, we used ETClabs' [OSCRouter](https://github.com/ETCLabs/OSCRouter),
which was running on the same machine, to subscribe to OSC messages from the light board, and have them distributed to various destinations
including this utility. It'd also be possible to have an external sender (such as the light board) transmit to the machine running this utility
directly; in that case, the `bind_addr` IP address would need to be changed to the IP address this machine is listening to, or to `0.0.0.0`.

The `cue_regex` is a [Regular Expression](https://en.wikipedia.org/wiki/Regular_expression). Note that the default, as provided in
the configuration file in this repository, matches [QLab's](https://qlab.app/) pattern for a "go" cue. This can be changed to any address
pattern as needed. The regex should have exactly one one capture group, and the content of that capture group is what's being displayed
in the utility's window.
