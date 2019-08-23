//! Makes registers ownable and movable
//!
//! The register code generated by svd2rust doesn't allows us to move and own
//! registers. We can only have shared references to them. This becomes
//! inconvenient, if we want to split a peripheral, so multiple components of an
//! API can access it, as every component requires a lifetime then.
//!
//! This module works around this limitation, by introducing a proxy struct that
//! provides access to a register.

use core::marker::PhantomData;
use core::mem::transmute;
use core::ops::Deref;

/// A proxy object for a register
///
/// This proxy can be moved and owned, then provide access to the register it
/// proxies from wherever it is. Access to the register is provided by
/// implementing `Deref`.
pub struct RegProxy<T>
where
    T: Reg,
{
    _marker: PhantomData<*const T>,
}

impl<T> RegProxy<T>
where
    T: Reg,
{
    /// Create a new proxy object
    ///
    /// If this method is used to create multiple proxies for the same register,
    /// using those proxies carelessly can result in race conditions. It's
    /// probably always a mistake to access such shared registers using
    /// `modify`, as these methods do a read-modify-write, which is not atomic.
    ///
    /// How to access a register race-free depends on the specifics of the
    /// hardware:
    /// - Restricting yourself to reading from a register might be safe, but
    ///   please note that even reading a register might have side effects
    ///   (possibly even in other registers).
    /// - Many registers are set up such, that only bits that are written as `1`
    ///   have an effect, while bits written as `0` don't. Such registers can
    ///   often be shared without race conditions.
    /// - Generally speaking, make sure you understand the hardware, and what
    ///   kind of access could or could not lead to race conditions.
    ///
    /// Please note that all of this isn't really different from the raw API
    /// generated by svd2rust, as multiple shared references to the same
    /// register can exist there, and a shared reference is all that's required
    /// to have full control over a register.
    pub fn new() -> Self {
        RegProxy {
            _marker: PhantomData,
        }
    }
}

unsafe impl<T> Send for RegProxy<T> where T: Reg {}

impl<T> Deref for RegProxy<T>
where
    T: Reg,
{
    type Target = T::Target;

    fn deref(&self) -> &Self::Target {
        // As long as `T` upholds the safety restrictions laid out in the
        // documentation of `Reg`, this should be safe. The pointer is valid for
        // the duration of the program. That means:
        // 1. It can always be dereferenced, so casting to a reference is safe.
        // 2. It is essentially `'static`, so casting to any lifetime is safe.
        unsafe { transmute(T::get()) }
    }
}

/// Implemented for registers that `RegProxy` can proxy
///
/// If you want to implement this trait for a register from a crate generated by
/// svd2rust, please use the `reg!` macro.
///
/// # Safety
///
/// The pointer returned by `get` must be valid for the duration of the program.
/// This should always be the case for MMIO registers.
pub unsafe trait Reg {
    /// The type that `RegProxy` should derefence to
    ///
    /// If only one instance of the register exists, this should be `Self`.
    /// If the same type in the svd2rust API is used to represent registers at
    /// multiple memory locations, this trait must be implemented for a type
    /// that represents a specific register at a specific location, and `Target`
    /// must be the common type.
    type Target;

    /// Return a pointer to the memory location of the register
    fn get() -> *const Self::Target;
}

#[macro_export]
macro_rules! reg {
    ($ty:ident, $target:ty, $peripheral:path, $field:ident) => {
        unsafe impl $crate::reg_proxy::Reg for $ty {
            type Target = $target;

            fn get() -> *const Self::Target {
                unsafe { &(*<$peripheral>::ptr()).$field as *const _ }
            }
        }
    };
}

// example:
// reg!(AHBCLKCTRL0, AHBCLKCTRL0, raw::SYSCON, ahbclkctrl0);

// unsafe impl crate::reg_proxy::Reg for AHBCLKCTRL0 {
//     type Target = AHBCLKCTRL0;
//     fn get() -> *const Self::Target {
//         unsafe { &(SYSCON::ptr()).ahbclkctrl0 as *const _ }
//     }
// }

// HOWEVER: without patches, the <alternateGroup> tag for AHBCLKCTRL0
// and AHBCLKCTRLX0 (which share the same address) causes `svd2rust`
// to be confused (or whatever, have not figured out), and `.ahbclkctrl0`
// is not generated, instead we get `._reserved_12_ahbclkctrl0`.
//
// unsafe impl crate::reg_proxy::Reg for AHBCLKCTRL0 {
//     type Target = AHBCLKCTRL0;
//     fn get() -> *const Self::Target {
//         unsafe { &(*<raw::SYSCON>::ptr())._reserved_12_ahbclkctrl0 as *const _ }
//     }
// }
//
// Not sure what the proper solution would be.
// There is https://github.com/rust-embedded/svd2rust/issues/97
// We just delete the *X? registers in the PAC.

