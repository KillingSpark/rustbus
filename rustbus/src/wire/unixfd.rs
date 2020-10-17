use crate::wire::marshal::MarshalContext;
use crate::wire::unmarshal::UnmarshalContext;
use crate::{Marshal, Signature, Unmarshal};
use std::os::unix::io::RawFd;

use std::cell::Cell;
use std::rc::Rc;
#[derive(Debug)]
struct UnixFdInner {
    inner: Cell<Option<RawFd>>,
}
impl Drop for UnixFdInner {
    fn drop(&mut self) {
        if let Some(fd) = self.inner.take() {
            nix::unistd::close(fd).ok();
        }
    }
}

#[derive(Clone, Debug)]
pub struct UnixFd(Rc<UnixFdInner>);
impl UnixFd {
    pub fn new(fd: RawFd) -> Self {
        UnixFd(Rc::new(UnixFdInner {
            inner: Cell::new(Some(fd)),
        }))
    }
    /// Gets a non-owning `RawFd`. If `None` is returned.
    /// then this UnixFd has already been taken by somebody else
    /// and is no longer valid.
    pub fn get_raw_fd(&self) -> Option<RawFd> {
        self.0.inner.get()
    }
    /// Gets a owning `RawFd` from the UnixFd.
    /// Subsequent attempt to get the `RawFd` from
    /// other `UnixFd` referencing the same file descriptor will
    /// fail.
    pub fn take_raw_fd(self) -> Option<RawFd> {
        self.0.inner.take()
    }
}
/// Allow for the comparison of `UnixFd` even after the `RawFd`
/// has been taken, to see if they originally referred to the same thing.
impl PartialEq<UnixFd> for UnixFd {
    fn eq(&self, other: &UnixFd) -> bool {
        Rc::ptr_eq(&self.0, &other.0) || self.get_raw_fd() == other.get_raw_fd()
    }
}

impl Signature for UnixFd {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::UnixFd)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}
impl Marshal for UnixFd {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        // TODO Maybe a dup() is the correct thing to do here. currently this allows to take the fd
        // out of the UnixFd somewhere else before the message has been sent, which is bad
        if let Some(fd) = self.get_raw_fd() {
            let new_fd = nix::unistd::dup(fd)
                .map_err(|e| crate::Error::Marshal(crate::wire::marshal::Error::DupUnixFd(e)))?;
            ctx.fds.push(UnixFd::new(new_fd));

            let idx = ctx.fds.len() - 1;
            ctx.align_to(Self::alignment());
            crate::wire::util::write_u32(idx as u32, ctx.byteorder, ctx.buf);
            Ok(())
        } else {
            Err(crate::Error::Marshal(
                crate::wire::marshal::Error::EmptyUnixFd,
            ))
        }
    }
}
impl Signature for &dyn std::os::unix::io::AsRawFd {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::UnixFd)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}
impl Marshal for &dyn std::os::unix::io::AsRawFd {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), crate::Error> {
        let fd = self.as_raw_fd();
        let new_fd = nix::unistd::dup(fd)
            .map_err(|e| crate::Error::Marshal(crate::wire::marshal::Error::DupUnixFd(e)))?;
        ctx.fds.push(UnixFd::new(new_fd));

        let idx = ctx.fds.len() - 1;
        ctx.align_to(Self::alignment());
        crate::wire::util::write_u32(idx as u32, ctx.byteorder, ctx.buf);
        Ok(())
    }
}

impl<'r, 'buf: 'r, 'fds> Unmarshal<'r, 'buf, 'fds> for UnixFd {
    fn unmarshal(
        ctx: &mut UnmarshalContext<'fds, 'buf>,
    ) -> crate::wire::unmarshal::UnmarshalResult<Self> {
        let (bytes, idx) = u32::unmarshal(ctx)?;

        if ctx.fds.len() <= idx as usize {
            Err(crate::wire::unmarshal::Error::BadFdIndex(idx as usize))
        } else {
            let val = &ctx.fds[idx as usize];
            Ok((bytes, val.clone()))
        }
    }
}
