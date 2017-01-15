/*
Copyright 2017 William Cody Laeder

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

//! Intel RTM Extensions.
//! 
//! Please note this crate only works on x86_64 Intel
//! processors, and only those built after
//! the boardwell 6th generation. 
//!
//!#Basic Intro:
//!
//! RTM works very similiar to a database. You can
//! read/write memory but you have to commit the
//! changes. If another thread modifies the same
//! region as you are, the other RTM transaction
//! will abort (the second chronologically). 
//!
//! RTM transaction can also be cancelled. Meaning
//! if you do not want to commit a transaction
//! as in you wish to roll it back that can be
//! accomplished via `abort(x: u8)` interface
//! within this library if you hit a condition
//! that requires rolling back the transaction.
//!
//!#Deep Dive:
//!
//! Now we need to perform a deep dive into
//! into RTM and it's implementation. RTM works on
//! the cache line level. This means each region
//! RTM _thinks_ it is exclusive to a cache line.
//! Each cache line in Intel CPU's is 64bytes,
//! so you will wish to ensure that your data
//! structures being modified WITHIN RTM 
//! transactions are `X * 64 = size_of::<T>()`
//! or `0 == size_of::<T>() % 64`. At the same
//! time you will wish to ensure the allocation
//! is on the 64 byte boundry (this is called
//! allignment) this simply means
//! `  &T % 64 == 0` (the physical pointer).
//!
//! The reason for this false sharing. If a
//! different thread modifies the same cacheline
//! you have decared RTM your modification may
//! abort reducing your preformance.
//!
//! RTM works via the [MESIF](https://en.wikipedia.org/wiki/MESIF_protocol) protocol. These are
//! the states a Cache Line can be in. E (Exclusive),
//! M (Modified), S (Shared), F (Forward), I (Invalid).
//! Effectively RTM attempts to ensure that all the 
//! writes/reads you will perform are on E/F values
//! (Exclusive/Forward). This means you either own the
//! the only copy of this in Cache OR another thread may
//! read this data, but not write to it.
//!
//! If another thread attempts to write to a cacheline
//! during the RTM transaction the status of your cache
//! will change `E -> S` or `F -> I`. And the other
//! thread is not executing RTM code, your transaction
//! will abort.
//!
//!#Architecture Notes:
//!
//! RTM changes are buffered in L1 cache.
//! so too many changes can result in very extreme
//! performance penalities. 
//!
//! RMT changes are a full instruction barrier, but
//! they are not the same as an `mfence` or `sfence`
//! or `lfence` instruction (only to the local cache
//! lines effected by an RTM transaction). 
//!
//!#Performance Notes:
//!
//! For modification of a single cache line 
//! `AtomicUsize` or `AtomicPtr` will be faster even
//! in `SeqCst` mode. RTM transaction are typically
//! faster for larger transaction on the order of
//! several cache lines (typically `>300` bytes) or so.
//!


#![no_std]
#![feature(link_llvm_intrinsics)]



/// Why the transaction aborted
///
/// This states the reason why.
#[derive(Copy,Clone,Debug)]
pub enum Abort{
  Retry,
  Conflict,
  Capacity,
  Debug,
  Nested,
  Code(i8),
  Undefined
}

/// Aborts a transaction in progress.
///
/// This will unroll any and all
/// writes that have taken place.
///
/// The argumen passed here will be
/// returned as the `Err(Abort::Code(x))`
/// value.
#[inline(always)]
pub fn abort(x: i8) {
  unsafe{_xabort(x)};
}

/// Execute a transaction
///
/// This accepts data and a lambda function. It will return if the operations
/// succeeded or not, and _how_ it failed if it did.
#[inline(always)]
pub fn transaction<R: Sync,F: Fn(&mut R)>(lambda: &F, data: &mut R) -> Result<(),Abort> {
  //bit masks will be reduced to to constants at compile time
  let explicit: i32 = 1 << 0;
  let retry: i32 = 1 << 1;
  let conflict: i32 = 1 << 2;
  let capacity: i32 = 1 << 3;
  let debug: i32 = 1 << 4;
  let nested: i32 = 1 << 5;
  let mut out: Result<(),Abort> = Ok(());
  match unsafe{_xbegin()} {
    -1 => {
      lambda(data);
    },
    x if (x&retry) > 0 => out = Err(Abort::Retry),
    x if (x&conflict) > 0 => out = Err(Abort::Conflict),
    x if (x&capacity) > 0 => out = Err(Abort::Capacity),
    x if (x&debug) > 0 => out = Err(Abort::Debug),
    x if (x&nested) > 0 => out = Err(Abort::Nested),
    x if (x&explicit) > 0 => {
      out = Err(Abort::Code(((x >> 24) & 0xFF) as i8));
    },
    _ => out = Err(Abort::Undefined)
  }; 
  unsafe{_xend()};
  out
}




/// Raw extension bindings
///
/// If a developer would rather roll their own
/// implementation without all the branching
/// and masking check this library does
/// internally.
///
/// If you would like to use these please see:
///
/// [Clang Reference](http://clang.llvm.org/doxygen/rtmintrin_8h.html)
///
/// [GCC Reference](https://gcc.gnu.org/onlinedocs/gcc-4.8.2/gcc/X86-transactional-memory-intrinsics.html)
///
/// [Intel Intrinsic Reference](https://software.intel.com/sites/landingpage/IntrinsicsGuide/#othertechs=RTM)
///
/// [Dr Dobb's Crash Course](http://www.drdobbs.com/parallel/transactional-synchronization-in-haswell/232600598)
///
pub mod tsx {
  extern {
    #[link_name = "llvm.x86.xbegin"]
    pub fn _xbegin() -> i32;
    #[link_name = "llvm.x86.xend"]
    pub fn _xend();
    #[link_name = "llvm.x86.xabort"]
    pub fn _xabort(a: i8);
  }
}
pub use tsx::*;

