use core::fmt;
use core::any::{Any, TypeId};
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

#[doc(hidden)]
pub trait CloneToAny {
    fn clone_to_any(&self) -> Box<dyn CloneAny>;
}

pub trait CloneToAnySend {
    fn clone_to_any(&self) -> Box<dyn CloneAny + Send>;
}

pub trait CloneToAnySendSync {
    fn clone_to_any(&self) -> Box<dyn CloneAny + Send + Sync>;
}

impl<T: Any + Clone> CloneToAny for T {
    fn clone_to_any(&self) -> Box<dyn CloneAny> {
        Box::new(self.clone())
    }
}

impl<T: Any + Clone + Send> CloneToAnySend for T {
    fn clone_to_any(&self) -> Box<dyn CloneAny + Send> {
        Box::new(self.clone())
    }
}

impl<T: Any + Clone + Send + Sync> CloneToAnySendSync for T {
    fn clone_to_any(&self) -> Box<dyn CloneAny + Send + Sync> {
        Box::new(self.clone())
    }
}

/// [`Any`] + cloning, no Send/Sync
pub trait CloneAny: Any + CloneToAny {}
impl<T: Any + Clone> CloneAny for T {}

/// [`Any`] + cloning + Send
pub trait CloneAnySend: Any + Send + CloneToAnySend {}
impl<T: Any + Clone + Send> CloneAnySend for T {}

/// [`Any`] + cloning + Send + Sync
pub trait CloneAnySendSync: Any + Send + Sync + CloneToAnySendSync {}
impl<T: Any + Clone + Send + Sync> CloneAnySendSync for T {}

macro_rules! impl_clone {
    ($t:ty, $clone_trait:ident) => {
        impl Clone for Box<$t> {
            #[inline]
            fn clone(&self) -> Box<$t> {
                // Call the correct clone_to_any method based on trait
                CloneToAny::clone_to_any(&**self)
            }
        }

        impl fmt::Debug for $t {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.pad(stringify!($t))
            }
        }
    };
}

// Implement Clone for the trait objects, specifying the appropriate CloneToAny variant
impl Clone for Box<dyn CloneAny> {
    fn clone(&self) -> Box<dyn CloneAny> {
        (**self).clone_to_any()
    }
}

impl Clone for Box<dyn CloneAny + Send> {
    fn clone(&self) -> Box<dyn CloneAny + Send> {
        // Upcast to CloneToAnySend
        let send = self as &dyn CloneToAnySend;
        send.clone_to_any()
    }
}

impl Clone for Box<dyn CloneAny + Send + Sync> {
    fn clone(&self) -> Box<dyn CloneAny + Send + Sync> {
        let sync = self as &dyn CloneToAnySendSync;
        sync.clone_to_any()
    }
}


/// Methods for downcasting from an `Any`-like trait object.
pub trait Downcast {
    fn type_id(&self) -> TypeId;

    unsafe fn downcast_ref_unchecked<T: 'static>(&self) -> &T;
    unsafe fn downcast_mut_unchecked<T: 'static>(&mut self) -> &mut T;
    unsafe fn downcast_unchecked<T: 'static>(self: Box<Self>) -> Box<T>;
}

/// Trait for converting into boxed trait object.
pub trait IntoBox<A: ?Sized + Downcast>: Any {
    fn into_box(self) -> Box<A>;
}

macro_rules! implement {
    ($any_trait:ident $(+ $auto_traits:ident)*) => {
        impl Downcast for dyn $any_trait $(+ $auto_traits)* {
            #[inline]
            fn type_id(&self) -> TypeId {
                self.type_id()
            }

            #[inline]
            unsafe fn downcast_ref_unchecked<T: 'static>(&self) -> &T {
                &*(self as *const Self as *const T)
            }

            #[inline]
            unsafe fn downcast_mut_unchecked<T: 'static>(&mut self) -> &mut T {
                &mut *(self as *mut Self as *mut T)
            }

            #[inline]
            unsafe fn downcast_unchecked<T: 'static>(self: Box<Self>) -> Box<T> {
                Box::from_raw(Box::into_raw(self) as *mut T)
            }
        }

        impl<T: $any_trait $(+ $auto_traits)*> IntoBox<dyn $any_trait $(+ $auto_traits)*> for T {
            #[inline]
            fn into_box(self) -> Box<dyn $any_trait $(+ $auto_traits)*> {
                Box::new(self)
            }
        }
    }
}

implement!(Any);
implement!(Any + Send);
implement!(Any + Send + Sync);
