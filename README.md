
# Pretty dtoa

Configurable float and double printing. ``pretty_dtoa`` Comes with lots of options for configuring different aspects of displaying floats.

This crate uses the [ryu-floating-decimal crate](https://github.com/Torrencem/ryu-floating-decimal) (itself a fork of the [ryu crate](https://github.com/dtolnay/ryu)) to generate a "floating decimal", or a floating point with radix 10, and then it uses formatting rules particular to the configuration to create a formatted string.

This module is slightly slow (usually between 1x and 2x slower than the default Display implementation for f64). Benchmarks can be run with ``cargo bench``.

Consider using ``pretty_dtoa`` if:

* Float printing is not a huge bottleneck for your application, or a small slowdown of float formatting is otherwise acceptable

* The default behavior of ``Display`` and alternative float printing libraries like ``ryu`` is not ideal for one reason or another

## Example

```rust
use pretty_dtoa::{dtoa, FmtFloatConfig};

let config = FmtFloatConfig::default()
     .force_no_e_notation()      // Don't use scientific notation
     .add_point_zero(true)       // Add .0 to the end of integers
     .max_significant_digits(4)  // Stop after the first 4 non-zero digits
     .radix_point(',')           // Use a ',' instead of a '.'
     .round();                   // Round after removing non-significant digits

 assert_eq!(dtoa(12459000.0, config), "12460000,0");
```

See the tests in ``src/lib.rs`` for examples of each feature, and [the documentation](https://docs.rs/pretty_dtoa) to see all configurable features.
