//! This contains the implementations for the `Marshal` trait for container types like lists and dicts

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
use crate::wire::util::write_u32;
impl<E: Marshal> Marshal for &[E] {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
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
                let ptr = *self as *const [E] as *const u8;
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

pub struct Variant<T: Marshal + Signature>(T);

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
}

impl<T: Marshal + Signature> Marshal for Variant<T> {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
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
