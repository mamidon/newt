# Newt


## State

Everything is an object, and all objects are part of a class.  Classes are objects too.
All classes describe what their state looks like and how much of it is available for public
read/write operations.

All functions are implemented in terms of what types they operate on, with a special case
being functions associated with a specific class.  Those functions can read/write private data.


```
struct FizzBuzz; // empty class definitions are allowed.

struct Foo {
			Bar: 	u32,
	imm		Fizz: 	i32, // cannot be changed after initialization
	get		Buzz: 	i32, // public read 
	set 	Flop: 	i32, // public write
	get set	Zap: 	i32
}
```

Classes can be defined in terms of other types.

```
struct LinkedList[T] {
	get	Value: 	T,
		Next:	LinkedList[T]
}

struct Foo[T: Animal + Mammal]; // T must implement the Animal & Mammal interfaces
```
## Behavior

Behavior can be described in terms of classes or interfaces.  Every class can implement
multiple interfaces.  All functions related to interfaces are implicitly public.

For functions which are not static, their first parameter must be an 'untyped' `self`.
This value is a reference to the instance of that class.  Static methods can omit this
parameter.

```
interface Animal { 
	fn MakeSound(self); 
}

struct Dog; 
impl Animal for Dog {
	fn MakeSound(self) {
		Console.WriteLine("Bark!");
	}
}

struct Cat;
impl Animal for Cat {
	fn MakeSound(self) {
		Console.WriteLine("Meow!");
	}
}
```

Functions can be implemented in terms of type constraints.

```
impl SomeInterface for SomeClass[T: Mammal] {
	/* Implement the SomeInterface interface if T also implements the Mammal interface */
	/* T may also have constraints defined on the struct definition */
} 

impl SomeClass[T] {
	pub fn DoSomething[Z](self, value: Z) { 
		/* You can call this function with any type Z */ 
	}

	pub fn DoSomethingElse[Z: Mammal](self, value: Z) {
		/* You can call this function with any type Z that also implements the Mammal interface */
		/* This isn't really any different than accepting a Mammal directly */
	}

	pub fn DoEverythingElse[Z: Mammal + Animal](self, value: Z) {
		/* This syntax might be uneccessary, perhaps make type constraints able to be defined inline */
	}
}
```

## Modules

Modules will be the unit of transfer between environments.  A module must request a list
of functions it will need provided.  It can also list a set of functions it exports.

But if I do not make any special affordances, to print to the console you'll have to pass
the console function all over the place.  I need some notion of importing a list of classes
which are implicitly available throughout the module, just like built in classes. 

For now I'll just pass around instances.

## Globals

## Locals

## Data Types

Primitives: i64, f64, char, and string
Complex: arrays and hash maps


## Control flow

The usual -- if .. else, while, for .. in, 

```
fn Foo() {
	 if 4 == 2 + 2 {
		  print("true");
	 } else {
		  print("false");
	 }

	 let i = 0;
	 while i < 3 {
		  i = i + 1;
		  print(str(i));
	 }

	 for x in range(3) {
		  print(str(x));
	 }

	 let map = { a: "foo", b: "bar", c: "whatever" };

	 for key in keys(map) {
		  print(key); 
	 }

	 for value in values(map) {
		  print(value);
	 }
}
```

the output of this would be

```
true
1
2
3
1
2
3
a
b
c
foo
bar
whatever
```

## Operators

Math
* `+`
* `-`
* `/`
* `*`
* `( .. )`

Logic
* `&&`
* `||`
* `!`

Comparison
* `==`
* `<`
* `>`
* `<=`
* `>=`

Assignment
* `=`

## Modules

## Environments


## Grammar