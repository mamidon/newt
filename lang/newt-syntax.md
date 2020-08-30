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

## Modules

At least at first, modules will do nothing other than make multi file code reasonable.  A module is declared with 
the module keyword, followed by a multi part identifier, followed by a scope which functions can be defined in.

Individual functions & globals in a module may be exported, and exports from other modules may be imported via keyword.
Note that the file pathnames do not need to match or have any relationship to the module pathnames.

Besides built in modules (e.g. 'System', 'Runtime' etc) which are implicitly available for import; 1st party modules
are found via a recursive directory search originating from the directory containing a newtc.config file.  Presumably
newtc was invoked pointed at that directory.

I won't add explicit support for third party modules right now -- package management is not a solved problem and deserves significant research.  For now it will be sufficient to either include 3rd party code side by side your code, or support extra root directories via 
newtc.config.

First.newt
```
module My.First.Module 
{
	 import My.Second.Module; // imports this module's exported members directly into namespace
	 import My.Third.Module as Third; // imports this module's exported members attached to the 'Third' table
	 
	 export FirstFirstFunction(left, right)
	 {
		  return Third.ThirdFunction(left + right);
	 }

	 export FirstSecondFunction(left, right) 
	 {
		  return SecondFunction(left + right);
	 }

	 FirstPrivateFunction()
	 {
		  // not visible to any modules importing My.First.Module
	 }
}
```

## Environments

I want a system which makes it easy for me to develop code locally, then publish it to intermediate (QA, Staging, etc) and final (Prod, end-user box) environments.
I also want to easily understand what's going on in these other environments.

Suppose we have the following module defined..
```
module EchoServer
{
	import IConfiguration as Configs;

	export Echo(message) 
	{
		return "Hello from " + Configs.SomeValue() + " your message is: " + message;
	}
}
```

In this case, IConfiguration is an interface -- there is no implementation available yet.
In order to actually create target environments (e.g. 'local', 'dev', 'prod'), we need syntax to represent & initialize a module like `EchoServer` 
as well as the ability to move such a module between physical processes.

We might define the 'local' & 'dev' environments like so...
```
module LocalConfiguration
{
	export SomeValue()
	{
		return "Local!";
	}
}

module LocalEnvironment
{
	// happens to conform to IConfiguration
	import LocalConfiguration as Configs;

	// weird syntax, which supplies LocalConfiguration as an implementation for IConfiguration
	// but otherwise works as the usual import
	import EchoServer { Configs: Configs } as LocalEchoServer;
}
```

But this doesn't really answer the question of how we map the lexical state of the `EchoServer` implementation to the runtime state of
these environments.  There's also questions of trust relationships & storing runtime state such as secret configuration values.

Perhaps we shouldn't try to have the lexical representation of an environment look so much like a module?
An environment is really:
1. A set of module (or interface) implementations which fill in unspecified dependencies of the `EchoServer`
2. An identity (public, private key?)
3. Trust relationships (a set of public keys for other environments or individuals)

## Grammar
```
Program
	ModuleStatement* 

ModuleStatement
	'module' ModuleIdentifier '{' GlobalDeclaration* '}'
	| GlobalDeclaration*

ModuleIdentifier
	Identifier('.' Identifier)*

GlobalDeclaration
	FunctionStatement

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

// Highest priority lower down 
Expression
	AddExpression
	| PropertyExpression
	| CallExpression
	
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
	| PropertyExpression
	| CallExpression
	| VariableExpression

LiteralExpression
	IntegerLiteralExpression
	| FloatLiteralExpression
	| StringLiteralExpression
	| GlyphLiteralExpression
	| TableLiteralExpression

IntegerLiteralExpression
	 [0-9]+

FloatLiteralExpression
	 [0-9]+ 'f' 
	 | [0-9]+ '.' [0-9]+ 'f'?

StringLiteralExpression
	 '"' .* '"'

GlyphLiteralExpression
	 ''' . '''

TableLiteralExpression
	 '{' (LiteralExpression ':' Expression)* '}'

VariableExpression
	 Identifier

ArgumentList
	 Expression ( ',' Expression )*

PropertyExpression
	 Expression '.' Identifier

CallExpression
	 VariableExpression
	 | Expression '(' ArgumentList ')'
```