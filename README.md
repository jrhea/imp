# Imp: 😈 Eth2 Network Agent 😈

**Imp** is a semi-autonomous network agent designed to crawl the eth2 network an collect data that will be used to monitor the network health in realtime.

## Demo

Here's a screencap of a couple of imp agents connecting to a sigp/lighthouse node & validator pair:

![imp](https://github.com/prrkl/docs/blob/master/resources/imphouse.gif)

## Docs

For more information on prkl click [here](https://github.com/prrkl/docs/blob/master/README.md)

![imp](https://camo.githubusercontent.com/f5284bbbff6acecfe71b9f7bdded3bfcc0894164/68747470733a2f2f692e696d6775722e636f6d2f564f68714832662e6a7067)

## Prereqs ( MacOS )

Rust:

Install rustup so you can switch between Rust versions:

```sh

> brew install rustup

```

Install the Rust compiler and package manager:

```sh

> rustup-init

```

Tmux (Optional):

Installing this will make running the demo easier:

```sh

> brew install tmux

```

## How to Build

Clone the repo:

```sh

> git clone https://github.com/prrkl/imp.git

```

For convenience, a Makefile is provided to alias the different build options with the following targets:

### debug

This build target is for developers and used (obviously) for debugging.

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;command: &nbsp;&nbsp;&nbsp;`make debug`

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;equivalent to: &nbsp;&nbsp;&nbsp;`cargo build`

### release

If you just want to run **imp**, then use this target.

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;command: &nbsp;&nbsp;&nbsp;`make release`

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;equivalent to: &nbsp;&nbsp;&nbsp;`cargo build --release`

### local

This is an option for developers.  It is a convenient was to reference packages, like [mothra](https://github.com/prrkl/mothra), that are being developed along with **imp**.

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;command: &nbsp;&nbsp;&nbsp;`make local`

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;equivalent to: &nbsp;&nbsp;&nbsp;`cargo build --features "local" --no-default-features`

## How to run

**Imp** is early in development and has limited functionality; however, if you want to play around with it, then I suggest running the demo script.

```sh

> cd scripts && sh demo.sh

```

Here is a glimpse of the available options:

```
> imp -h
imp 0.1.0
Jonny Rhea
Eth2 Network Agent

USAGE:
    imp [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --debug-level <LEVEL>
            Log filter. [default: info]  [possible values: info, debug, trace, warn, error, crit]

        --p2p-protocol-version <P2P_PROTOCOL_VERSION>    P2P protocol version to advertise. [default: imp/libp2p]
        --testnet-dir <DIR>                              The location of the testnet directory to use.

SUBCOMMANDS:
    help      Prints this message or the help of the given subcommand(s)
    mothra    P2P networking component.



> imp mothra -h
imp-mothra 0.1.0
P2P networking component.

USAGE:
    imp mothra [FLAGS] [OPTIONS]

FLAGS:
    -a, --auto-ports                 Allow the OS to select from available TCP/UDP ports.
    -d, --disable-enr-auto-update    This fixes the ENR's IP/PORT to whatever is specified at startup.
    -h, --help                       Prints help information
    -V, --version                    Prints version information

OPTIONS:
        --boot-nodes <ENR-LIST>           One or more comma-delimited base64-encoded ENR's to bootstrap the p2p network.
        --datadir <DIR>                   The location of the data directory to use.
        --debug-level <LEVEL>             Log filter. [default: info]  [possible values: info, debug, trace, warn,
                                          error, crit]
        --discovery-port <PORT>           The discovery UDP port.
        --libp2p-addresses <MULTIADDR>    One or more comma-delimited multiaddrs to manually connect to a libp2p peer
                                          without an ENR.
        --listen-address <ADDRESS>        The address the client will listen for UDP and TCP connections. [default:
                                          127.0.0.1]
        --maxpeers <maxpeers>             The maximum number of peers. [default: 10]
        --port <PORT>                     The TCP/UDP port to listen on. [default: 9000]
        --topics <STRING>                 One or more comma-delimited gossipsub topics to subscribe to.

```
