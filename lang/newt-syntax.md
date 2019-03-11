# Newt

I'll start with a dynamically typed language -- static typing can come later.  
Everything will be either a primitive (u64,i64,f64,string,glyph) or a hash table.

Although the intent is to have a JSX like sub-syntax, we'll start off using only
hash tables.

## Functions
Functions are declared via the fn keyword, followed by an argument list, followed by the body.
Closures are supported.

```
fn MyFunc(argument1, argument2) {
    if !argument1 {
        return {}; // empty table
    }

    if !argument2 {
        return {}; // empty table
    }

    let inner = (arg) {
        return arg + 1;
    }

    return { foo: "bar", bar: inner(2) };
}
```

## Globals

Besides function definitions, no globals are supported at first.  Possibly in the future we'll support read only globals/statics.
This is because I don't want any potential race conditions.

## Locals

All local variables are lexically scoped, as you would expect.

## Data Types

Primitives: i64, f64, char, and string
Complex: arrays and hash maps

```
fn Foo() {
    let signedInt = 32;
    let float = 3.14f;
    let char = 'a';
    let string = "foo";

    let array = [0,1,2,3];
    let map = { foo: "bar", test: 42 };

    print(map.foo);
    print(map["foo"]);
}
```

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


## JSX

At first we won't directly support <..> syntax -- you'll have to build up your component hierarchies manually.
Special properties are:

* tag
* props
* props.children
* props.value ? -- possibly for elements which host as their only child some primitive type e.g. text boxes

```
fn MyComponent() {
    
    let props = {
        children: [
            { tag: MyOtherComponent, props: { count: 1 }},
            { tag: MyOtherComponent, props: { count: 2 }},
        ]
     };

    return { tag: label, props: props };
}

fn MyOtherComponent(props) {
    return { tag: label, props: { text: props.count }};
}
```


## Grammar

Program
	FunctionStatement*
	
FunctionStatement
	'fn' Identifier '(' Identifier? (',' Identifier)* ')' StatementBlock
	
StatementBlock
	'{' Statement* '}'
	
Statement
	LetStatement
	| FunctionStatement
	| IfElseStatement
	| WhileStatement
	| ForInStatement
	| ExpressionStatement
	
LetStatement
	'let' Identifier '=' ExpressionStatement
	
ExpressionStatement
	Expression ';'
	
Expression
	MathExpression
	
MathExpression
	AddExpression
	
AddExpression
	MultiplicationExpression
	| '+' Expression
	| '-' Expression
	
MultiplicationExpression
	UnaryExpression
	| '*' Expression
	| '/' Expression
	
UnaryExpression
	PrimaryExpression
	| '-' Expression
	| '!' Expression
	
PrimaryExpression
	FunctionCallExpression
	| LiteralExpression
	
LiteralExpression
	IntegerLiteralExpression
	| FloatLiteralExpression
	| StringLiteralExpression
	| GlyphLiteralExpression