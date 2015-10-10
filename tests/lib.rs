#![cfg_attr(feature = "nightly", feature(type_macros))]

#[macro_use]
extern crate rusty_mock;

mod default_tests {
  #[test]
  fn it_works() {
      assert_eq!(4, 4);
  }
}

mod no_stub {
  trait Trait {
    fn no_self_fn();
    fn self_fn(&self);
    fn mut_self_fn(&mut self);
    fn own_self_fn(self);
    fn self_fn_args(&self, i32, &i32);
  }

  struct TraitStub;

  impl TraitStub {
    fn new() -> TraitStub { TraitStub }
  }

  instrument_stub! {
    TraitStub as Trait {
      {nostub: no_self_fn () -> ()}
      {nostub: self_fn (&self) -> ()}
      {nostub: mut_self_fn (&mut self) -> ()}
      {nostub: own_self_fn (self) -> ()}
      {nostub: self_fn_args (&self, a: i32, b: &i32) -> ()}
    }
  }

  #[test]
  #[should_panic(expected = "Method [no_self_fn] was not stubbed and static methods cannot currently be stubbed")]
  fn panics_when_not_stubbed_no_self() {
    let _ = TraitStub::no_self_fn();
  }

  #[test]
  #[should_panic(expected = "Method [self_fn] was not stubbed")]
  fn panics_when_not_stubbed_self_fn() {
    TraitStub::new().self_fn()
  }

  #[test]
  #[should_panic(expected = "Method [mut_self_fn] was not stubbed")]
  fn panics_when_not_stubbed_mut_self_fn() {
    TraitStub::new().mut_self_fn()
  }

  #[test]
  #[should_panic(expected = "Method [own_self_fn] was not stubbed and self-consuming methods cannot currently be stubbed")]
  fn panics_when_not_stubbed_consuming_self_fn() {
    TraitStub::new().own_self_fn()
  }

  #[test]
  #[should_panic(expected = "Method [self_fn_args] was not stubbed")]
  fn panics_when_not_stubbed_self_fn_args() {
    TraitStub::new().self_fn_args(5, &5)
  }
}

mod simple_stub {
  use rusty_mock::*;

  trait Trait {
    fn self_fn(&self);
    fn mut_self_fn(&mut self);
    fn self_fn_args_return(&self, i32, &i32) -> i32;
  }

  struct TraitStub {
    self_fn: SimpleStub<()>,
    mut_self_fn: SimpleStub<()>,
    self_fn_args_return: SimpleStub<i32>
  }

  impl TraitStub {
    fn new() -> TraitStub {
      TraitStub {
        self_fn: SimpleStub::new(),
        mut_self_fn: SimpleStub::new(),
        self_fn_args_return: SimpleStub::new()
      }
    }
  }

  instrument_stub! {
    TraitStub as Trait {
      {SimpleStub: self_fn (&self) -> ()}
      {SimpleStub: mut_self_fn (&mut self) -> ()}
      {SimpleStub: self_fn_args_return (&self, a: i32, b: &i32) -> i32}
    }
  }

  #[test]
  #[should_panic(expected = "#returns was not called on [self_fn] prior to invocation")]
  fn panics_when_return_not_called_earlier() {
    TraitStub::new().self_fn()
  }

  #[test]
  fn responds_correctly_to_was_called_before_call() {
    let stub = TraitStub::new();
    assert!(!stub.self_fn.was_called());
    assert!(!stub.mut_self_fn.was_called());
    assert!(!stub.self_fn_args_return.was_called());
  }

  #[test]
  fn records_calls_on_stubs() {
    let mut stub = TraitStub::new();
    stub.self_fn.returns(()); // TODO: Make this unnecessary, I know this is silly
    stub.mut_self_fn.returns(());
    stub.self_fn();
    stub.mut_self_fn();
    assert!(stub.self_fn.was_called());
    assert!(stub.mut_self_fn.was_called());
    assert!(stub.self_fn.was_called_once());
    assert!(stub.mut_self_fn.was_called_once());
    stub.self_fn();
    stub.mut_self_fn();
    assert!(stub.self_fn.was_called_n_times(2));
    assert!(stub.mut_self_fn.was_called_n_times(2));
  }

  #[test]
  fn returns_the_correct_value() {
    let mut stub = TraitStub::new();
    stub.self_fn_args_return.returns(10);
    let result = stub.self_fn_args_return(1, &1);
    assert!(result == 10);
  }

}

mod arg_watching_stub {
  use rusty_mock::*;

  trait Trait {
    fn self_fn(&self);
    fn mut_self_fn(&mut self);
    fn self_fn_args_return(&self, i32) -> i32;
  }

  struct TraitStub {
    self_fn: ArgWatchingStub<(), ()>,
    mut_self_fn: ArgWatchingStub<(), ()>,
    self_fn_args_return: ArgWatchingStub<i32, (i32)>
  }

  impl TraitStub {
    fn new() -> TraitStub {
      TraitStub {
        self_fn: ArgWatchingStub::new(),
        mut_self_fn: ArgWatchingStub::new(),
        self_fn_args_return: ArgWatchingStub::new()
      }
    }
  }

  instrument_stub! {
    TraitStub as Trait {
      {ArgWatchingStub: self_fn (&self) -> ()}
      {ArgWatchingStub: mut_self_fn (&mut self) -> ()}
      {ArgWatchingStub: self_fn_args_return (&self, a: i32) -> i32}
    }
  }

  #[test]
  #[should_panic(expected = "#returns was not called on [self_fn] prior to invocation")]
  fn panics_when_return_not_called_earlier() {
    TraitStub::new().self_fn()
  }

  #[test]
  fn responds_correctly_to_was_called_before_call() {
    let stub = TraitStub::new();
    assert!(!stub.self_fn.was_called());
    assert!(!stub.mut_self_fn.was_called());
    assert!(!stub.self_fn_args_return.was_called());
  }

  #[test]
  fn records_calls_on_stubs() {
    let mut stub = TraitStub::new();
    stub.self_fn.returns(()); // TODO: Make this unnecessary, I know this is silly
    stub.mut_self_fn.returns(());
    stub.self_fn();
    stub.mut_self_fn();
    assert!(stub.self_fn.was_called());
    assert!(stub.mut_self_fn.was_called());
    assert!(stub.self_fn.was_called_once());
    assert!(stub.mut_self_fn.was_called_once());
    stub.self_fn();
    stub.mut_self_fn();
    assert!(stub.self_fn.was_called_n_times(2));
    assert!(stub.mut_self_fn.was_called_n_times(2));
  }

  #[test]
  fn returns_the_correct_value() {
    let mut stub = TraitStub::new();
    stub.self_fn_args_return.returns(10);
    let result = stub.self_fn_args_return(1);
    assert!(result == 10);
  }

  #[test]
  fn remembers_the_call_args() {
    let mut stub = TraitStub::new();
    stub.self_fn_args_return.returns(10);
    let _ = stub.self_fn_args_return(1);
    assert!(stub.self_fn_args_return.was_called_with_args(&(1)));
    assert!(!stub.self_fn_args_return.was_called_with_args(&(2)));
    let _ = stub.self_fn_args_return(1);
    assert!(stub.self_fn_args_return.always_called_with_args(&(1)));
    let _ = stub.self_fn_args_return(2);
    assert!(stub.self_fn_args_return.was_called_with_args(&(2)));
  }
}

mod intercepting_stub {
  use rusty_mock::*;

  trait Trait {
    fn self_fn(&self);
    fn mut_self_fn(&mut self);
    fn self_fn_args_return(&self, i32, &i32) -> i32;
  }

  struct TraitStub {
    self_fn: InterceptingStub<(), Fn()>,
    mut_self_fn: InterceptingStub<(), Fn()>,
    self_fn_args_return: InterceptingStub<i32, Fn(i32, &i32)>
  }

  impl TraitStub {
    fn new() -> TraitStub {
      TraitStub {
        self_fn: InterceptingStub::new(),
        mut_self_fn: InterceptingStub::new(),
        self_fn_args_return: InterceptingStub::new()
      }
    }
  }

  instrument_stub! {
    TraitStub as Trait {
      {InterceptingStub: self_fn (&self) -> ()}
      {InterceptingStub: mut_self_fn (&mut self) -> ()}
      {InterceptingStub: self_fn_args_return (&self, a: i32, b: &i32) -> i32}
    }
  }

  #[test]
  #[should_panic(expected = "#returns was not called on [self_fn] prior to invocation")]
  fn panics_when_return_not_called_earlier() {
    TraitStub::new().self_fn()
  }

  #[test]
  fn responds_correctly_to_was_called_before_call() {
    let stub = TraitStub::new();
    assert!(!stub.self_fn.was_called());
    assert!(!stub.mut_self_fn.was_called());
    assert!(!stub.self_fn_args_return.was_called());
  }

  #[test]
  fn records_calls_on_stubs() {
    let mut stub = TraitStub::new();
    stub.self_fn.returns(()); // TODO: Make this unnecessary, I know this is silly
    stub.mut_self_fn.returns(());
    stub.self_fn();
    stub.mut_self_fn();
    assert!(stub.self_fn.was_called());
    assert!(stub.mut_self_fn.was_called());
    assert!(stub.self_fn.was_called_once());
    assert!(stub.mut_self_fn.was_called_once());
    stub.self_fn();
    stub.mut_self_fn();
    assert!(stub.self_fn.was_called_n_times(2));
    assert!(stub.mut_self_fn.was_called_n_times(2));
  }

  #[test]
  fn returns_the_correct_value() {
    let mut stub = TraitStub::new();
    stub.self_fn_args_return.returns(10);
    let result = stub.self_fn_args_return(1, &2);
    assert!(result == 10);
  }

  #[test]
  #[should_panic(expected = "Successfully executed interceptor test")]
  fn calls_the_associated_fn_with_the_args() {
    let mut stub = TraitStub::new();
    stub.self_fn_args_return.returns(10);
    stub.self_fn_args_return.set_interceptor(Box::new(move |x, y| {
      assert!(x == *y && x == 1);
      // Panicing is not suggested usage to indicate completion, but for this test it proves that
      //   the test was executed
      panic!("Successfully executed interceptor test");
    }));
    let _ = stub.self_fn_args_return(1, &1);
  }
}

mod multiple_stub_types {
  use rusty_mock::*;

  trait Trait {
    fn arg_watching_stub(&self, i32, String) -> i32;
    fn intercepting_stub(&mut self, &i32, &str) -> i32;
    fn no_stub(&self, i32, &i32, &str, &Box<Vec<Vec<Vec<&str>>>>) -> i32;
  }

  struct TraitStub {
    arg_watching_stub: ArgWatchingStub<i32, (i32, String)>,
    intercepting_stub: InterceptingStub<i32, Fn(&i32, &str)>,
  }

  impl TraitStub {
    fn new() -> TraitStub {
      TraitStub {
        arg_watching_stub: ArgWatchingStub::new(),
        intercepting_stub: InterceptingStub::new(),
      }
    }
  }

  instrument_stub! {
    TraitStub as Trait {
      {ArgWatchingStub: arg_watching_stub (&self, a: i32, b: String) -> i32}
      {InterceptingStub: intercepting_stub (&mut self, a: &i32, b: &str) -> i32}
      {nostub: no_stub (&self, a: i32, b: &i32, c: &str, d: &Box<Vec<Vec<Vec<&str>>>>) -> i32}
    }
  }

  #[test]
  #[should_panic(expected = "Method [no_stub] was not stubbed")]
  fn non_stubbed_function_behaves_as_expected() {
    let _ = TraitStub::new().no_stub(1, &2, "three", &Box::new(vec![vec![vec!["four"]]]));
  }

  #[test]
  fn arg_watching_function_behaves_as_expected() {
    let mut stub = TraitStub::new();
    stub.arg_watching_stub.returns(10);
    let _ = stub.arg_watching_stub(1, "Hello".to_owned());
    assert!(stub.arg_watching_stub.was_called_with_args(&(1, "Hello".to_owned())));
    assert!(!stub.arg_watching_stub.was_called_with_args(&(2, "Hello".to_owned())));
    let _ = stub.arg_watching_stub(1, "Hello".to_owned());
    assert!(stub.arg_watching_stub.always_called_with_args(&(1, "Hello".to_owned())));
    let _ = stub.arg_watching_stub(2, "Hello".to_owned());
    assert!(stub.arg_watching_stub.was_called_with_args(&(2, "Hello".to_owned())));
  }

  #[test]
  fn interceptor_function_behaves_as_expeected() {
    let mut stub = TraitStub::new();
    stub.intercepting_stub.returns(10);
    stub.intercepting_stub.set_interceptor(Box::new(move |x, y| {
      assert!(*x == 1);
      assert!(y == "hello");
    }));
    let _ = stub.intercepting_stub(&1, "hello");
    assert!(stub.intercepting_stub.was_called_once());
  }
}

mod multiple_trait_stub {
  use rusty_mock::*;

  trait FirstTrait {
    fn give_a_string(&self, i32) -> String;
  }

  trait SecondTrait {
    fn give_another_string(&self, i32) -> String;
  }

  struct TraitStub {
    give_a_string: SimpleStub<String>,
    give_another_string: SimpleStub<String>,
  }

  impl TraitStub {
    fn new() -> TraitStub {
      TraitStub {
        give_a_string: SimpleStub::new(),
        give_another_string: SimpleStub::new(),
      }
    }
  }

  instrument_stub! {
    TraitStub as FirstTrait {
      {SimpleStub: give_a_string (&self, a: i32) -> String}
    }
  }

  instrument_stub! {
    TraitStub as SecondTrait {
      {SimpleStub: give_another_string (&self, a: i32) -> String}
    }
  }

  fn takes_both_traits<T: FirstTrait + SecondTrait>(x: &T) -> String {
    x.give_a_string(1) + &(x.give_another_string(2))
  }

  #[test]
  fn multiple_traits_can_be_stubbed() {
    let mut stub = TraitStub::new();
    stub.give_a_string.returns("Hello ".to_owned());
    stub.give_another_string.returns("World".to_owned());
    let result = takes_both_traits(&stub);
    assert!(stub.give_a_string.was_called_once());
    assert!(stub.give_another_string.was_called_once());
    assert!(&result == "Hello World");
  }

}

#[cfg(feature = "nightly")]
mod stub_create_macro {
  use rusty_mock::*;

  trait Trait {
    fn arg_watching_stub(&self, u32, u32, u32) -> Result<u32, u32>;
    fn simple_stub(&self, u32) -> u32;
    fn intercepting_stub(&self, &u32) -> u32;
  }

  create_stub! {
    TraitStub {
      {ArgWatchingStub: arg_watching_stub (u32, u32, u32) -> Result<u32, u32>}
      {SimpleStub: simple_stub (u32) -> u32}
      {InterceptingStub: intercepting_stub (&u32) -> u32}
    }
  }

  instrument_stub! {
    TraitStub as Trait {
      {ArgWatchingStub: arg_watching_stub (&self, a1: u32, a2: u32, a3: u32) -> Result<u32, u32>}
      {SimpleStub: simple_stub (&self, b1: u32) -> u32}
      {InterceptingStub: intercepting_stub (&self, b1: &u32) -> u32}
    }
  }

  #[test]
  #[should_panic(expected = "#returns was not called on [arg_watching_stub] prior to invocation")]
  fn panics_when_return_not_defined() {
    let stub = TraitStub::new();
    let _ = stub.arg_watching_stub(1, 2, 3);
  }

  #[test]
  fn it_was_called() {
    let mut stub = TraitStub::new();
    stub.arg_watching_stub.returns(Err(5));
    assert!(!stub.arg_watching_stub.was_called());
    let _ = stub.arg_watching_stub(1, 2, 3);
    assert!(stub.arg_watching_stub.was_called());
    let _ = stub.arg_watching_stub(1, 2, 3);
    assert!(stub.arg_watching_stub.was_called());
  }

  #[test]
  fn it_calls_the_args_correctly() {
    let mut stub = TraitStub::new();
    stub.arg_watching_stub.returns(Err(5));
    let _ = stub.arg_watching_stub(1, 2, 3);
    assert!(stub.arg_watching_stub.was_called_with_args(&(1,2,3)));
  }

  #[test]
  fn it_calls_once() {
    let mut stub = TraitStub::new();
    stub.arg_watching_stub.returns(Err(5));
    let _ = stub.arg_watching_stub(1, 2, 3);
    assert!(stub.arg_watching_stub.was_called_once());
  }

  #[test]
  #[should_panic(expected = "assertion failed: *x == 5")]
  fn it_can_test_borrows_and_fail() {
    let mut stub = TraitStub::new();
    stub.intercepting_stub.set_interceptor(Box::new(move |x: &u32| assert!(*x == 5)));
    stub.intercepting_stub.returns(10);
    let _ = stub.intercepting_stub(&1);
  }

  #[test]
  fn it_can_test_borrows_and_succeed() {
    let mut stub = TraitStub::new();
    stub.intercepting_stub.set_interceptor(Box::new(move |x: &u32| assert!(*x == 1)));
    stub.intercepting_stub.returns(10);
    let _ = stub.intercepting_stub(&1);
    assert!(stub.intercepting_stub.was_called_once());
  }

  #[test]
  fn it_never_calls_with_args() {
    let mut stub = TraitStub::new();
    stub.arg_watching_stub.returns(Err(5));
    let _ = stub.arg_watching_stub(1, 2, 4);
    assert!(stub.arg_watching_stub.never_called_with_args(&(1,2,3)));
    let _ = stub.arg_watching_stub(1, 2, 3);
    assert!(!stub.arg_watching_stub.never_called_with_args(&(1,2,3)));
  }

  #[test]
  fn it_always_calls_with_args() {
    let mut stub = TraitStub::new();
    stub.arg_watching_stub.returns(Err(5));
    let _ = stub.arg_watching_stub(1, 2, 3);
    assert!(stub.arg_watching_stub.always_called_with_args(&(1,2,3)));
    let _ = stub.arg_watching_stub(1, 2, 4);
    assert!(!stub.arg_watching_stub.never_called_with_args(&(1,2,3)));
  }
}
