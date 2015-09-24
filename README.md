# Rusty-Mock

[![Build Status](https://travis-ci.org/acmcarther/rusty-mock.svg?branch=master)](https://travis-ci.org/acmcarther/rusty-mock)

Rusty-Mock provides a super quick macro for mocking out an implementor for your Trait. If you follow the practice of building traits for your application boundaries this lets you test them in isolation. The macro & interface is super raw, so please take care.

## Example
```rust
pub type DoEngineer = String;
pub type DoTools = String;
pub type FinishedProduct = String;

pub trait MyDoer {
  fn do_a_thing(&self, _: DoEngineer, _: DoTools) -> FinishedProduct;
  fn do_that_other_thing(&self, _: DoTools) -> FinishedProduct;
}

pub fn use_a_doer<T: MyDoer>(a: &T) -> FinishedProduct {
  a.do_a_thing("Donald".to_owned(), "Shovel".to_owned())
}

#[cfg(test)]
mod tests {
  use super::*;

  stub! {
    MyDoer as DoerStub {
      fn do_a_thing (person: DoEngineer, tools: DoTools) -> FinishedProduct
      fn do_that_other_thing (tools: DoTools) -> FinishedProduct
    }
  }

  #[test]
  fn gets_it_done() {
    let mut stub = DoerStub::new();
    stub.do_a_thing.returns("a completed thing".to_owned());
    let result = use_a_doer(&stub);
    assert!(result == "a completed thing".to_owned());
    assert!(stub.do_a_thing.was_called_once());
    assert!(stub.do_a_thing.was_called_with_args(&("Donald".to_owned(), "Shovel".to_owned())));
  }
}
```

## Contributing

1. Fork it ( http://github.com/acmcarther/rusty-mock/fork )
2. Create your feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create new Pull Request
