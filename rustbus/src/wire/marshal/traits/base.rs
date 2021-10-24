//! This contains the implementations for the `Marshal` trait for base types like integers and strings

use crate::wire::errors::MarshalError;
use crate::wire::marshal::traits::SignatureBuffer;
use crate::wire::marshal::MarshalContext;
use crate::wire::util;
use crate::wire::ObjectPath;
use crate::wire::SignatureWrapper;
use crate::Marshal;
use crate::Signature;

impl Signature for u64 {
    #[inline]
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Uint64)
    }
    #[inline]
    fn alignment() -> usize {
        8
    }
    #[inline]
    unsafe fn valid_slice(bo: crate::ByteOrder) -> bool {
        bo == crate::ByteOrder::NATIVE
    }
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("t");
    }
    fn has_sig(sig: &str) -> bool {
        sig.starts_with('t')
    }
}
impl Marshal for u64 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
        ctx.align_to(Self::alignment());
        util::write_u64(*self, ctx.byteorder, ctx.buf);
        Ok(())
    }
}

impl Signature for i64 {
    #[inline]
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Int64)
    }
    #[inline]
    fn alignment() -> usize {
        8
    }
    #[inline]
    unsafe fn valid_slice(bo: crate::ByteOrder) -> bool {
        bo == crate::ByteOrder::NATIVE
    }
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("x");
    }
    fn has_sig(sig: &str) -> bool {
        sig.starts_with('x')
    }
}
impl Marshal for i64 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
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
        4
    }
    #[inline]
    unsafe fn valid_slice(bo: crate::ByteOrder) -> bool {
        bo == crate::ByteOrder::NATIVE
    }
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("u");
    }
    fn has_sig(sig: &str) -> bool {
        sig.starts_with('u')
    }
}
impl Marshal for u32 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
        ctx.align_to(Self::alignment());
        crate::wire::util::write_u32(*self, ctx.byteorder, ctx.buf);
        Ok(())
    }
}

impl Signature for i32 {
    #[inline]
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Int32)
    }
    #[inline]
    fn alignment() -> usize {
        4
    }
    #[inline]
    unsafe fn valid_slice(bo: crate::ByteOrder) -> bool {
        bo == crate::ByteOrder::NATIVE
    }
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("i");
    }
    fn has_sig(sig: &str) -> bool {
        sig.starts_with('i')
    }
}
impl Marshal for i32 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
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
        2
    }
    #[inline]
    unsafe fn valid_slice(bo: crate::ByteOrder) -> bool {
        bo == crate::ByteOrder::NATIVE
    }
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("q");
    }
    fn has_sig(sig: &str) -> bool {
        sig.starts_with('q')
    }
}
impl Marshal for u16 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
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
        2
    }
    #[inline]
    unsafe fn valid_slice(bo: crate::ByteOrder) -> bool {
        bo == crate::ByteOrder::NATIVE
    }
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("n");
    }
    fn has_sig(sig: &str) -> bool {
        sig.starts_with('n')
    }
}
impl Marshal for i16 {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
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
        1
    }
    #[inline]
    unsafe fn valid_slice(_: crate::ByteOrder) -> bool {
        true
    }
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("y");
    }
    #[inline]
    fn has_sig(sig: &str) -> bool {
        sig.starts_with('y')
    }
}
impl Marshal for u8 {
    #[inline]
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
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
        4
    }
    #[inline]
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("b");
    }
    #[inline]
    fn has_sig(sig: &str) -> bool {
        sig.starts_with('b')
    }
}
impl Marshal for bool {
    #[inline]
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
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
        4
    }
    #[inline]
    fn sig_str(sig: &mut SignatureBuffer) {
        sig.push_static("s");
    }
    #[inline]
    fn has_sig(sig: &str) -> bool {
        sig.starts_with('s')
    }
}
impl Marshal for String {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
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
    #[inline]
    fn has_sig(sig: &str) -> bool {
        String::has_sig(sig)
    }
}
impl Marshal for &str {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
        ctx.align_to(Self::alignment());
        crate::wire::util::write_string(self, ctx.byteorder, ctx.buf);
        Ok(())
    }
}

impl<S: AsRef<str>> Signature for ObjectPath<S> {
    #[inline]
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::ObjectPath)
    }
    #[inline]
    fn alignment() -> usize {
        4
    }
    #[inline]
    fn sig_str(s_buf: &mut SignatureBuffer) {
        s_buf.push_static("o");
    }
    #[inline]
    fn has_sig(sig: &str) -> bool {
        sig.starts_with('o')
    }
}
impl<S: AsRef<str>> Marshal for ObjectPath<S> {
    #[inline]
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
        self.as_ref().marshal(ctx)
    }
}

impl<S: AsRef<str>> Signature for SignatureWrapper<S> {
    #[inline]
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Signature)
    }
    #[inline]
    fn alignment() -> usize {
        1
    }
    #[inline]
    fn sig_str(s_buf: &mut SignatureBuffer) {
        s_buf.push_static("g");
    }
    #[inline]
    fn has_sig(sig: &str) -> bool {
        sig.starts_with('g')
    }
}
impl<S: AsRef<str>> Marshal for SignatureWrapper<S> {
    #[inline]
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
        crate::wire::util::write_signature(self.as_ref(), ctx.buf);
        Ok(())
    }
}
