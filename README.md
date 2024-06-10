# Telepath

A fast, versatile file system link creator written in Rust.

## Installation

Telepath is a self-contained single executable so you don't need to install anything. But for convenience of use, you can put it somewhere in your `PATH` environment variable.

## Usage

Say you've got a complicated project in a directory called `origin` on your local file system. For each file inside its maze of folders, you want to have a symbolic link created in another directory `mirror`, while still maintaining `origin`'s folder structure, so you can test something wild there without messing up `origin`. That's a typical use case of Telegraph:

```shell
telepath mirror origin
```

You can create links to multiple sources, or to individual files, all inside your `mirror`:

```shell
telepath mirror foo bar baz/package.json
```

Some programs don't follow symlinks. You can create hard links instead, with `--hard` or its short version `-H` (options are case-sensitive):

```shell
telepath mirror origin -H
```

Filter things with `--glob` (`-g`), specifying multiple patterns is supported:

```shell
telepath mirror origin -g *.rs *.h *.c src/**/*.json
```

Limit directory depth with `--depth` (`-D`) followed by a number:

```shell
telepath mirror origin -D 3
```

You can tell Telepath not to create folder structures at all, which effectively provides you with a "flattened" view of all the files scattered here and there:

```shell
telepath mirror origin --no-tree --combine
```

> You can checkout all available options with `--help` (`-h`).

## Building from Source

MSRV: 1.79.0

```shell
cargo build --release
```
