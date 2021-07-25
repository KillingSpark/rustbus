pub struct SignatureIter<'a> {
    idx: usize,
    sigs: &'a str,
}

impl<'a> SignatureIter<'a> {
    pub fn new(sigs: &'a str) -> SignatureIter<'a> {
        SignatureIter { sigs, idx: 0 }
    }

    pub fn new_at_idx(sigs: &'a str, idx: usize) -> SignatureIter<'a> {
        SignatureIter { sigs, idx }
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