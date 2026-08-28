# Tontoo UIKit

A Full UI Framework for Making apps on TontooOS 

## Made for TontooOS

Explore more at https://github.com/TontooOS/Libs

## Adding to Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
sdk = { path = "/Library/System/sdk", features = ["UIKit"] }
```

Then at the crate root:

```rust
sdk::preinclude!();
use UIKit::{ /* ... */ };
```

## License

TCL v26.1