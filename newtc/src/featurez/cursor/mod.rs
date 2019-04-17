
pub trait Cursor: Iterator {
	fn consumed(&self) -> usize;
	fn current(&self) -> Option<Self::Item>;
}

pub struct CursorIterator<I> where
	I: Iterator,
	<I as std::iter::Iterator>::Item: Copy
{
	iter: I,
	consumed: usize,
	current: Option<I::Item>
}


impl<I: Iterator> Cursor for CursorIterator<I> where
	<I as std::iter::Iterator>::Item: Copy
{
	fn consumed(&self) -> usize {
		self.consumed
	}

	fn current(&self) -> Option<<Self as Iterator>::Item> { self.current }
}

impl<I: Iterator> Iterator for CursorIterator<I> where
	<I as std::iter::Iterator>::Item: Copy
{
	type Item = I::Item;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		self.current = self.iter.next();
		
		match self.current {
			Some(item) => {
				self.consumed += 1;

				Some(item)	
			},
			None => None
		}
	}
}
use std::iter::Peekable;
impl<I: Iterator> CursorIterator<Peekable<I>> where
	<I as std::iter::Iterator>::Item: Copy
{
	pub fn from_iter(iter: I) -> CursorIterator<I> where
		I::Item: Copy
	{
		CursorIterator {
			iter,
			consumed: 0,
			current: None
		}
	}
	
	pub fn peek(&mut self) -> Option<&I::Item> {
		self.iter.peek()
	}
}

#[cfg(test)]
mod tests {
	use super::{CursorIterator, Cursor};
	
	#[test]
	fn cursor_still_iterable() {
		let mut iter = vec![1,2,3]
			.into_iter()
			.peekable();
		
		let mut cursor = CursorIterator::from_iter(iter);
		
		assert_eq!(cursor.next(), Some(1));
		assert_eq!(cursor.next(), Some(2));
		assert_eq!(cursor.next(), Some(3));
		assert_eq!(cursor.next(), None);
	}

	#[test]
	fn cursor_maintains_current_item() {
		let mut iter = vec![1,2,3]
			.into_iter()
			.peekable();

		let mut cursor = CursorIterator::from_iter(iter);
		
		assert_eq!(cursor.current(), None);
		
		cursor.next();
		assert_eq!(cursor.current(), Some(1));
		cursor.next();
		assert_eq!(cursor.current(), Some(2));
		cursor.next();
		assert_eq!(cursor.current(), Some(3));

		cursor.next();
		assert_eq!(cursor.current(), None);
	}

	#[test]
	fn cursor_maintains_next_item() {
		let mut iter = vec![1,2,3]
			.into_iter()
			.peekable();

		let mut cursor = CursorIterator::from_iter(iter);
		
		assert_eq!(cursor.peek(), Some(&1));

		cursor.next();
		assert_eq!(cursor.peek(), Some(&2));
		cursor.next();
		assert_eq!(cursor.peek(), Some(&3));
		cursor.next();
		assert_eq!(cursor.peek(), None);

		cursor.next();
		assert_eq!(cursor.peek(), None);
	}

	#[test]
	fn cursor_maintains_correct_count_of_consumed_items() {
		let mut iter = vec![1,2,3]
			.into_iter()
			.peekable();

		let mut cursor = CursorIterator::from_iter(iter);
		
		assert_eq!(cursor.consumed(), 0);

		cursor.next();
		assert_eq!(cursor.consumed(), 1);
		cursor.next();
		assert_eq!(cursor.consumed(), 2);
		cursor.next();
		assert_eq!(cursor.consumed(), 3);
		
		cursor.next();
		assert_eq!(cursor.consumed(), 3);
	}
}
