This is [LibMPSSE-I2C](http://www.ftdichip.com/Support/SoftwareExamples/MPSSE/LibMPSSE-I2C.htm) bindings for Rust.
Please install LibMPSSE before using this library.

In order to use this library, this C code should compile and run in your environment
with `$ gcc example.c -lMPSSE` or simillar commands.

```
#include <libMPSSE_i2c.h>
#include <stdio.h>

int main() {
    uint32 x;
    I2C_GetNumChannels(&x);
    printf("%u", x);
    return 0;
}
```

## Example

```main.rs
extern crate mpsse_i2c;

use mpsse_i2c::get_num_channels;

fn main() {
    println!(
        "{}",
        get_num_channels().expect("failed to get the number of channels")
    );
}
```

```Cargo.toml
[dependencies]
mpsse-i2c = { git = "https://github.com/ichyo/mpsse-i2c-rust" }
```
