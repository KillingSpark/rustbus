//! Marshal trait and implementations for the basic types
use crate::wire::marshal::MarshalContext;

mod base;
mod container;
pub use base::*;
pub use container::*;

/// The Marshal trait allows to push any type onto an message_builder::OutMessage as a parameter.
/// There are some useful implementations here for slices and hashmaps which map to arrays and dicts in the dbus message.
///
/// The way dbus structs are represented is with rust tuples. This lib provides Marshal impls for tuples with up to 5 elements.
/// If you need more you can just copy the impl and extend it to how many different entries you need.
///
/// There is a crate (rustbus_derive) for deriving Marshal impls with #[derive(rustbus_derive::Marshal)]. This should work for most of your needs.
/// You can of course derive Signature as well.
///
/// If there are special needs, you can implement Marshal for your own structs:
/// ```rust
/// struct MyStruct {
///     x: u64,
///     y: String,
/// }
///
/// use rustbus::ByteOrder;
/// use rustbus::signature;
/// use rustbus::wire::util;
/// use rustbus::Marshal;
/// use rustbus::wire::marshal::MarshalContext;
/// use rustbus::wire::marshal::traits::SignatureBuffer;
/// use rustbus::Signature;
/// impl Signature for &MyStruct {
///     fn signature() -> signature::Type {
///         signature::Type::Container(signature::Container::Struct(signature::StructTypes::new(vec![
///             u64::signature(),
///             String::signature(),
///         ]).unwrap()))
///     }
///
///     fn alignment() -> usize {
///         8
///     }
///     fn sig_str(s_buf: &mut SignatureBuffer) {
///         s_buf.push_static("(ts)");
///     }
///     fn has_sig(sig: &str) -> bool {
///         sig == "(ts)"
///     }
/// }    
/// impl Marshal for &MyStruct {
///     fn marshal(
///         &self,
///         ctx: &mut MarshalContext,
///     ) -> Result<(), rustbus::Error> {
///         // always align to 8 at the start of a struct!
///         ctx.align_to(8);
///         self.x.marshal(ctx)?;
///         self.y.marshal(ctx)?;
///         Ok(())
///     }
/// }
/// ```
/// # Implementing for your own structs
/// There are some rules you need to follow, or the messages will be malformed:
/// 1. Structs need to be aligned to 8 bytes. Use `ctx.align_to(8);` to do that. If your type is marshalled as a primitive type
///     you still need to align to that types alignment.
/// 1. If you write your own dict type, you need to align every key-value pair at 8 bytes like a struct
/// 1. The signature needs to be correct, or the message will be malformed
/// 1. The alignment must report the correct number. This does not need to be a constant like in the example, but it needs to be consistent with the type
///     the signature() function returns. If you are not sure, just use Self::signature().get_alignment().
pub trait Marshal: Signature {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error>;
    fn marshal_as_variant(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        let mut sig = SignatureBuffer::new();
        Self::sig_str(&mut sig);
        if sig.len() > 255 {
            let sig_err = crate::signature::Error::SignatureTooLong;
            return Err(sig_err.into());
        }
        debug_assert!(crate::params::validation::validate_signature(&sig).is_ok());
        crate::wire::util::write_signature(&sig, ctx.buf);
        self.marshal(ctx)
    }
}

/// `SignatureBuffer` is used to store static or dynamic signatures and avoid allocations if possible.
/// It is a wrapper around Cow.
#[derive(Debug)]
pub struct SignatureBuffer(Cow<'static, str>);

impl SignatureBuffer {
    #[inline]
    pub fn new() -> Self {
        Self(Cow::Borrowed(""))
    }
    /// Pushes a `&str` into the signature buffer.
    ///
    /// Avoids an allocation if the `self` was empty and was not allocated already,
    /// by storing the `&'static str` inside a `Cow::Borrowed` variant.
    #[inline]
    pub fn push_static(&mut self, sig: &'static str) {
        match &mut self.0 {
            Cow::Borrowed("") => self.0 = Cow::Borrowed(sig),
            Cow::Owned(s) if s.capacity() == 0 => self.0 = Cow::Borrowed(sig),
            cow => cow.to_mut().push_str(sig),
        }
    }

    /// Pushes a `&str` into the signature buffer.
    ///
    /// If `sig` has a `'static` lifetime then [`SignatureBuffer::push_static`] should almost always be used
    /// instead of this, because it can provide a performance benefit by avoiding allocation.
    #[inline]
    pub fn push_str(&mut self, sig: &str) {
        self.0.to_mut().push_str(sig);
    }

    /// Return a `&mut String` which can be used to modify the signature.
    ///
    /// Internally this is just a call to `Cow::to_mut`.
    #[inline]
    pub fn to_string_mut(&mut self) -> &mut String {
        self.0.to_mut()
    }

    /// Clear the signature.
    ///
    /// If an allocation was already made it is retained for future use.
    /// If you wish to deallocate when clearing, then simply use [`SignatureBuffer::new`].
    #[inline]
    pub fn clear(&mut self) {
        match &mut self.0 {
            Cow::Borrowed(_) => *self = Self::new(),
            Cow::Owned(s) => s.clear(),
        }
    }
    #[inline]
    pub fn from_string(sig: String) -> Self {
        Self(Cow::Owned(sig))
    }
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn truncate(&mut self, new_len: usize) -> Result<(), crate::params::validation::Error> {
        if new_len > 0 {
            crate::params::validate_signature(&self.0.as_ref()[0..new_len])?;
        }
        self.0.to_mut().truncate(new_len);
        Ok(())
    }
}
impl std::ops::Deref for SignatureBuffer {
    type Target = str;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}
impl AsRef<str> for SignatureBuffer {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl Default for SignatureBuffer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

use std::borrow::Cow;
pub trait Signature {
    fn signature() -> crate::signature::Type;
    fn alignment() -> usize;
    /// If this returns `true`,
    /// it indicates that for implementing type `T`,
    /// Rust's `[T]` is identical to DBus's array format
    /// and can be copied into a message after aligning the first element.
    ///
    /// The default implementation is `false` but can be overridden for a performance gain
    /// if it is valid.
    ///
    /// # Safety
    /// Calls to this function should always be safe. Implementors should respect this property.
    /// The reason this method is `unsafe` is to indicate to people implementing `Signature` that
    /// overriding it has the potential to induce unsound behavior
    /// if the following rules aren't followed:
    /// 1. The type `T` implementing `Signature` must be `Copy`.
    /// 2. The size of `T` must be **equivalent** to it's DBus alignment (see [here]).
    /// 3. Every possible bit-pattern must represent a valid instance of `T`.
    ///    For example `std::num::NonZeroU32` does not meet this requirement `0` is invalid.
    /// 4. The type should not contain an Fd receieved from the message.
    ///    When implementing `Unmarshal` the type should only dependent the `'buf` lifetime.
    ///    It should never require the use of `'fds`.
    ///
    /// # Notes
    /// * This method exists because of limitiation with Rust type system.
    ///   Should `#[feature(specialization)]` ever become stablized this will hopefully be unnecessary.
    /// * This method should use the `ByteOrder` to check if it matches native order before returning `true`.
    ///   `ByteOrder::NATIVE` can be used to detect the native order.
    ///
    /// [here]: https://dbus.freedesktop.org/doc/dbus-specification.html#idm702
    #[inline]
    unsafe fn valid_slice(_bo: crate::ByteOrder) -> bool {
        false
    }
    /// Appends the signature of the type to the `SignatureBuffer`.
    ///
    /// By using `SignatureBuffer`, implementations of this method can avoid unnecessary allocations
    /// by only allocating if a signature is dynamic.
    ///
    /// The default implementation of `sig_str` can be pretty slow.
    /// If type, that `Signature` is being implemented for, has a static (unchanging) signature
    /// then overriding this method can have a significant performance benefit when marshal/unmarshalling
    /// the type inside variants.
    fn sig_str(s_buf: &mut SignatureBuffer) {
        let s_buf = s_buf.to_string_mut();
        let typ = Self::signature();
        typ.to_str(s_buf);
    }

    /// Check if this type fulfills this signature. This may expect to only be called with valid signatures.
    /// But it might be called with the wrong signature. This means for example you must check the length before indexing.
    ///
    /// The default impl uses Signature::sig_str and compares it to the given signature. The same performance
    /// implications as for Signature::sig_str apply here.
    fn has_sig(sig: &str) -> bool {
        let mut s_buf = SignatureBuffer::new();
        Self::sig_str(&mut s_buf);
        sig == s_buf.as_str()
    }
}

impl<S: Signature> Signature for &S {
    fn signature() -> crate::signature::Type {
        S::signature()
    }
    fn alignment() -> usize {
        S::alignment()
    }
    fn has_sig(sig: &str) -> bool {
        S::has_sig(sig)
    }
}

impl<P: Marshal> Marshal for &P {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        (*self).marshal(ctx)
    }
}

#[cfg(test)]
mod test {
    use crate::wire::marshal::MarshalContext;
    use crate::wire::ObjectPath;
    use crate::wire::SignatureWrapper;

    #[test]
    fn test_trait_signature_creation() {
        let mut msg = crate::message_builder::MarshalledMessage::new();
        let body = &mut msg.body;

        body.push_param("a").unwrap();
        body.push_param(ObjectPath::new("/a/b").unwrap()).unwrap();
        body.push_param(SignatureWrapper::new("(a{su})").unwrap())
            .unwrap();

        let fd = crate::wire::UnixFd::new(nix::unistd::dup(1).unwrap());
        body.push_param(&fd).unwrap();
        body.push_param(true).unwrap();
        body.push_param(0u8).unwrap();
        body.push_param(0u16).unwrap();
        body.push_param(0u32).unwrap();
        body.push_param(0u64).unwrap();
        body.push_param(0i16).unwrap();
        body.push_param(0i32).unwrap();
        body.push_param(0i64).unwrap();
        body.push_param(&[0u8][..]).unwrap();

        let map: std::collections::HashMap<String, (u64, u32, u16, u8)> =
            std::collections::HashMap::new();
        body.push_param(&map).unwrap();

        assert_eq!("soghbyqutnixaya{s(tuqy)}", msg.get_sig());
    }

    #[test]
    fn test_empty_array_padding() {
        use crate::wire::marshal::container::marshal_container_param;

        let mut msg = crate::message_builder::MarshalledMessage::new();
        let body = &mut msg.body;
        let empty = vec![0u64; 0];
        body.push_param(&empty[..]).unwrap();

        let empty = crate::params::Container::make_array_with_sig(
            crate::signature::Type::Base(crate::signature::Base::Uint64),
            empty.into_iter(),
        )
        .unwrap();

        let mut fds = Vec::new();
        let mut buf = Vec::new();
        let mut ctx = MarshalContext {
            fds: &mut fds,
            buf: &mut buf,
            byteorder: crate::ByteOrder::LittleEndian,
        };

        marshal_container_param(&empty, &mut ctx).unwrap();

        // 0 length and padded to 8 bytes even if there are no elements
        assert_eq!(msg.get_buf(), &[0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(buf.as_slice(), &[0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_variant_marshalling() {
        let mut msg = crate::message_builder::MarshalledMessage::new();
        let body = &mut msg.body;

        let original = (100u64, "ABCD", true);
        body.push_variant(original).unwrap();

        assert_eq!(
            msg.get_buf(),
            // signature ++ padding ++ u64 ++ string ++ padding ++ boolean
            &[
                5, b'(', b't', b's', b'b', b')', b'\0', 0, 100, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0,
                b'A', b'B', b'C', b'D', b'\0', 0, 0, 0, 1, 0, 0, 0
            ]
        )
    }
}
