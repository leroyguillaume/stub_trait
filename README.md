![ci](https://github.com/leroyguillaume/stub_trait/actions/workflows/ci.yml/badge.svg)
![publish](https://github.com/leroyguillaume/stub_trait/actions/workflows/publish.yml/badge.svg)

# stub_trait

Macro to implement stub object for a trait.

## Overview

Stub traits is a technique to simulate some comportments or to avoid to be blocked by a specific part of the code that is not implemented yet.

## Usage

stub_trait is generally only used by tests. Add the following snippet into your `Cargo.toml`:
```toml
[dev-dependencies]
stub_trait = "1.0.0"
```

You can use it like this:
```rust
#[cfg(test)]
use stub_trait::stub;

#[cfg_attr(test, stub)]
trait Animal {
    fn feed(&self, quantity: usize) -> &str;
}

#[cfg(test)]
fn test() {
    let animal = StubAnimal::new().with_stub_of_feed(|i, quantity| {
        if i == 0 {
            assert_eq!(quantity, 10);
            "sad!"
        } else if i == 1 {
            assert_eq!(quantity, 20);
            "happy!"
        } else {
            panic!("too much invocations!")
        }
    });
    assert_eq!(animal.feed(10), "sad!");
    assert_eq!(animal.feed(20), "happy!");
    assert_eq!(animal.count_calls_of_feed(), 2);
}
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) file.
