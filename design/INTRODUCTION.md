
```
use std:io:print

fn main() {
	print("yooooo was geht")
}
```

Mo is a fast, statically typed, compiled programming language.

Special focuses are syntactic minimalism, readability and execution speed.

Easy to learn, easy to write. In 50 years this may be used as a low-level teaching language instead of C (if its done by then bruh). 

You can think of Mo as a modern C without all the quirks, like a Go without a huge runtime and garbage collector, like an Odin without the weird syntax choices (proc? wtf are you talking about)

Main inspirations for the language are Zig, Rust, Odin, Jai and Go. 

The compiler is written in Rust.

# Project status

Mo is still in early development, contributions are welcome.

This is a long-term hobby project that could eventually evolve into a full production-grade language. Maybe. If not it's a fun intellectual challenge for my brain and something I can be proud of.

# Learn

This will have a link to a book in the future explaining the language. But as the language isn't really refined and done yet, this will have to do for now.

# Vision

Mo is supposed to be pretty dumb as a language. That doesn't mean we won't have good error messages (that's actually a really important focus), but the compiler won't do that much. There is no classes, inheritance or complex type system. It's just functions and data. 

A Mo program just exists of collections of functions and data types. The program starts at the main function.

There will be pointers, monomorphized generics and very simple traits. 

### Struct
You can define structs very simply, as you would expect
```
struct Cat {
	name String
	food_level u32
}
```
### Functions
```
fn main() {
	// where everything starts
}

fn my_function() {
	do_stuff()
}

fn my_function_with_return_type() i32 {
	ret 10
}
```

Every function with a pointer as first argument to a type can be called with method syntax
```
fn feed(cat *Cat) {
	cat.food_level += 10
}

fn main() {
	cat := Cat // creating a new zeroed cat on the stack
	cat.name = "Matz"
	
	cat.feed()
	// equivalent to:
	feed(&cat)
}
```
Functions can be overloaded:
```
fn feed(programmer *Human) {
	programmer.food_level += 10
}

fn main() {
	cat := Cat
	cat.name = "Matz"
	
	programmer := Human
	programmer.name = "Moritz"
		
	cat.feed()
	programmer.feed()
}

```

### Traits
Traits are just contracts forced by the compiler for specific functions to be implemented in the scope.
```
trait Feedable[T] {
	fn feed(hungy *T)
}

impl Feedable[Cat] // writing this, the compiler enforces the "feed" function to be implemented with in this case T being replaced with the Cat type.

// The program now only compiles if this function is given
fn feed(cat *Cat) {
	cat.food_level += 10
}
```
You can also enforce multiple trait implementations in one line:
```
impl Feedable[Cat] Feedable[Human]
```

### Generics
You can easily define generic types like this:
```
struct List[T] {
    value T
    next_list *List[T]
}

/// appends item to list
fn append[T](list *List[T], item T) {
	// implementation
}
```
You create an instance like this:
```
cats := List[Cat]
cats.init() // there must be some kind of init function because every struct is zeroed at first
```
The compiler just creates a separate variant of List and its methods (in this case append) with Cat being replaced for the generic type T (monomorphization). 

Then you can use it like this
```
matz Cat := Cat
matz.name = "Matz"

cats.append(matz)
```
You can also call a generic function with an explicit type with the `:[]`-syntax like this:
```
append:[Cat](&cats, matz)
```

There can be trait bounds on generic functions. All this does is require the type to have all the functions of a specific trait implemented.

```
trait Ord[T] {
  fn cmp[T](first *T, second *T) Ordering
}

fn max[T impl PartialOrd](a T, b T) T {
    ret if a > b { a } else { b }
}

struct NumWrapper { num i32 }
impl Ord[NumWrapper]
fn cmp[T](first *T, second *T) Ordering {
	ret first.num.cmp(&second.num)
}

fn main() {
	num1 := NumWrapper; num2 := NumWrapper
	num1.num = 1
	num2.num = 2
	
	print(max(&num1, &num2).num) // 2
}
```


