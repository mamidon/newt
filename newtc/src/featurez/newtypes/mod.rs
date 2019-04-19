use std::mem::size_of;

// Inspired by Rowan
pub unsafe trait TransparentNewType: Sized {
	type Inner;
	
	fn from_inner(inner: &Self::Inner) -> &Self {
		assert_eq!(size_of::<Self>(), size_of::<Self::Inner>());
		
		unsafe {
			let inner_ptr = inner as *const Self::Inner;
			let outer_ptr = inner_ptr as *const Self;
			let outer_ref = &*outer_ptr;
			
			return outer_ref;
		}
	}
	
	fn to_inner(&self) -> &Self::Inner {
		assert_eq!(size_of::<Self>(), size_of::<Self::Inner>());
		
		unsafe {
			let outer_ptr = self as *const Self;
			let inner_ptr = outer_ptr as *const Self::Inner;
			let inner_ref = &*inner_ptr;
			
			return inner_ref;
		}
	}
}