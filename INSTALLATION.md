# Installation

[![Release-Crates]][Release-Crates-1]
[![Release-GitHub]][Release-GitHub-1]

You can [download the latest GitHub release][Release-GitHub-1] if there is one for your system and architecture.

You can also install using cargo with:

```shell
RUSTFLAGS="-C target-cpu=native -O" cargo install fixurjavainstall
```

Or by running:

```shell
git clone https://github.com/EpicVon2468/fixurjavainstall
cd fixurjavainstall
RUSTFLAGS="-C target-cpu=native -O" cargo build --release --path .
```

## Documentation

For standalone in-memory documentation:

```shell
fuji --help
```

For UNIX `man` entries:

An additional note: If the version of `fuji` you have installed is <= `0.8.0`, DO NOT RUN `fuji manual`!!!<br>
See also: [GHSA-fq3w-p4fg-mw73](https://github.com/EpicVon2468/fixurjavainstall/security/advisories/GHSA-fq3w-p4fg-mw73).

```shell
# if compiled with `--features dev` (installs to "$PWD/man", only works as long as "$PWD/man" is on the manpath)
fuji manual && export MANPATH="$PWD/man:$(manpath)"
# else (installs to "/usr/share/man")
sudo fuji manual
```

For rustdoc pages in a local website:

```shell
cargo clean --doc
RUSTDOCFLAGS="--default-theme=ayu" cargo doc --document-private-items --all-features --release --color=always --no-deps --open
```

For online rustdoc pages:

<https://docs.rs/fixurjavainstall/latest/fuji/>

[Release-Crates]: https://img.shields.io/crates/v/fixurjavainstall?logo=rust
[Release-Crates-1]: https://crates.io/crates/fixurjavainstall/

[Release-GitHub]: https://img.shields.io/github/v/release/EpicVon2468/fixurjavainstall?logo=github&label=github
[Release-GitHub-1]: https://github.com/EpicVon2468/fixurjavainstall/releases/latest/