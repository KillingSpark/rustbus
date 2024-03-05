//! This contains the implementations for the `Marshal` trait for container types like lists and dicts

use crate::signature::SignatureIter;
use crate::wire::errors::MarshalError;
use crate::wire::marshal::traits::SignatureBuffer;
use crate::wire::marshal::MarshalContext;
use crate::Marshal;
use crate::Signature;

impl<E: Signature> Signature for (E,) {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(
            crate::signature::StructTypes::new(vec![E::signature()]).unwrap(),
        ))
    }
    fn alignment() -> usize {
        8
    }
    fn has_sig(sig: &str) -> bool {
        let Some(sig) = sig.strip_prefix('(') else {
            return false;
        };
        let Some(sig) = sig.strip_suffix(')') else {
            return false;
        };
        let mut iter = SignatureIter::new(sig);
        let Some(s) = iter.next() else { return false };
        iter.next().is_none() && E::has_sig(s)
    }
}
impl<E: Marshal> Marshal for (E,) {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
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
    fn has_sig(sig: &str) -> bool {
        let Some(sig) = sig.strip_prefix('(') else {
            return false;
        };
        let Some(sig) = sig.strip_suffix(')') else {
            return false;
        };
        let mut iter = SignatureIter::new(sig);
        let Some(s1) = iter.next() else { return false };
        let Some(s2) = iter.next() else { return false };
        iter.next().is_none() && E1::has_sig(s1) && E2::has_sig(s2)
    }
}
impl<E1: Marshal, E2: Marshal> Marshal for (E1, E2) {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
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
    fn has_sig(sig: &str) -> bool {
        let Some(sig) = sig.strip_prefix('(') else {
            return false;
        };
        let Some(sig) = sig.strip_suffix(')') else {
            return false;
        };
        let mut iter = SignatureIter::new(sig);
        let Some(s1) = iter.next() else { return false };
        let Some(s2) = iter.next() else { return false };
        let Some(s3) = iter.next() else { return false };
        iter.next().is_none() && E1::has_sig(s1) && E2::has_sig(s2) && E3::has_sig(s3)
    }
}
impl<E1: Marshal, E2: Marshal, E3: Marshal> Marshal for (E1, E2, E3) {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
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
    fn has_sig(sig: &str) -> bool {
        let Some(sig) = sig.strip_prefix('(') else {
            return false;
        };
        let Some(sig) = sig.strip_suffix(')') else {
            return false;
        };
        let mut iter = SignatureIter::new(sig);
        let Some(s1) = iter.next() else { return false };
        let Some(s2) = iter.next() else { return false };
        let Some(s3) = iter.next() else { return false };
        let Some(s4) = iter.next() else { return false };
        iter.next().is_none()
            && E1::has_sig(s1)
            && E2::has_sig(s2)
            && E3::has_sig(s3)
            && E4::has_sig(s4)
    }
}
impl<E1: Marshal, E2: Marshal, E3: Marshal, E4: Marshal> Marshal for (E1, E2, E3, E4) {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
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
    fn has_sig(sig: &str) -> bool {
        let Some(sig) = sig.strip_prefix('(') else {
            return false;
        };
        let Some(sig) = sig.strip_suffix(')') else {
            return false;
        };
        let mut iter = SignatureIter::new(sig);
        let Some(s1) = iter.next() else { return false };
        let Some(s2) = iter.next() else { return false };
        let Some(s3) = iter.next() else { return false };
        let Some(s4) = iter.next() else { return false };
        let Some(s5) = iter.next() else { return false };
        iter.next().is_none()
            && E1::has_sig(s1)
            && E2::has_sig(s2)
            && E3::has_sig(s3)
            && E4::has_sig(s4)
            && E5::has_sig(s5)
    }
}
impl<E1: Marshal, E2: Marshal, E3: Marshal, E4: Marshal, E5: Marshal> Marshal
    for (E1, E2, E3, E4, E5)
{
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
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
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
        <&[E] as Marshal>::marshal(&self.as_slice(), ctx)
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
    fn has_sig(sig: &str) -> bool {
        if let Some(_prefix) = sig.strip_prefix('a') {
            let mut iter = SignatureIter::new(&sig[1..]);
            E::has_sig(iter.next().unwrap())
        } else {
            false
        }
    }
}
impl<E: Marshal> Marshal for [E] {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
        <&[E] as Marshal>::marshal(&self, ctx)
    }
}

impl<E: Signature, const N: usize> Signature for [E; N] {
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
    fn has_sig(sig: &str) -> bool {
        <[E]>::has_sig(sig)
    }
}
impl<E: Marshal, const N: usize> Marshal for [E; N] {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
        <&[E] as Marshal>::marshal(&self.as_slice(), ctx)
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
    fn has_sig(sig: &str) -> bool {
        <[E]>::has_sig(sig)
    }
}
use crate::wire::util::write_u32;
impl<E: Marshal> Marshal for &[E] {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
        // always align to 4
        ctx.align_to(4);
        let alignment = E::alignment();
        unsafe {
            if E::valid_slice(ctx.byteorder) {
                debug_assert_eq!(alignment, std::mem::size_of::<E>());
                let len = alignment * self.len();
                assert!(len <= u32::MAX as usize);
                write_u32(len as u32, ctx.byteorder, ctx.buf);
                ctx.align_to(alignment);
                let ptr = self.as_ptr().cast::<u8>();
                let slice = std::slice::from_raw_parts(ptr, len);
                ctx.buf.extend_from_slice(slice);
                return Ok(());
            }
        }

        let size_pos = ctx.buf.len();
        ctx.buf.extend_from_slice(&[0; 4]);

        ctx.align_to(alignment);

        if self.is_empty() {
            return Ok(());
        }

        // In an array each entry, except the last  will take up at least its alignment in space.
        // The last may take less (like type '(yy)') but this is small and worth it.
        ctx.buf.reserve(self.len() * alignment);
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

pub struct Variant<T: Marshal + Signature>(pub T);

impl<T: Marshal + Signature> Signature for Variant<T> {
    #[inline]
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Variant)
    }
    #[inline]
    fn alignment() -> usize {
        1
    }
    #[inline]
    fn sig_str(s_buf: &mut SignatureBuffer) {
        s_buf.push_static("v")
    }
    fn has_sig(sig: &str) -> bool {
        sig.starts_with('v')
    }
}

impl<T: Marshal + Signature> Marshal for Variant<T> {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
        self.0.marshal_as_variant(ctx)
    }
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
    fn sig_str(s_buf: &mut SignatureBuffer) {
        s_buf.push_str("a{");
        K::sig_str(s_buf);
        V::sig_str(s_buf);
        s_buf.push_str("}");
    }
    fn has_sig(sig: &str) -> bool {
        if sig.starts_with("a{") {
            let mut iter = SignatureIter::new(&sig[2..sig.len() - 1]);
            K::has_sig(iter.next().unwrap()) && V::has_sig(iter.next().unwrap())
        } else {
            false
        }
    }
}

impl<K: Marshal, V: Marshal> Marshal for std::collections::HashMap<K, V> {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), MarshalError> {
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
