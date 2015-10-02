#![cfg_attr(feature = "nightly", feature(type_macros))]

use std::cell::{Cell, RefCell};

pub trait CallWatcher { fn call_count(&self) -> u32;
  fn was_called_n_times(&self, times: u32) -> bool { self.call_count() == times }
  fn was_called_once(&self) -> bool { self.was_called_n_times(1) }
  fn was_called(&self) -> bool { self.call_count() != 0 }
}

pub trait ReturnStubber<T> {
  fn returns(&mut self, val: T);
}

pub struct SimpleStub<T: Clone> {
  pub return_val: Option<T>,
  pub call_count: Cell<u32>
}

pub struct ArgWatchingStub<T: Clone, Args> {
  pub return_val: Option<T>,
  pub call_args: RefCell<Vec<Args>>
}

pub struct InterceptingStub<T: Clone, Interceptor: ?Sized> {
  pub return_val: Option<T>,
  pub call_interceptor: Option<Box<Interceptor>>,
  pub call_count: Cell<u32>,
}

impl<T: Clone> SimpleStub<T> {
  pub fn new() -> SimpleStub<T> {
    SimpleStub {
      return_val: None,
      call_count: Cell::new(0)
    }
  }
}

impl<T: Clone, Args> ArgWatchingStub<T, Args> {
  pub fn new() -> ArgWatchingStub<T, Args> {
    ArgWatchingStub {
      return_val: None,
      call_args: RefCell::new(Vec::new())
    }
  }
}

impl<T: Clone, Interceptor: ?Sized> InterceptingStub<T, Interceptor> {
  pub fn new() -> InterceptingStub<T, Interceptor> {
    InterceptingStub {
      return_val: None,
      call_interceptor: None,
      call_count: Cell::new(0),
    }
  }
}

impl<T: Clone> ReturnStubber<T> for SimpleStub<T> {
  fn returns(&mut self, val: T) { self.return_val = Some(val); }
}

impl<T: Clone, Args> ReturnStubber<T> for ArgWatchingStub<T, Args> {
  fn returns(&mut self, val: T) { self.return_val = Some(val); }
}

impl<T: Clone, Interceptor: ?Sized> ReturnStubber<T> for InterceptingStub<T, Interceptor> {
  fn returns(&mut self, val: T) { self.return_val = Some(val); }
}

impl<T: Clone, Args> CallWatcher for ArgWatchingStub<T, Args> {
  fn call_count(&self) -> u32 { self.call_args.borrow().len() as u32 }
}

impl<T: Clone> CallWatcher for SimpleStub<T> {
  fn call_count(&self) -> u32 { self.call_count.get() }
}

impl<T: Clone, Interceptor: ?Sized> CallWatcher for InterceptingStub<T, Interceptor> {
  fn call_count(&self) -> u32 { self.call_count.get() }
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

impl<T: Clone, Interceptor: ?Sized> InterceptingStub<T, Interceptor> {
  pub fn set_interceptor(&mut self, f: Box<Interceptor>) {
    self.call_interceptor = Some(f)
  }
}

#[macro_export]
macro_rules! impl_helper {
  (ArgWatchingStub: $fn_ident:ident (&self $(, $arg_ident:ident: $arg_type:ty)*) -> $ret_type:ty) => {
    fn $fn_ident (&self, $($arg_ident: $arg_type),*) -> $ret_type {
      match self.$fn_ident.return_val.clone() {
        Some(val) => {
          let mut args = self.$fn_ident.call_args.borrow_mut();
          args.push(($($arg_ident),*));
          val
        },
        _ => panic!("#returns was not called on [{}] prior to invocation", stringify!($fn_ident))
      }
    }
  };
  (InterceptingStub: $fn_ident:ident (&self $(, $arg_ident:ident: $arg_type:ty)*) -> $ret_type:ty) => {
    fn $fn_ident (&self, $($arg_ident: $arg_type),*) -> $ret_type {
      match self.$fn_ident.return_val.clone() {
        Some(val) => {
          match self.$fn_ident.call_interceptor {
            Some(ref method) => method($($arg_ident),*),
            None => ()
          }
          self.$fn_ident.call_count.set(1 + self.$fn_ident.call_count.get());
          val
        },
        _ => panic!("#returns was not called on [{}] prior to invocation", stringify!($fn_ident))
      }
    }
  };
  (SimpleStub: $fn_ident:ident (&self $(, $arg_ident:ident: $arg_type:ty)*) -> $ret_type:ty) => {
    #[allow(unused_variables)]
    fn $fn_ident (&self, $($arg_ident: $arg_type),*) -> $ret_type {
      match self.$fn_ident.return_val.clone() {
        Some(val) => {
          self.$fn_ident.call_count.set(1 + self.$fn_ident.call_count.get());
          val
        },
        _ => panic!("#returns was not called on [{}] prior to invocation", stringify!($fn_ident))
      }
    }
  };
  (ArgWatchingStub: $fn_ident:ident (&mut self $(, $arg_ident:ident: $arg_type:ty)*) -> $ret_type:ty) => {
    fn $fn_ident (&mut self, $($arg_ident: $arg_type),*) -> $ret_type {
      match self.$fn_ident.return_val.clone() {
        Some(val) => {
          let mut args = self.$fn_ident.call_args.borrow_mut();
          args.push(($($arg_ident),*));
          val
        },
        _ => panic!("#returns was not called on [{}] prior to invocation", stringify!($fn_ident))
      }
    }
  };
  (InterceptingStub: $fn_ident:ident (&mut self $(, $arg_ident:ident: $arg_type:ty)*) -> $ret_type:ty) => {
    fn $fn_ident (&mut self, $($arg_ident: $arg_type),*) -> $ret_type {
      match self.$fn_ident.return_val.clone() {
        Some(val) => {
          match self.$fn_ident.call_interceptor {
            Some(ref method) => method($($arg_ident),*),
            None => ()
          }
          self.$fn_ident.call_count.set(1 + self.$fn_ident.call_count.get());
          val
        },
        _ => panic!("#returns was not called on [{}] prior to invocation", stringify!($fn_ident))
      }
    }
  };
  (SimpleStub: $fn_ident:ident (&mut self $(,$arg_ident:ident: $arg_type:ty)*) -> $ret_type:ty) => {
    #[allow(unused_variables)]
    fn $fn_ident (&mut self, $($arg_ident: $arg_type),*) -> $ret_type {
      match self.$fn_ident.return_val.clone() {
        Some(val) => {
          self.$fn_ident.call_count.set(1 + self.$fn_ident.call_count.get());
          val
        },
        _ => panic!("#returns was not called on [{}] prior to invocation", stringify!($fn_ident))
      }
    }
  };
  (nostub: $fn_ident:ident (&self $(, $arg_ident:ident: $arg_type:ty)*) -> $ret_type:ty) => {
    fn $fn_ident (&self, $(_: $arg_type),*) -> $ret_type {
      panic!("Method [{}] was not stubbed", stringify!($fn_ident))
    }
  };
  (nostub: $fn_ident:ident (&mut self $(,$arg_ident:ident: $arg_type:ty)*) -> $ret_type:ty) => {
    fn $fn_ident (&mut self, $(_: $arg_type),*) -> $ret_type {
      panic!("Method [{}] was not stubbed", stringify!($fn_ident))
    }
  };
  (nostub: $fn_ident:ident (self $(, $arg_ident:ident: $arg_type:ty)*) -> $ret_type:ty) => {
    fn $fn_ident (self, $(_: $arg_type),*) -> $ret_type {
      panic!("Method [{}] was not stubbed and self-consuming methods cannot currently be stubbed", stringify!($fn_ident))
    }
  };
  (nostub: $fn_ident:ident ($($arg_ident:ident: $arg_type:ty),*) -> $ret_type:ty) => {
    fn $fn_ident ($(_: $arg_type),*) -> $ret_type {
      panic!("Method [{}] was not stubbed and static methods cannot currently be stubbed", stringify!($fn_ident))
    }
  };
}

#[macro_export]
macro_rules! instrument_stub {
  (
    $new_type:ty as $tr8:ident {
      $({$($e:tt)*})*
    }
  ) => {
    impl $tr8 for $new_type {
      $(impl_helper!($($e)*);)*
    }
  }

}

#[cfg(feature = "nightly")]
#[macro_use]
mod type_macros {

  #[macro_export]
  macro_rules! build_stub_type {
    (ArgWatchingStub ($($arg_type:ty),*) -> $ret_type:ty) => {
      ArgWatchingStub<$ret_type, ($($arg_type),*)>
    };
    (SimpleStub ($($arg_type:ty),*) -> $ret_type:ty) => {
      SimpleStub<$ret_type>
    };
    (InterceptingStub ($($arg_type:ty),*) -> $ret_type:ty) => {
      InterceptingStub<$ret_type, Fn($($arg_type),*)>
    };
  }

  #[macro_export]
  macro_rules! create_stub {
    (
      $new_type:ident {
        $({$stub_ty:ident: $fn_ident:ident $($e:tt)*})*
      }
    ) => {
      struct $new_type {
        $($fn_ident: build_stub_type!($stub_ty $($e)*)),*
      }

      impl $new_type {
        fn new() -> $new_type {
          $new_type {
            $($fn_ident: $stub_ty::new()),*
          }
        }
      }
    }
  }
}


#[cfg(test)]
mod tests {
}
