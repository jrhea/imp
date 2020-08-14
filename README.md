# Imp: 😈 Eth2 Network Agent 😈

**Imp** is a semi-autonomous network agent designed to crawl the eth2 network an collect data that will be used to monitor the network health in realtime.

**WARNING** Code is not in a stable state.  The project is used for adhoc data collection and will NOT be easy to get working.  Sorry...I haven't decided what the long term plans are.

## Demo

Here's a screencap of two imp agents peered and communicating with a [Lighthouse](https://github.com/sigp/lighthouse) node / validator pair:

![imp](https://github.com/prrkl/docs/blob/master/resources/imphouse.gif)

## QuickStart

If you have [docker](https://docs.docker.com/get-docker) installed and you have already cloned the repo, then running **imp** in crawl mode is easy:

1) `make release-docker`
2) `make crawl-docker`

the output will be stored in csv files in the current dir under .schlesi

## Docs

For more information on prkl click [here](https://github.com/prrkl/docs/blob/master/README.md)

## Prereqs

If you just want to build and run then [docker](https://docs.docker.com/get-docker) is the easiest way. See the QuickStart section above.

### MacOS

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

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;docker command: &nbsp;&nbsp;&nbsp;`make debug-docker`

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;macos command: &nbsp;&nbsp;&nbsp;`make debug`


### release

If you just want to run **imp**, then use this target.

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;docker command: &nbsp;&nbsp;&nbsp;`make release-docker`

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;macos command: &nbsp;&nbsp;&nbsp;`make release`


### local

This is an option for developers.  It is a convenient was to reference packages, like [mothra](https://github.com/prrkl/mothra), that are being developed along with **imp**.

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;command: &nbsp;&nbsp;&nbsp;`make debug-local`


## How to run

There are two modes of operation:

### crawler

This mode will have imp crawl the DHT of an eth2 testnet and output info to a csv

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;docker command: &nbsp;&nbsp;&nbsp;`make crawl-docker`

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;macos command: &nbsp;&nbsp;&nbsp;`cd scripts && sh crawl-network.sh schlesi|topaz num_crawlers snapshot|timehsitory`

### agent

This mode is designed to have imp impersonate an eth2 node and listen to gossip messages on the network. Try the following script to learn more:

```
> cd scripts && sh connect-imp-topaz.sh
```

### cli options:

**imp args:**

```
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
    crawler    ETH2 network crawler.
    help       Prints this message or the help of the given subcommand(s)
    mothra     P2P networking component.
```

**imp crawler args:**

```
> imp crawler -h
imp-crawler 0.1.0
ETH2 network crawler.

USAGE:
    imp crawler [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --boot-nodes <ENR-LIST>       One or more comma-delimited base64-encoded ENR's to bootstrap the p2p network.
        --datadir <DIR>               The location of the data directory to use.
        --listen-address <ADDRESS>    The address the client will listen for UDP and TCP connections. [default:
                                      127.0.0.1]
        --port <PORT>                 The TCP/UDP port to listen on. [default: 9000]
```

**imp mothra args:**

```
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
        --maxpeers <maxpeers>             The maximum number of peers. [default: 50]
        --port <PORT>                     The TCP/UDP port to listen on. [default: 9000]
        --topics <STRING>                 One or more comma-delimited gossipsub topics to subscribe to.

```
