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
    idx: usize,
    sigs: &'a str,
}

impl<'a> SignatureIter<'a> {
    /// This does not validate the content, it expects a valid signature.
    pub fn new(sigs: &'a str) -> SignatureIter<'a> {
        SignatureIter { idx: 0, sigs }
    }

    /// This does not validate the content, it expects a valid signature.
    pub fn new_at_idx(sigs: &'a str, idx: usize) -> SignatureIter<'a> {
        SignatureIter { idx, sigs }
    }
}

impl<'a> Iterator for SignatureIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.sigs.len() {
            return None;
        }
        let mut end_pos = self.idx;
        let mut open_brackets = 0;
        loop {
            let current = self.sigs.chars().nth(end_pos).unwrap();
            end_pos += 1;

            if current == '(' || current == '{' {
                open_brackets += 1;
            } else if current == ')' || current == '}' {
                open_brackets -= 1;
            }
            if current == 'a' {
                continue;
            }
            if open_brackets == 0 {
                break;
            }
        }
        let sig = &self.sigs[self.idx..end_pos];
        self.idx += sig.len();
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
