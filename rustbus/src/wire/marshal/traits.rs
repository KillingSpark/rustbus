//! Marshal trait and implementations for the basic types

use crate::params;
use crate::wire::marshal::base::marshal_base_param;
use crate::wire::marshal::MarshalContext;

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
}

impl<S: Signature> Signature for &S {
    fn signature() -> crate::signature::Type {
        S::signature()
    }
    fn alignment() -> usize {
        S::alignment()
    }
}

impl<P: Marshal> Marshal for &P {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        (*self).marshal(ctx)
    }
}

impl<E: Signature> Signature for (E,) {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(
            crate::signature::StructTypes::new(vec![E::signature()]).unwrap(),
        ))
    }

    fn alignment() -> usize {
        8
    }
}
impl<E: Marshal> Marshal for (E,) {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        // always align to 8
        ctx.align_to(8);
        self.0.marshal(ctx)?;
        Ok(())
    }
}

impl<E1: Signature, E2: Signature> Signature for (E1, E2) {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(
            crate::signature::StructTypes::new(vec![E1::signature(), E2::signature()]).unwrap(),
        ))
    }
    fn alignment() -> usize {
        8
    }
    fn sig_str(s_buf: &mut SignatureBuffer) {
        s_buf.push_str("(");
        E1::sig_str(s_buf);
        E2::sig_str(s_buf);
        s_buf.push_str(")");
    }
}
impl<E1: Marshal, E2: Marshal> Marshal for (E1, E2) {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        // always align to 8
        ctx.align_to(8);
        self.0.marshal(ctx)?;
        self.1.marshal(ctx)?;
        Ok(())
    }
}

impl<E1: Signature, E2: Signature, E3: Signature> Signature for (E1, E2, E3) {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(
            crate::signature::StructTypes::new(vec![
                E1::signature(),
                E2::signature(),
                E3::signature(),
            ])
            .unwrap(),
        ))
    }
    fn alignment() -> usize {
        8
    }

    fn sig_str(s_buf: &mut SignatureBuffer) {
        s_buf.push_str("(");
        E1::sig_str(s_buf);
        E2::sig_str(s_buf);
        E3::sig_str(s_buf);
        s_buf.push_str(")");
    }
}
impl<E1: Marshal, E2: Marshal, E3: Marshal> Marshal for (E1, E2, E3) {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        // always align to 8
        ctx.align_to(8);
        self.0.marshal(ctx)?;
        self.1.marshal(ctx)?;
        self.2.marshal(ctx)?;
        Ok(())
    }
}

impl<E1: Signature, E2: Signature, E3: Signature, E4: Signature> Signature for (E1, E2, E3, E4) {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(
            crate::signature::StructTypes::new(vec![
                E1::signature(),
                E2::signature(),
                E3::signature(),
                E4::signature(),
            ])
            .unwrap(),
        ))
    }
    fn alignment() -> usize {
        8
    }
    fn sig_str(s_buf: &mut SignatureBuffer) {
        s_buf.push_str("(");
        E1::sig_str(s_buf);
        E2::sig_str(s_buf);
        E3::sig_str(s_buf);
        E4::sig_str(s_buf);
        s_buf.push_str(")");
    }
}
impl<E1: Marshal, E2: Marshal, E3: Marshal, E4: Marshal> Marshal for (E1, E2, E3, E4) {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        // always align to 8
        ctx.align_to(8);
        self.0.marshal(ctx)?;
        self.1.marshal(ctx)?;
        self.2.marshal(ctx)?;
        self.3.marshal(ctx)?;
        Ok(())
    }
}

impl<E1: Signature, E2: Signature, E3: Signature, E4: Signature, E5: Signature> Signature
    for (E1, E2, E3, E4, E5)
{
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(
            crate::signature::StructTypes::new(vec![
                E1::signature(),
                E2::signature(),
                E3::signature(),
                E4::signature(),
                E5::signature(),
            ])
            .unwrap(),
        ))
    }
    fn alignment() -> usize {
        8
    }
    fn sig_str(s_buf: &mut SignatureBuffer) {
        s_buf.push_str("(");
        E1::sig_str(s_buf);
        E2::sig_str(s_buf);
        E3::sig_str(s_buf);
        E4::sig_str(s_buf);
        E5::sig_str(s_buf);
        s_buf.push_str(")");
    }
}
impl<E1: Marshal, E2: Marshal, E3: Marshal, E4: Marshal, E5: Marshal> Marshal
    for (E1, E2, E3, E4, E5)
{
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        // always align to 8
        ctx.align_to(8);
        self.0.marshal(ctx)?;
        self.1.marshal(ctx)?;
        self.2.marshal(ctx)?;
        self.3.marshal(ctx)?;
        self.4.marshal(ctx)?;
        Ok(())
    }
}

impl<E: Marshal> Marshal for Vec<E> {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        self.as_slice().marshal(ctx)
    }
}

impl<E: Signature> Signature for [E] {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Array(Box::new(
            E::signature(),
        )))
    }
    #[inline]
    fn alignment() -> usize {
        4
    }
    fn sig_str(s_buf: &mut SignatureBuffer) {
        s_buf.push_str("a");
        E::sig_str(s_buf);
    }
}
impl<E: Marshal> Marshal for [E] {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        (&self).marshal(ctx)
    }
}

impl<E: Signature> Signature for &[E] {
    #[inline]
    fn signature() -> crate::signature::Type {
        <[E]>::signature()
    }
    #[inline]
    fn alignment() -> usize {
        <[E]>::alignment()
    }
    #[inline]
    fn sig_str(s_buf: &mut SignatureBuffer) {
        <[E]>::sig_str(s_buf)
    }
}
impl<E: Marshal> Marshal for &[E] {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        // always align to 4
        ctx.align_to(4);

        let size_pos = ctx.buf.len();
        ctx.buf.push(0);
        ctx.buf.push(0);
        ctx.buf.push(0);
        ctx.buf.push(0);

        ctx.align_to(E::alignment());

        if self.is_empty() {
            return Ok(());
        }

        // we can reserve at least one byte per entry without wasting memory, and save at least a few reallocations of buf
        ctx.buf.reserve(self.len());
        let size_before = ctx.buf.len();
        for p in self.iter() {
            p.marshal(ctx)?;
        }
        let size_of_content = ctx.buf.len() - size_before;
        crate::wire::util::insert_u32(
            ctx.byteorder,
            size_of_content as u32,
            &mut ctx.buf[size_pos..size_pos + 4],
        );

        Ok(())
    }
}

/// **_!!! This assumes that you are marshalling to the platforms byteorder !!!_**
///
/// It just memcpy's the content of the array into the message. This is fine for all integer types, but you cannot use it for structs,
/// even if they are copy!
/// They might have padding to be correctly aligned in the slice. I would recommend to only use this for marshalling
/// big integer arrays but I cannot express this in the type system cleanly so here is a comment.
pub struct OptimizedMarshal<'a, E: Copy + Marshal>(pub &'a [E]);
impl<'a, E: Copy + Marshal> Signature for OptimizedMarshal<'a, E> {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Array(Box::new(
            E::signature(),
        )))
    }

    fn alignment() -> usize {
        4
    }
}
impl<'a, E: Copy + Marshal> Marshal for OptimizedMarshal<'a, E> {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        // always align to 4
        ctx.align_to(4);

        let size_pos = ctx.buf.len();
        ctx.buf.push(0);
        ctx.buf.push(0);
        ctx.buf.push(0);
        ctx.buf.push(0);

        ctx.align_to(E::alignment());

        if self.0.is_empty() {
            return Ok(());
        }

        let size_of_content = std::mem::size_of::<E>() * self.0.len();
        let len_before_resize = ctx.buf.len();
        ctx.buf.resize(ctx.buf.len() + size_of_content, 0);
        unsafe {
            let content_ptr = ctx.buf.as_mut_ptr().add(len_before_resize);
            std::ptr::copy_nonoverlapping(
                self.0.as_ptr() as *const u8,
                content_ptr,
                size_of_content,
            );
        }
        crate::wire::util::insert_u32(
            ctx.byteorder,
            size_of_content as u32,
            &mut ctx.buf[size_pos..size_pos + 4],
        );

        Ok(())
    }
}

#[test]
fn verify_optimized_arrays() {
    use crate::wire::marshal::container::marshal_container_param;
    use crate::ByteOrder;
    let mut fds = Vec::new();
    let mut buf = Vec::new();
    let mut ctx = MarshalContext {
        buf: &mut buf,
        fds: &mut fds,
        byteorder: ByteOrder::LittleEndian,
    };
    let ctx = &mut ctx;

    // marshal array of u64 optimized and non-optimized and compare
    let arru64: Vec<u64> = vec![1, 2, 3, 4, 5, 6, u64::MAX, u64::MAX / 2, u64::MAX / 1024];
    let mut buf_normal = Vec::new();
    ctx.buf = &mut buf_normal;
    arru64.as_slice().marshal(ctx).unwrap();

    let mut buf_optimized = Vec::new();
    ctx.buf = &mut buf_optimized;
    OptimizedMarshal(arru64.as_slice()).marshal(ctx).unwrap();
    assert_eq!(buf_normal, buf_optimized);

    let mut ctx = MarshalContext {
        buf: &mut buf,
        fds: &mut fds,
        byteorder: ByteOrder::LittleEndian,
    };
    let ctx = &mut ctx;

    // marshal array of u8 optimized and non-optimized and compare
    let mut buf_normal = Vec::new();
    ctx.buf = &mut buf_normal;
    let arru8: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 255, 128, 80, 180];
    arru8.as_slice().marshal(ctx).unwrap();

    let mut buf_optimized = Vec::new();
    ctx.buf = &mut buf_optimized;
    OptimizedMarshal(arru8.as_slice()).marshal(ctx).unwrap();
    assert_eq!(buf_normal, buf_optimized);

    let mut ctx = MarshalContext {
        buf: &mut buf,
        fds: &mut fds,
        byteorder: ByteOrder::LittleEndian,
    };
    let ctx = &mut ctx;

    // check that empty arrays work as expected
    let empty: Vec<u8> = Vec::new();
    let x = crate::params::Container::make_array("y", empty.clone().into_iter()).unwrap();

    let mut buf_normal = Vec::new();
    ctx.buf = &mut buf_normal;
    empty.as_slice().marshal(ctx).unwrap();

    let mut buf_optimized = Vec::new();
    ctx.buf = &mut buf_optimized;
    OptimizedMarshal(empty.as_slice()).marshal(ctx).unwrap();

    let mut buf_old = Vec::new();
    ctx.buf = &mut buf_old;
    marshal_container_param(&x, ctx).unwrap();
    assert_eq!(buf_normal, buf_optimized);
    assert_eq!(buf_normal, buf_old);
    assert_eq!(vec![0, 0, 0, 0], buf_old);
}

impl<K: Signature, V: Signature> Signature for std::collections::HashMap<K, V> {
    fn signature() -> crate::signature::Type {
        let ks = K::signature();
        let vs = V::signature();
        if let crate::signature::Type::Base(ks) = ks {
            crate::signature::Type::Container(crate::signature::Container::Dict(ks, Box::new(vs)))
        } else {
            panic!("Ivalid key sig")
        }
    }

    fn alignment() -> usize {
        4
    }
}

impl<K: Marshal, V: Marshal> Marshal for std::collections::HashMap<K, V> {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        // always align to 4
        ctx.align_to(4);

        let size_pos = ctx.buf.len();
        ctx.buf.push(0);
        ctx.buf.push(0);
        ctx.buf.push(0);
        ctx.buf.push(0);

        // always align to 8
        ctx.align_to(8);

        if self.is_empty() {
            return Ok(());
        }

        let size_before = ctx.buf.len();
        for p in self.iter() {
            // always align to 8
            ctx.align_to(8);
            p.0.marshal(ctx)?;
            p.1.marshal(ctx)?;
        }
        let size_of_content = ctx.buf.len() - size_before;
        crate::wire::util::insert_u32(
            ctx.byteorder,
            size_of_content as u32,
            &mut ctx.buf[size_pos..size_pos + 4],
        );

        Ok(())
    }
}

impl Signature for u64 {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Uint64)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
    #[inline]
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("t");
    }
}
impl Marshal for u64 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        let b: params::Base = self.into();
        marshal_base_param(&b, ctx)
    }
}

impl Signature for i64 {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Int64)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
    #[inline]
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("x");
    }
}
impl Marshal for i64 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        let b: params::Base = self.into();
        marshal_base_param(&b, ctx)
    }
}

impl Signature for u32 {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Uint32)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
    #[inline]
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("u");
    }
}
impl Marshal for u32 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        let b: params::Base = self.into();
        marshal_base_param(&b, ctx)
    }
}

impl Signature for i32 {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Int32)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
    #[inline]
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("i");
    }
}
impl Marshal for i32 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        let b: params::Base = self.into();
        marshal_base_param(&b, ctx)
    }
}

impl Signature for u16 {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Uint16)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
    #[inline]
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("q");
    }
}
impl Marshal for u16 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        let b: params::Base = self.into();
        marshal_base_param(&b, ctx)
    }
}

impl Signature for i16 {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Int16)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
    #[inline]
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("n");
    }
}
impl Marshal for i16 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        let b: params::Base = self.into();
        marshal_base_param(&b, ctx)
    }
}

impl Signature for u8 {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Byte)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
    #[inline]
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("y");
    }
}
impl Marshal for u8 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        let b: params::Base = self.into();
        marshal_base_param(&b, ctx)
    }
}

impl Signature for bool {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Boolean)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
    #[inline]
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("b");
    }
}
impl Marshal for bool {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        let b: params::Base = self.into();
        marshal_base_param(&b, ctx)
    }
}

impl Signature for String {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::String)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
    #[inline]
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("s");
    }
}
impl Marshal for String {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        crate::wire::util::write_string(self.as_str(), ctx.byteorder, ctx.buf);
        Ok(())
    }
}

impl Signature for &str {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::String)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
    #[inline]
    fn sig_str(sig: &mut SignatureBuffer) {
        String::sig_str(sig);
    }
}
impl Marshal for &str {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        crate::wire::util::write_string(self, ctx.byteorder, ctx.buf);
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct ObjectPath<S: AsRef<str>>(S);
impl<S: AsRef<str>> ObjectPath<S> {
    pub fn new(path: S) -> Result<Self, crate::params::validation::Error> {
        crate::params::validate_object_path(path.as_ref())?;
        Ok(ObjectPath(path))
    }
    pub fn to_owned(&self) -> ObjectPath<String> {
        ObjectPath(self.as_ref().to_owned())
    }
}
impl<S: AsRef<str>> AsRef<str> for ObjectPath<S> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
impl<S: AsRef<str>> Signature for ObjectPath<S> {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::ObjectPath)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}
impl<S: AsRef<str>> Marshal for ObjectPath<S> {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        self.0.as_ref().marshal(ctx)
    }
}
#[derive(Debug, PartialEq)]
pub struct SignatureWrapper<'a>(&'a str);
impl<'a> SignatureWrapper<'a> {
    pub fn new(sig: &'a str) -> Result<Self, crate::params::validation::Error> {
        crate::params::validate_signature(sig)?;
        Ok(SignatureWrapper(sig))
    }
}
impl<'a> AsRef<str> for SignatureWrapper<'a> {
    fn as_ref(&self) -> &str {
        self.0
    }
}
impl Signature for SignatureWrapper<'_> {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Signature)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
    #[inline]
    fn sig_str(s_buf: &mut SignatureBuffer) {
        s_buf.push_static("g");
    }
}
impl Marshal for SignatureWrapper<'_> {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        crate::wire::util::write_signature(self.0, ctx.buf);
        Ok(())
    }
}

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
            5, b'(', b't', b's', b'b', b')', b'\0', 0, 100, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, b'A',
            b'B', b'C', b'D', b'\0', 0, 0, 0, 1, 0, 0, 0
        ]
    )
}
