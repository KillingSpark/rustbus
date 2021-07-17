//! This contains the implementations for the `Marshal` trait for base types like integers and strings

use crate::wire::marshal::traits::SignatureBuffer;
use crate::wire::marshal::MarshalContext;
use crate::wire::util;
use crate::Marshal;
use crate::Signature;

impl Signature for u64 {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Uint64)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
    #[inline]
    unsafe fn valid_slice(bo: crate::ByteOrder) -> bool {
        bo == crate::ByteOrder::NATIVE
    }
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("t");
    }
}
impl Marshal for u64 {
    #[inline]
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        util::write_u64(*self, ctx.byteorder, ctx.buf);
        Ok(())
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
    unsafe fn valid_slice(bo: crate::ByteOrder) -> bool {
        bo == crate::ByteOrder::NATIVE
    }
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("x");
    }
}
impl Marshal for i64 {
    #[inline]
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        // Ok because rust represents i64 as a twos complement, which is what dbus uses too
        util::write_u64(*self as u64, ctx.byteorder, ctx.buf);
        Ok(())
    }
}

impl Signature for u32 {
    #[inline]
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Uint32)
    }
    #[inline]
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
    #[inline]
    unsafe fn valid_slice(bo: crate::ByteOrder) -> bool {
        bo == crate::ByteOrder::NATIVE
    }
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("u");
    }
}
impl Marshal for u32 {
    #[inline]
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        crate::wire::util::write_u32(*self, ctx.byteorder, ctx.buf);
        Ok(())
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
    unsafe fn valid_slice(bo: crate::ByteOrder) -> bool {
        bo == crate::ByteOrder::NATIVE
    }
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("i");
    }
}
impl Marshal for i32 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        // Ok because rust represents i32 as a twos complement, which is what dbus uses too
        crate::wire::util::write_u32(*self as u32, ctx.byteorder, ctx.buf);
        Ok(())
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
    unsafe fn valid_slice(bo: crate::ByteOrder) -> bool {
        bo == crate::ByteOrder::NATIVE
    }
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("q");
    }
}
impl Marshal for u16 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        util::write_u16(*self, ctx.byteorder, ctx.buf);
        Ok(())
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
    unsafe fn valid_slice(bo: crate::ByteOrder) -> bool {
        bo == crate::ByteOrder::NATIVE
    }
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("n");
    }
}
impl Marshal for i16 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.align_to(Self::alignment());
        // Ok because rust represents i16 as a twos complement, which is what dbus uses too
        util::write_u16(*self as u16, ctx.byteorder, ctx.buf);
        Ok(())
    }
}

impl Signature for u8 {
    #[inline]
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Byte)
    }
    #[inline]
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
    #[inline]
    unsafe fn valid_slice(_: crate::ByteOrder) -> bool {
        true
    }
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("y");
    }
}
impl Marshal for u8 {
    #[inline]
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        ctx.buf.push(*self);
        Ok(())
    }
}

impl Signature for bool {
    #[inline]
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Boolean)
    }
    #[inline]
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
    #[inline]
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("b");
    }
}
impl Marshal for bool {
    #[inline]
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        (*self as u32).marshal(ctx)
    }
}

impl Signature for String {
    #[inline]
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::String)
    }
    #[inline]
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
        self.as_str().marshal(ctx)
    }
}

impl Signature for &str {
    #[inline]
    fn signature() -> crate::signature::Type {
        String::signature()
    }
    #[inline]
    fn alignment() -> usize {
        String::alignment()
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
