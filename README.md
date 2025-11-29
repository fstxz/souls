An (incomplete) open source implementation of the Soulseek server. Currently, it doesn't do anything useful, but it's possible to connect to the server from a Soulseek client (only tested with Nicotine+).

New users are created automatically upon connecting.

Thanks to Nicotine+ people for maintaining the [Soulseek protocol documentation](https://github.com/nicotine-plus/nicotine-plus/blob/master/doc/SLSKPROTOCOL.md).

## Building and running

Install [Rust](https://rustup.rs/), then execute the following commands to build the project:

```
git clone https://github.com/fstxz/souls.git
cd souls
cargo build
```

To run the server, execute `cargo run`. By default, it will listen on port `2242`. To configure the port, run `cargo run -- -p 2242`.

## License

This project is licensed under the [GPLv3](LICENSE.txt).
