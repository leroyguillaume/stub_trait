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
stub_trait = "0.2.0"
```

You can use it like this:
```rust
#[cfg(test)]
use stub_trait::stub;

#[cfg_attr(test, stub)]
trait Animal {
    fn name(&self) -> &str;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stub_all_calls() {
        let mut animal = StubAnimal::default();
        animal.stub_all_calls_of_name(|| "Ivana");
        assert_eq!(animal.name(), "Ivana");
        assert_eq!(animal.name(), "Ivana");
        assert_eq!(animal.count_calls_of_name(), 2);
    }

    #[test]
    fn stub_call_by_call() {
        let mut animal = StubAnimal::default();
        animal.register_stub_of_name(|| "Ivana");
        animal.register_stub_of_name(|| "Truffle");
        assert_eq!(animal.name(), "Ivana");
        assert_eq!(animal.name(), "Truffle");
        assert_eq!(animal.count_calls_of_name(), 2);
    }
}
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) file.
