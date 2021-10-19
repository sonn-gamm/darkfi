# DarkFi v0 user tutorial

Welcome to the dark renaissance. This tutorial will teach you how to
install darkfi on your system, and how to use the testnet to send and
receive anonymous tokens.

This tutorial is intended for standard darkfi users. If you'd like to
run a cashier, see this tutorial: [].

## Download

To run darkfi, we must first install the software. Do this by cloning
the darkfi repo:

```
$ git clone https://github.com/darkrenaissance/darkfi
```

## Build

In the project root directory, run provided Makefile. This will download
the trusted setup params and compile the source code. This might take
some time if it's your first time building the project.

```
$ make
```

## Install

We will now install the project. This will install the binaries on
your device in /usr/local, so you can run darkfi from the command-line
directly. It will also create a new directory for config files at
$HOME/.config/darkfi.  Feel free to review the installed config files,
but you don't need to change anything to run the testnet. The defaults
will work fine.

```
$ sudo make install
```

## Run

Darkfi consists of several software daemons or processes. These daemons
have seperate, isolated concerns.

As a user, your interest is in the `darkfid` daemon.  This is a user
node that interacts with your wallet and communicates with services on
the darkfi network.  It is operated using the `drk` command-line tool.

After the installation, you should have `drk` and `darkfid` binaries in
`/usr/local`. Also, the params and configuration files should be in
`~/.config/darkfi`.

We're now ready to use the testnet.

Open two terminal windows. In one terminal, start `darkfid`:

```
$ darkfid -v
```

And another terminal, run `drk`. This is the command-line interface to
interact with `darkfid`.

```
$ drk -h
drk

USAGE:
    drk [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Increase verbosity

OPTIONS:
    -c, --config <CONFIG>    Sets a custom config file

SUBCOMMANDS:
    deposit     Deposit clear tokens for Dark tokens
    features    Show what features the cashier supports
    hello       Say hello to the RPC
    help        Prints this message or the help of the given subcommand(s)
    id          Get hexidecimal ID for token symbol
    transfer    Transfer Dark tokens to address
    wallet      Wallet operations
    withdraw    Withdraw Dark tokens for clear tokens
```

## Deposit

Let's start by depositing some coins into darkfi.

First, we'll need testnet coins on either Bitcoin or Solana.  For Bitcoin
these can be acquired from a faucet like [].  You will need to switch
your Bitcoin wallet to testnet mode.

For Solana, you can either install the Solana command-line suite or
use sollet.io. Follow this tutorial for the Solana command-line [].
For sollet.io, switch the network to testnet and click the ... to airdrop
yourself some testnet coins.

Now that we have testnet coins we can deposit into DarkFi.

We'll do this by sending testnet coins to the DarkFi cashier, which will
issue darkened versions of the deposted coin. This process of darkening
involves the cashier minting new anonymous tokens that are 1:1 redeemable
for deposits.

To deposit testnet BTC:

```
$ ./target/release/drk deposit btc --network bitcoin

```

To deposit testnet SOL:

```
$ ./target/release/drk deposit sol --network solana

```

To deposit any other asset:

```
$ ./target/release/drk deposit [ASSET] --network solana

```

This command will send a deposit request to the cashier.  After running
it, you should get an address printed to your terminal, like this:

[image]

Using Bitcoin or Solana, deposit the desired tokens to the specified
cashier address. This ...

## Configure

Now that you have a copy of the software on your device, you will need
to compile the project. But first we must configure our preferences.

DarkFi is highly configurable by design. Key system parameters can be
changed inside the config files.

Default config files can be found here: [example/config](example/config).

First create a new directory for your config files:

```
$ mkdir ~/.config/darkfi
```

Copy darkfid.toml and drk.toml to ~/.config/darkfi.

```
$ cp example/config/darkfid.toml example/config/drk.toml ~/.config/darkfi
```

Take some time to familiarize yourself with the config options.
The defaults should be sufficient for most users and are safe to use
for demo purposes.

See the cashier tutorial [] for how to modify `darkfid.toml` to work
with any cashier.

