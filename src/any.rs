use core::fmt;
use core::any::{Any, TypeId};
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

#[doc(hidden)]
pub trait CloneToAny<T: ?Sized> {
    fn clone_to_any(&self) -> Box<T>;
}

impl<T: Any + Clone> CloneToAny<dyn CloneAny<dyn CloneAnyTrait>> for T {
    fn clone_to_any(&self) -> Box<dyn CloneAny<dyn CloneAnyTrait>> {
        Box::new(self.clone())
    }
}
impl<T: Any + Clone + Send> CloneToAny<dyn CloneAny<dyn CloneAnyTrait + Send>> for T {
    fn clone_to_any(&self) -> Box<dyn CloneAny<dyn CloneAnyTrait + Send>> {
        Box::new(self.clone())
    }
}
impl<T: Any + Clone + Send + Sync> CloneToAny<dyn CloneAny<dyn CloneAnyTrait + Send + Sync>> for T {
    fn clone_to_any(&self) -> Box<dyn CloneAny<dyn CloneAnyTrait + Send + Sync>> {
        Box::new(self.clone())
    }
}

// To simplify and unify trait object naming,
// define a zero-sized marker trait to represent the trait object type for CloneAny
// (Because Rust doesn't allow direct generic parameters on trait objects, so we use a marker)
pub trait CloneAnyTrait {}
impl CloneAnyTrait for dyn CloneAnyTrait {}
impl CloneAnyTrait for dyn CloneAnyTrait + Send {}
impl CloneAnyTrait for dyn CloneAnyTrait + Send + Sync {}

/// [`Any`], but with cloning.
/// Now generic over the boxed trait object `T`.
pub trait CloneAny<T: ?Sized>: Any + CloneToAny<T> {}

// Implement CloneAny for each variant with proper bounds.
impl<T: Any + Clone> CloneAny<dyn CloneAnyTrait> for T {}
impl<T: Any + Clone + Send> CloneAny<dyn CloneAnyTrait + Send> for T {}
impl<T: Any + Clone + Send + Sync> CloneAny<dyn CloneAnyTrait + Send + Sync> for T {}

// Macro to implement Clone for Box<dyn CloneAny<...>>
macro_rules! impl_clone {
    ($t:ty) => {
        impl Clone for Box<$t> {
            #[inline]
            fn clone(&self) -> Box<$t> {
                let clone: Box<$t> = (**self).clone_to_any();
                Box::from_raw(Box::into_raw(clone))
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

// Now the rest of your code is the same as before:

/// Methods for downcasting from an `Any`-like trait object.
///
/// This should only be implemented on trait objects for subtraits of `Any`, though you can
/// implement it for other types and it’ll work fine, so long as your implementation is correct.
pub trait Downcast {
    /// Gets the `TypeId` of `self`.
    fn type_id(&self) -> TypeId;

    unsafe fn downcast_ref_unchecked<T: 'static>(&self) -> &T;
    unsafe fn downcast_mut_unchecked<T: 'static>(&mut self) -> &mut T;
    unsafe fn downcast_unchecked<T: 'static>(self: Box<Self>) -> Box<T>;
}

/// A trait for the conversion of an object into a boxed trait object.
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

implement_clone!(dyn CloneAny<dyn CloneAnyTrait>);
implement_clone!(dyn CloneAny<dyn CloneAnyTrait + Send>);
implement_clone!(dyn CloneAny<dyn CloneAnyTrait + Send + Sync>);
