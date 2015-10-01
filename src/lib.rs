use std::cell::RefCell;
use std::marker::PhantomData;

pub trait CallWatcher {
  fn call_count(&self) -> u32;
  fn was_called_n_times(&self, times: u32) -> bool { self.call_count() == times }
  fn was_called_once(&self) -> bool { self.was_called_n_times(1) }
  fn was_called(&self) -> bool { self.call_count() != 0 }
}

pub trait ReturnStubber<T> {
  fn returns(&mut self, val: T);
}

pub struct SimpleStub<T: Clone> {
  pub return_val: Option<T>,
  pub call_count: u32
}

pub struct ArgWatchingStub<T: Clone, Args> {
  pub return_val: Option<T>,
  pub call_args: RefCell<Vec<Args>>
}

pub struct InterceptingStub<T: Clone, Args, F: Fn(Args)> {
  pub return_val: Option<T>,
  pub call_interceptor: F,
  pub call_count: u32,
  phantom: PhantomData<Args> // Necessary because Fn(Args) does not count as "using" args
}

impl<T: Clone, Args> ArgWatchingStub<T, Args> {
  pub fn new() -> ArgWatchingStub<T, Args> {
    ArgWatchingStub {
      return_val: None,
      call_args: RefCell::new(Vec::new())
    }
  }
}

impl<T: Clone> ReturnStubber<T> for SimpleStub<T> {
  fn returns(&mut self, val: T) { self.return_val = Some(val); }
}

impl<T: Clone, Args> ReturnStubber<T> for ArgWatchingStub<T, Args> {
  fn returns(&mut self, val: T) { self.return_val = Some(val); }
}

impl<T: Clone, Args, F: Fn(Args)> ReturnStubber<T> for InterceptingStub<T, Args, F> {
  fn returns(&mut self, val: T) { self.return_val = Some(val); }
}

impl<T: Clone, Args> CallWatcher for ArgWatchingStub<T, Args> {
  fn call_count(&self) -> u32 { self.call_args.borrow().len() as u32 }
}

impl<T: Clone> CallWatcher for SimpleStub<T> {
  fn call_count(&self) -> u32 { self.call_count }
}

impl<T: Clone, Args, F: Fn(Args)> CallWatcher for InterceptingStub<T, Args, F> {
  fn call_count(&self) -> u32 { self.call_count }
}

impl<T: Clone, Args: Clone> ArgWatchingStub<T, Args> {
  pub fn get_args_for_call(&self, call: usize) -> Option<Args> {
    self.call_args.borrow()
      .get(call)
      .map(|val| val.clone())
  }
}

impl<T: Clone, Args: PartialEq> ArgWatchingStub<T, Args> {
  pub fn was_called_with_args(&self, args: &Args) -> bool {
    self.call_args.borrow()
      .iter()
      .filter(|call_args| *call_args == args)
      .count() > 0
  }

  pub fn always_called_with_args(&self, args: &Args) -> bool {
    let total_calls = self.call_count();
    self.call_args.borrow()
      .iter()
      .filter(|call_args| *call_args == args)
      .count() as u32 == total_calls
  }

  pub fn never_called_with_args(&self, args: &Args) -> bool {
    !self.was_called_with_args(args)
  }
}

impl<T: Clone, Args, F: Fn(Args)> InterceptingStub<T, Args, F> {
  pub fn set_interceptor(&mut self, f: F) {
    self.call_interceptor = f
  }
}

#[macro_export]
macro_rules! create_stub {
  (
    $new_type:ident {
      $($fn_ident:ident ($($arg_ty:ty),*) -> $ret_ty:ty),*
    }
  ) => {
    struct $new_type {
      $($fn_ident: ArgWatchingStub<$ret_ty, ($($arg_ty),*)>),*,
    }

    impl $new_type {
      fn new() -> $new_type {
        $new_type {
          $($fn_ident: ArgWatchingStub::new()),*
        }
      }
    }
  }
}

#[macro_export]
macro_rules! impl_helper {
  (stub $fn_ident:ident (&self, $($arg_ident:ident: $arg_type:ty),*) -> $ret_type:ty) => {
    fn $fn_ident (&self, $($arg_ident: $arg_type),*) -> $ret_type {
      match self.$fn_ident.return_val.clone() {
        Some(val) => {
          let mut args = self.$fn_ident.call_args.borrow_mut();
          args.push(($($arg_ident),*));
          val
        },
        _ => panic!("#returns was not called on {} prior to invocation", stringify!($fn_ident))
      }
    }
  };
  (stub $fn_ident:ident (&mut self, $($arg_ident:ident: $arg_type:ty),*) -> $ret_type:ty) => {
    fn $fn_ident (&mut self, $($arg_ident: $arg_type),*) -> $ret_type {
      match self.$fn_ident.return_val.clone() {
        Some(val) => {
          let mut args = self.$fn_ident.call_args.borrow_mut();
          args.push(($($arg_ident),*));
          val
        },
        _ => panic!("#returns was not called on {} prior to invocation", stringify!($fn_ident))
      }
    }
  };
  (nostub $fn_ident:ident (&self, $($arg_ident:ident: $arg_type:ty),*) -> $ret_type:ty) => {
    fn $fn_ident (&self, $(_: $arg_type),*) -> $ret_type {
      panic!("Method was not stubbed {}", stringify!($fn_ident))
    }
  };
  (nostub $fn_ident:ident (&mut self, $($arg_ident:ident: $arg_type:ty),*) -> $ret_type:ty) => {
    fn $fn_ident (&mut self, $(_: $arg_type),*) -> $ret_type {
      panic!("Method was not stubbed {}", stringify!($fn_ident))
    }
  };
  (nostub $fn_ident:ident (self, $($arg_ident:ident: $arg_type:ty),*) -> $ret_type:ty) => {
    fn $fn_ident (self, $(_: $arg_type),*) -> $ret_type {
      panic!("Method {} not stubbed, and self-consuming methods cannot currently be stubbed", stringify!($fn_ident))
    }
  };
  (nostub $fn_ident:ident ($($arg_ident:ident: $arg_type:ty),*) -> $ret_type:ty) => {
    fn $fn_ident ($(_: $arg_type),*) -> $ret_type {
      panic!("Method {} not stubbed and static methods cannot currently be stubbed", stringify!($fn_ident))
    }
  };
}

#[macro_export]
macro_rules! instrument_stub {
  (
    $new_type:ident as $tr8:ident {
      $({$fn_ident:ident $($e:tt)*})*
    }
  ) => {
    impl $tr8 for $new_type {
      $(impl_helper!($fn_ident $($e)*);)*
    }
  }

}


#[cfg(test)]
mod tests {
  use super::*;
  type Repository = u32;
  type IssueId = u32;
  type CreateIssueComment = u32;
  type IssueComment = u32;
  type GitErr = u32;

  trait IssueCommenter {
    fn create_comment(&self, _: Repository, _: IssueId, details: CreateIssueComment) -> Result<IssueComment, GitErr>;
    fn create_fun(&self, _: u32) -> u32;
  }

  fn i_take_a_thing<T: IssueCommenter>(a: &T) {
    let _ = a.create_comment(1, 2, 3);
  }

  create_stub! {
    IssueCommenterStub {
      create_comment(Repository, IssueId, CreateIssueComment) -> Result<IssueComment, GitErr>,
      create_fun(u32) -> u32
    }
  }

  instrument_stub! {
    IssueCommenterStub as IssueCommenter {
      {stub create_comment (&self, a1: Repository, a2: IssueId, a3: CreateIssueComment) -> Result<IssueComment, GitErr>}
      {stub create_fun (&self, b1: u32) -> u32}
    }
  }


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
