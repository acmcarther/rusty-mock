# Rusty-Mock

[![Build Status](https://travis-ci.org/acmcarther/rusty-mock.svg?branch=master)](https://travis-ci.org/acmcarther/rusty-mock)

Rusty-Mock provides a super quick macro for mocking out an implementor for your Trait. If you follow the practice of building traits for your application boundaries this lets you test them in isolation. The macro & interface is super raw, so please take care.

## Example
### Creating some useful traits for us
```Rust
trait Trait1 {
  fn static_method1(u32) -> u32;
  fn static_method2(u32) -> u32;
  fn self_method(&self, i32, i32, i32) -> i32;
  fn mut_self_method(&mut self, i32, i32, i32) -> i32;
  fn consuming_method(self, u32) -> u32;
  fn method_with_ref(&mut self, &i32, i32) -> i32};
}

trait Trait2 {
  fn someone_elses_method(&self, i32) -> i32;
}
```
### Creating a function using something implementing those traits
```rust
fn test_fn<Tester: Trait1 + Trait2>(x: &Tester) {
  x.someone_elses_method(1);
  x.self_method(1, 2, 3);
}
```
### Stubbing out those traits (in your test)
```rust
create_stub! {
  TraitStub2 {
    self_method(i32, i32, i32) -> i32,
    mut_self_method(i32, i32, i32) -> i32,
    someone_elses_method(i32) -> i32
    method_with_ref(i32, i32) -> i32}
  }
}

instrument_stub! {
  TraitStub2 as Trait1 {
    {nostub static_method1(a: u32) -> u32}
    {nostub static_method2(a: u32) -> u32}
    {stub self_method(&self, a: i32, b: i32, c: i32) -> i32}
    {stub mut_self_method(&mut self, a: i32, b: i32, c: i32) -> i32}
    {clone_stub method_with_ref(&mut self, a: &i32, b: i32) -> i32}
    {nostub consuming_method(self, a: u32) -> u32}
  }
}

instrument_stub! {
  TraitStub2 as Trait2 {
    {stub someone_elses_method(&self, a: i32) -> i32}
  }
}
```
Note: Return value must be cloneable so the stub can be called multiple times. If the method being stubbed takes references, you must stub them with clone_stub, and build the stub with the actual type rather than the reference as in the function signature. All of the arguments in a clone_stub method must be cloneable, to avoid lifetime issues.

To recap: Return values must always be cloneable
If a method takes any references it must be stubbed with clone_stub, and all args must be cloneable

### In your test
```rust
#[test]
fn it_was_called() {
  let mut x = TraitStub1::new();
  x.someone_elses_method.returns(5);
  x.self_method.returns(5);
  test_fn(&x);
  assert!(x.self_method.was_called());
  assert!(!x.mut_self_method.was_called());
}
```

## Contributing

1. Fork it ( http://github.com/acmcarther/rusty-mock/fork )
2. Create your feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create new Pull Request
