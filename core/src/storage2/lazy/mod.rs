// Copyright 2018-2020 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod entry;
mod lazy_array;
mod lazy_cell;
mod lazy_hmap;
mod lazy_imap;

use self::entry::{
    Entry,
    EntryState,
};
pub use self::{
    lazy_array::{
        LazyArray,
        LazyArrayLength,
    },
    lazy_cell::LazyCell,
    lazy_hmap::LazyHashMap,
    lazy_imap::LazyIndexMap,
};
use super::{
    ClearForward,
    KeyPtr,
    PullForward,
    PushForward,
    StorageFootprint,
};
use crate::storage2::traits2::SpreadLayout;
use ink_primitives::Key;

/// A lazy storage entity.
///
/// This loads its value from storage upon first use.
///
/// # Note
///
/// Use this if the storage field doesn't need to be loaded in some or most cases.
#[derive(Debug)]
pub struct Lazy<T>
where
    T: SpreadLayout,
{
    cell: LazyCell<T>,
}

impl<T> StorageFootprint for Lazy<T>
where
    T: SpreadLayout,
    T: StorageFootprint,
{
    const VALUE: u64 = <T as StorageFootprint>::VALUE;
}

impl<T> PullForward for Lazy<T>
where
    T: SpreadLayout,
    T: StorageFootprint,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        Self {
            cell: <LazyCell<T> as PullForward>::pull_forward(ptr),
        }
    }
}

impl<T> PushForward for Lazy<T>
where
    T: SpreadLayout,
    T: PushForward + StorageFootprint,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        <LazyCell<T> as PushForward>::push_forward(&self.cell, ptr)
    }
}

impl<T> ClearForward for Lazy<T>
where
    T: SpreadLayout,
    T: ClearForward + StorageFootprint,
{
    fn clear_forward(&self, ptr: &mut KeyPtr) {
        <LazyCell<T> as ClearForward>::clear_forward(&self.cell, ptr)
    }
}

impl<T> Lazy<T>
where
    T: SpreadLayout,
{
    /// Creates an eagerly populated lazy storage value.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self {
            cell: LazyCell::new(Some(value)),
        }
    }

    /// Creates a true lazy storage value for the given key.
    #[must_use]
    pub fn lazy(key: Key) -> Self {
        Self {
            cell: LazyCell::lazy(key),
        }
    }
}

impl<T> Lazy<T>
where
    T: SpreadLayout,
    T: StorageFootprint + PullForward,
{
    /// Returns a shared reference to the lazily loaded value.
    ///
    /// # Note
    ///
    /// This loads the value from the contract storage if this did not happed before.
    ///
    /// # Panics
    ///
    /// If loading from contract storage failed.
    #[must_use]
    pub fn get(&self) -> &T {
        self.cell.get().expect("expected Some value")
    }

    /// Returns an exclusive reference to the lazily loaded value.
    ///
    /// # Note
    ///
    /// This loads the value from the contract storage if this did not happed before.
    ///
    /// # Panics
    ///
    /// If loading from contract storage failed.
    #[must_use]
    pub fn get_mut(&mut self) -> &mut T {
        self.cell.get_mut().expect("expected Some value")
    }
}

impl<T> From<T> for Lazy<T>
where
    T: SpreadLayout,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Default for Lazy<T>
where
    T: SpreadLayout,
    T: Default,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> core::cmp::PartialEq for Lazy<T>
where
    T: SpreadLayout,
    T: PartialEq + StorageFootprint + PullForward,
{
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.get(), other.get())
    }
}

impl<T> core::cmp::Eq for Lazy<T> where
    T: Eq + SpreadLayout + StorageFootprint + PullForward
{
}

impl<T> core::cmp::PartialOrd for Lazy<T>
where
    T: SpreadLayout,
    T: PartialOrd + StorageFootprint + PullForward,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(self.get(), other.get())
    }
    fn lt(&self, other: &Self) -> bool {
        PartialOrd::lt(self.get(), other.get())
    }
    fn le(&self, other: &Self) -> bool {
        PartialOrd::le(self.get(), other.get())
    }
    fn ge(&self, other: &Self) -> bool {
        PartialOrd::ge(self.get(), other.get())
    }
    fn gt(&self, other: &Self) -> bool {
        PartialOrd::gt(self.get(), other.get())
    }
}

impl<T> core::cmp::Ord for Lazy<T>
where
    T: SpreadLayout,
    T: core::cmp::Ord + StorageFootprint + PullForward,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(self.get(), other.get())
    }
}

impl<T> core::fmt::Display for Lazy<T>
where
    T: SpreadLayout,
    T: core::fmt::Display + StorageFootprint + PullForward,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(self.get(), f)
    }
}

impl<T> core::hash::Hash for Lazy<T>
where
    T: SpreadLayout,
    T: core::hash::Hash + StorageFootprint + PullForward,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

impl<T> core::convert::AsRef<T> for Lazy<T>
where
    T: SpreadLayout,
    T: StorageFootprint + PullForward,
{
    fn as_ref(&self) -> &T {
        self.get()
    }
}

impl<T> core::convert::AsMut<T> for Lazy<T>
where
    T: SpreadLayout,
    T: StorageFootprint + PullForward,
{
    fn as_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}

impl<T> ink_prelude::borrow::Borrow<T> for Lazy<T>
where
    T: SpreadLayout,
    T: StorageFootprint + PullForward,
{
    fn borrow(&self) -> &T {
        self.get()
    }
}

impl<T> ink_prelude::borrow::BorrowMut<T> for Lazy<T>
where
    T: SpreadLayout,
    T: StorageFootprint + PullForward,
{
    fn borrow_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}

impl<T> core::ops::Deref for Lazy<T>
where
    T: SpreadLayout,
    T: StorageFootprint + PullForward,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> core::ops::DerefMut for Lazy<T>
where
    T: SpreadLayout,
    T: StorageFootprint + PullForward,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}
