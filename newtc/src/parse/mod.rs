use std::str::Chars;


pub mod tokens;


pub struct Cursor<'a> {
	text: &'a str,
	consumed: usize,
}

impl<'a> Cursor<'a> {
	fn new(text: &'a str) -> Cursor {
		Cursor {
			text,
			consumed: 0,
		}
	}

	fn current(&self) -> Option<char> {
		self.chars().next()
	}

	fn nth(&self, n: usize) -> Option<char> {
		self.chars().nth(n)
	}

	fn consume(&mut self) -> Option<char> {
		let next = self.current()?;
		self.consumed += 1;
		Some(next)
	}

	fn matches(&self, candidate: char) -> bool {
		self.current() == Some(candidate)
	}

	fn matches_predicate<P: Fn(char) -> bool>(&self, predicate: P) -> bool {
		self.current().map(predicate).unwrap_or(false)
	}
	
	fn empty(&self) -> bool {
		self.current().is_none()
	}

	fn chars(&self) -> Chars {
		self.text[self.consumed..].chars()
	}
}


/*
There is the concept of a call tree in addition to the usual call stack.
Calling some entry point with the same props would yield the came call tree, 
but for the following additional points:

Each function in the call tree (aka component) can store internal state via hooks.
Each element yielded by the call tree can have event handlers attached to events (e.g. onClick).
These events will either be handled within the call tree (e.g. modifying props or state), do nothing,
or be published to a view model.

View models are bags of properties which are mutated asynchronously -- and as far as newt is 
concerned atomically. These view models can optionally receive certain events.
	
*/

/*
The types:
	primitives: i(8,16,32,64), u(8,16,32,64), f32, f64, glyph, string
	[string is a list of glyphs, a glyph is a unicode glyph, individual bytes are u8]

	complex: struct (and anonymous tuples), enums, tree, list, map
    
The operations:
	match expression
		match expression { option1 => ..., option2 => ..., // must be exhaustive }
	null checks
		no actual null values, optionality is indicated with ? (e.g. int8?) and can only be accessed
		within proper checks. e.g. (let foo: int? = 42; if foo { not null! } else { null... })
*/

/*
Some thoughts:
	no globals, possibly later implement global const expressions (but how powerful can resolver be?)
	avoid excessive syntax -- no one *wants* to learn this, so provide maximal value for minimal cost
	
*/

/*
// start off with simple strong+dynamically typed language
// making the types static later & adding proper annotations + possibly generics
fn main() {
	return (
		<Window height=400 width=400>
			<Span>Hello, World!</Span>
		</Window>
	);
}
*/

/*

type HelloWorldProps struct {
	Lines [string]
}

type HelloWorldEvent enum {
	CounterIncremented,
	Quit // can nest remainder of struct declaration inline
}


fn main() tree { 
	let model HelloWorldProps = useViewModel('MainModel', HelloWorldProps { 
		Lines = []
	});
	let channel fn(HelloWorldEvent) = useChannel<HelloWorldEvent>('MainChannel');
	let children [tree] = useChildren();
	let lines [tree] = [];
	
	for line in model.Lines {
		lines.push((<Span>{line}</Span>));
	}
	
	return (
		<Window height=400 width=400>
			<Button text="more!" onClick={(event ClickEvent) => channel.publish(HelloWorldEvent.CounterIncremented)} />
			<Button text="quit?" onClick={(event ClickEvent) => channel.publish(HelloWorldEvent.Quit)} />
			{lines}
		</Window>
	);
}

*/