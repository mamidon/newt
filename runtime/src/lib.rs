pub struct Cursor<'a, T>
where
    T: Copy + PartialEq,
{
    slice: &'a [T]
}

impl<'a, T> Cursor<'a, T>
where
    T: Copy + PartialEq,
{
    pub fn create(slice: &'a [T]) -> Cursor<'a, T>
    {
        Cursor {
            slice,
        }
    }

    /// Unconditionally consume the next item.
    pub fn consume(&mut self) -> Option<T> {
        let next = self.slice.get(0).copied();

        if next.is_some() {
            self.slice = &self.slice[1..];
        }

        next
    }

    // Consume the next item if it matches the expected value.
    pub fn consume_if(&mut self, expected: T) -> bool {
        if let Some(&current) = self.slice.get(0) {
            current == expected && self.consume().is_some()
        } else {
            false
        }
    }

    // Consume the next item if it is contained by the provided slice.
    pub fn consume_in(&mut self, expected: &[T]) -> Option<T> {
        self.slice.get(0)
            .filter(|&i| expected.contains(i))
            .and_then(|_| self.consume())
    }

    pub fn peek(&self, n: usize) -> Option<T> {
        self.slice.get(n).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::Cursor;

    #[test]
    fn consume_always_advances() {
        let v = vec![1,2,3];
        let mut c = Cursor::create(v.as_slice());
        
        assert_eq!(Some(1), c.consume());
        assert_eq!(Some(2), c.consume());
        assert_eq!(Some(3), c.consume());
        assert_eq!(None, c.consume());
    }

    #[test]
    fn consume_returns_none_at_sequence_end() {
        let v = vec![1,2,3];
        let mut c = Cursor::create(v.as_slice());
        
        assert_eq!(Some(1), c.consume());
        assert_eq!(Some(2), c.consume());
        assert_eq!(Some(3), c.consume());
        assert_eq!(None, c.consume());
    }

    #[test]
    fn consume_if_advances_only_on_match() {
        let v = vec![1,2,3];
        let mut c = Cursor::create(v.as_slice());

        assert!(c.consume_if(1));
        assert_eq!(false, c.consume_if(1));
        assert!(c.consume_if(2));
        assert_eq!(false, c.consume_if(2));
    }

    #[test]
    fn consume_if_returns_none_at_sequence_end() {
        let v = vec![1,2,3];
        let mut c = Cursor::create(v.as_slice());

        assert!(c.consume_if(1));
        assert!(c.consume_if(2));
        assert!(c.consume_if(3));
        assert_eq!(false, c.consume_if(3));
        assert_eq!(false, c.consume_if(3));
    }

    #[test]
    fn consume_in_advances_only_on_match() {
        let v = vec![1,2,3];
        let mut c = Cursor::create(v.as_slice());

        assert_eq!(None, c.consume_in(&[2]));
        c.consume();
        assert_eq!(Some(2), c.consume_in(&[2, 3]));
        assert_eq!(Some(3), c.consume_in(&[2, 3]));
    }

    #[test]
    fn consume_in_returns_none_at_sequence_end() {
        let v = vec![1,2,3];
        let mut c = Cursor::create(v.as_slice());

        c.consume();
        c.consume();
        assert_eq!(Some(3), c.consume_in(&[3]));
        assert_eq!(None, c.consume_in(&[3]));
        assert_eq!(None, c.consume_in(&[3]));
    }
}
