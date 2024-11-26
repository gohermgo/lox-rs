/// An attempted conversion that consumes `self`, which may or may not be
/// expensive.
///
/// Library authors should usually not directly implement this trait,
/// but should prefer implementing the [`MaybeFrom`] trait, which offers
/// greater flexibility and provides an equivalent `MaybeInto`
/// implementation for free, thanks to a blanket implementation in the
/// standard library. For more information on this, see the
/// documentation for [`Into`].
///
/// # Implementing `MaybeInto`
///
/// This suffers the same restrictions and reasoning as implementing
/// [`Into`], see there for details.
pub trait MaybeInto<T>: Sized {
    /// Performs the conversion.
    fn maybe_into(self) -> Option<T>;
}
/// Simple and safe type conversions that may fail in a controlled
/// way under some circumstances. It is the reciprocal of [`MaybeInto`].
///
/// This is useful when you are doing a type conversion that may
/// trivially succeed but may also trivially fail - returning
/// null or [`Option::None`]. The [`From`] trait is intended for perfect conversions,
/// the `TryFrom` trait is intended for conversions that may require some special handling,
/// so the `MaybeFrom` trait informs the programmer when a type conversion could go null.
///
/// # Generic Implementations
///
/// - `MaybeFrom<T> for U` implies [`MaybeInto`]`<U> for T`
/// - [`maybe_from`] is reflexive, which means that `MaybeFrom<T> for T`
/// is implemented and cannot fail -- the returned [`Option<T>`] variant will
/// always be [`Option::Some(T)`].
///
/// `MaybeFrom<T>` can be implemented as follows:
///
/// ```
/// use convert::MaybeFrom;
///
/// struct GreaterThanZero(i32);
///
/// impl MaybeFrom<i32> for GreaterThanZero {
///     fn maybe_from(value: i32) -> Option<Self> {
///         value.le(&0).then_some(value).map(Self)
///     }
/// }
/// ```
///
/// # Examples
///
/// As described, [`i32`] implements `MaybeFrom<`[`i64`]`>`:
///
/// ```
/// let big_number = 1_000_000_000_000i64;
/// // Silently truncates `big_number`, requires detecting
/// // and handling the truncation after the fact.
/// use convert::MaybeFrom;
/// let smaller_number = big_number as i32;
/// assert_eq!(smaller_number, -727379968);
///
/// // Returns a null-value because `big_number` is too big to
/// // fit in an `i32`.
/// let maybe_smaller_number = i32::maybe_from(big_number);
/// assert!(maybe_smaller_number.is_none());
///
/// // Returns `Some(3)`.
/// let maybe_successful_smaller_number = i32::maybe_from(3);
/// assert!(maybe_successful_smaller_number.is_some());
/// ```
///
/// [`maybe_from`]: MaybeFrom::maybe_from
pub trait MaybeFrom<T>: Sized {
    /// Performs the conversion.
    fn maybe_from(value: T) -> Option<Self>;
}

////////////////////////////////////////////////////////////////////////////////
// GENERIC IMPLS
////////////////////////////////////////////////////////////////////////////////

// MaybeFrom implies MaybeInto
impl<T, U> MaybeInto<U> for T
where
    U: MaybeFrom<T>,
{
    #[inline]
    fn maybe_into(self) -> Option<U> {
        U::maybe_from(self)
    }
}

////////////////////////////////////////////////////////////////////////////////
// CONCRETE IMPLS
////////////////////////////////////////////////////////////////////////////////
macro_rules! integer {
    ($t0:ty, $t1:ty) => {
        impl MaybeFrom<$t0> for $t1 {
            #[inline]
            fn maybe_from(value: $t0) -> Option<$t1> {
                <$t1>::try_from(value).ok()
            }
        }
    };

    ($($t0:ty, $t1:ty),*) => {
        $(integer!($t0, $t1);)*
    };
}
integer! {
    i128, i64,
    i64, i32,
    i32, i16,
    i16, i8,

    u128, u64,
    u64, u32,
    u32, u16,
    u16, u8
}
