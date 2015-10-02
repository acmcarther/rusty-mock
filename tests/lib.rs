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
}

mod intercepting_stub {
}

mod multiple_stub_types {
}

mod multiple_trait_stub {
}

#[cfg(feature = "nightly")]
mod stub_create_macro {
}
/*
  #[test]
  #[should_panic(expected = "#returns was not called on create_comment prior to invocation")]
  fn panics_when_return_not_defined() {
    let stub = IssueCommenterStub::new();
    let _ = stub.create_comment(1, 2, 3);
  }

  #[test]
  fn it_was_called() {
    let mut stub = IssueCommenterStub::new();
    stub.create_comment.returns(Err(5));
    assert!(!stub.create_comment.was_called());
    let _ = stub.create_comment(1, 2, 3);
    assert!(stub.create_comment.was_called());
    let _ = stub.create_comment(1, 2, 3);
    assert!(stub.create_comment.was_called());
  }

  #[test]
  fn it_calls_the_args_correctly() {
    let mut stub = IssueCommenterStub::new();
    stub.create_comment.returns(Err(5));
    let _ = stub.create_comment(1, 2, 3);
    assert!(stub.create_comment.was_called_with_args(&(1,2,3)));
  }

  #[test]
  fn it_calls_once() {
    let mut stub = IssueCommenterStub::new();
    stub.create_comment.returns(Err(5));
    let _ = stub.create_comment(1, 2, 3);
    assert!(stub.create_comment.was_called_once());
  }

  #[test]
  #[should_panic(expected = "assertion failed: *x == 5")]
  fn it_can_test_borrows_and_fail() {
    let mut stub = IssueCommenterStub::new();
    stub.create_more.set_interceptor(Box::new(move |x: &u32| assert!(*x == 5)));
    stub.create_more.returns(10);
    let _ = stub.create_more(&1);
  }

  #[test]
  fn it_can_test_borrows_and_succeed() {
    let mut stub = IssueCommenterStub::new();
    stub.create_more.set_interceptor(Box::new(move |x: &u32| assert!(*x == 1)));
    stub.create_more.returns(10);
    let _ = stub.create_more(&1);
    assert!(stub.create_more.was_called_once());
  }

  #[test]
  fn it_never_calls_with_args() {
    let mut stub = IssueCommenterStub::new();
    stub.create_comment.returns(Err(5));
    let _ = stub.create_comment(1, 2, 4);
    assert!(stub.create_comment.never_called_with_args(&(1,2,3)));
    let _ = stub.create_comment(1, 2, 3);
    assert!(!stub.create_comment.never_called_with_args(&(1,2,3)));
  }

  #[test]
  fn it_always_calls_with_args() {
    let mut stub = IssueCommenterStub::new();
    stub.create_comment.returns(Err(5));
    let _ = stub.create_comment(1, 2, 3);
    assert!(stub.create_comment.always_called_with_args(&(1,2,3)));
    let _ = stub.create_comment(1, 2, 4);
    assert!(!stub.create_comment.never_called_with_args(&(1,2,3)));
  }

  #[test]
  fn it_works_in_place_of_the_trait() {
    let mut stub = IssueCommenterStub::new();
    stub.create_comment.returns(Err(5));
    i_take_a_thing(&stub);
    assert!(stub.create_comment.was_called());
  }
}
*/
