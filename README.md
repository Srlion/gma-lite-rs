# gma-lite

Minimal Rust library to read and write Garry's Mod Addon (.gma) archives.

- Small API: [`crate::read`](src/reader.rs) and [`crate::Builder`](src/builder.rs)
- Types: [`crate::Entry`](src/lib.rs), [`crate::GmaError`](src/lib.rs)
- Format constants: [`crate::HEADER`](src/lib.rs), [`crate::VERSION`](src/lib.rs)

## Installation

Add to your Cargo.toml:

```toml
[dependencies]
gma-lite = "0.1"
```

## Usage

### Read a GMA

```rust
use gma_lite::read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("addon.gma")?;
    let entries = read(&bytes[..])?; // Vec<Entry>

    for e in &entries {
        println!("{} ({} bytes)", e.name(), e.size());
    }
    Ok(())
}
```

### Build a GMA

```rust
use gma_lite::Builder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut b = Builder::new("My Addon", 76561197960287930);
    b.set_author("you");
    b.set_description("Example addon");
    b.file_from_string("lua/autorun/example.lua", "print('hello from gma-lite')");

    let mut out = Vec::new();
    b.write_to(&mut out)?;
    std::fs::write("my_addon.gma", out)?;
    Ok(())
}
```

## API

- Reader: [`crate::read`](src/reader.rs) -> `Result<Vec<crate::Entry>, crate::GmaError>`
- Writer: [`crate::Builder`](src/builder.rs) with `write_to<W: std::io::Write>(&self, w) -> Result<(), crate::GmaError>`
- Types: [`crate::Entry`](src/lib.rs), [`crate::GmaError`](src/lib.rs)

See [src/lib.rs](src/lib.rs) for format details and error types.

## License

MIT
