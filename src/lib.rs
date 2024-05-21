#![cfg_attr(not(test), no_std)]

//! A cell-like type with clone-on-write borrowing.
//!
//! ```rust
//! use cowcell::CowCell;
//!
//! let cell = CowCell::new(44);
//! let mut borrow = cell.borrow();
//!
//! // The borrow transparently dereferences to the original value
//! assert_eq!(*borrow, 44);
//! assert!(!borrow.is_cloned());
//!
//! // The borrow copies the original value on mutable access.
//! *borrow += 1;
//! assert_eq!(*borrow, 45);
//! assert!(borrow.is_cloned());
//!
//! // The original value has not been modified.
//! assert_eq!(*cell, 44);
//! ```

use core::ops::{Deref, DerefMut};

/// A cell that can create borrows with clone-on-write semantics.
#[derive(
	Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord,
)]
#[repr(transparent)]
pub struct CowCell<T> {
	val: T,
}

impl<T> CowCell<T> {
	/// Create a new [`CowCell`] containing the given value.
	#[inline]
	pub const fn new(val: T) -> Self {
		Self { val }
	}

	/// Create a new borrow with copy-on-write semantics.
	#[inline]
	pub const fn borrow(&self) -> CowRef<'_, T> {
		CowRef::new(self)
	}

	/// Consume the [`CowCell`], retrieving the inner value.
	#[inline]
	pub fn into_inner(self) -> T {
		self.val
	}
}

impl<T> Deref for CowCell<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.val
	}
}

impl<T> From<T> for CowCell<T> {
	#[inline]
	fn from(val: T) -> Self {
		Self::new(val)
	}
}

/// A borrow with clone-on-write semantics on mutable access.
///
/// This type will provide zero-cost immutable access to the original
/// value contained in a [`CowCell`]. When the inner type is accessed
/// mutably, this type will clone the value, allowing the user to
/// modify a private copy of `T`.
#[derive(Debug)]
pub struct CowRef<'a, T> {
	ptr: &'a CowCell<T>,
	copy: Option<T>,
}

impl<'a, T> CowRef<'a, T> {
	/// A new borrow from a [`CowCell`].
	#[inline]
	const fn new(ptr: &'a CowCell<T>) -> Self {
		Self { ptr, copy: None }
	}

	/// Returns a reference to the [`CowCell`] that originated this
	/// borrow.
	#[inline]
	pub const fn get_cell(&self) -> &CowCell<T> {
		self.ptr
	}

	/// Get an immutable reference to the inner value.
	#[inline]
	pub const fn get_ref(&self) -> &T {
		match self.copy.as_ref() {
			Some(v) => v,
			None => &self.ptr.val,
		}
	}

	/// Returns [`true`] if this [`CowRef`] has made a copy of the
	/// original value.
	#[inline]
	pub const fn is_cloned(&self) -> bool {
		self.copy.is_some()
	}
}

impl<'a, T: Clone> CowRef<'a, T> {
	/// Get a mutable reference to the inner value, cloning the
	/// original value if necessary.
	#[inline]
	pub fn get_mut(&mut self) -> &mut T {
		self.copy.get_or_insert_with(|| self.ptr.val.clone())
	}

	/// Consume the [`CowRef`], retrieving the inner value. This
	/// clones the original value if a copy was not already made.
	#[inline]
	pub fn into_inner(self) -> T {
		self.copy.unwrap_or_else(|| self.ptr.val.clone())
	}
}

impl<'a, T> From<&'a CowCell<T>> for CowRef<'a, T> {
	#[inline]
	fn from(cell: &'a CowCell<T>) -> Self {
		Self::new(cell)
	}
}

impl<T> Deref for CowRef<'_, T> {
	type Target = T;

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.get_ref()
	}
}

impl<T: Clone> DerefMut for CowRef<'_, T> {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.get_mut()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
}
