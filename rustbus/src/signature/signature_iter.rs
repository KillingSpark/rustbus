/// Implements an iterator over signatures contained in a &str.
/// This does not validate the content, it expects a valid signature.
/// ```rust
/// use rustbus::signature::SignatureIter;
/// let mut iter = SignatureIter::new("s(x)a(xxy)a{s(st)}");
/// assert_eq!(iter.next(), Some("s"));
/// assert_eq!(iter.next(), Some("(x)"));
/// assert_eq!(iter.next(), Some("a(xxy)"));
/// assert_eq!(iter.next(), Some("a{s(st)}"));
/// ```
pub struct SignatureIter<'a> {
    sigs: &'a str,
}

impl<'a> SignatureIter<'a> {
    /// This does not validate the content, it expects a valid signature.
    pub fn new(sigs: &'a str) -> SignatureIter<'a> {
        SignatureIter { sigs }
    }

    /// This does not validate the content, it expects a valid signature.
    pub fn new_at_idx(sigs: &'a str, idx: usize) -> SignatureIter<'a> {
        if idx >= sigs.len() {
            Self::new("")
        } else {
            SignatureIter {
                sigs: sigs.split_at(idx).1,
            }
        }
    }
}

impl<'a> Iterator for SignatureIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        if self.sigs.is_empty() {
            return None;
        }
        let mut end_pos = 0;
        let mut open_brackets = 0;
        loop {
            // A valid siganture consists of only ascii characters and
            // `.chars().nth(end_pos)` takes linear time.
            let current = *self.sigs.as_bytes().get(end_pos).unwrap();
            end_pos += 1;

            match current {
                b'(' | b'{' => open_brackets += 1,
                b')' | b'}' => open_brackets -= 1,
                b'a' => continue,
                _ => (),
            }

            if open_brackets == 0 {
                break;
            }
        }
        let (sig, rest) = self.sigs.split_at(end_pos);
        self.sigs = rest;
        Some(sig)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn signature_iterator() {
        let mut iter = super::SignatureIter::new("(aas)a{s(b(xt))}ssss(((((x)))))");
        assert_eq!(iter.next(), Some("(aas)"));
        assert_eq!(iter.next(), Some("a{s(b(xt))}"));
        assert_eq!(iter.next(), Some("s"));
        assert_eq!(iter.next(), Some("s"));
        assert_eq!(iter.next(), Some("s"));
        assert_eq!(iter.next(), Some("s"));
        assert_eq!(iter.next(), Some("(((((x)))))"));
        assert_eq!(iter.next(), None);
    }
}
