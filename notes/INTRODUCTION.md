
```rust
use std:io:print

fn main() {
	print("yooooo was geht")
}
```

Mo is a fast, statically typed, compiled programming language.

Special focuses are syntactic minimalism, readability and execution speed.

Easy to learn, easy to write!

You can think of Mo as a modern C without all the quirks, like a Go without a huge runtime and garbage collector, like an Odin without the weird syntax choices (proc? what are you talking about, we are not in 1970 >:( )

Main inspirations for the language are Zig, Rust, Odin, Jai and Go. 

The compiler is written in Rust.

# Project status

Mo is still in very early development - in the draft stage you could say - ideas and contributions are welcome though.

This is a long-term hobby project that could eventually evolve into a full production-grade language. Maybe. If not it's a fun intellectual challenge for my brain and something I can be proud of.

# Philosophy

Mo is supposed to be a pretty dumb language. That means the compiler won't do much magic. There are no classes, inheritance or complex type system. It's mostly just functions and data.

A Mo program just exists of collections of functions and data types. The program starts at the main function.

There will be pointers/references, monomorphized generics and basic traits. 

Mo still aims to be a modern language with good tooling like a simple to use CLI, integrated build system and a friendly compiler with great error messages.

# Learn

I want to have an online book in the future explaining the language in more detail. But as the language isn't refined and implemented yet, this document will do for now as an overview to learn about its planned features and get a feel for what it would be like to use.

### Variables

Variables are defined like this:
```
my_variable type_of_variable = value
```
You can also infer the type by replacing it with an underscore:
```
my_variable _ = value
```
Variables are updated like you'd expect:
```
my_variable = new_value
```

### Note on whitespace, line breaks and semicolons
Most whitespace is insignificant except for line breaks. Every line break, that is LF `\n` or CRLF `\r\n`, can be replaced with a semicolon `;` and vice versa. So in most cases you don't need to use semicolons, but you can use them in cases where you want to put stuff on the same line.

### Struct
You can define structs very simply, as you would expect
```go
struct Cat {
	name String
	food_level u32
}
```
You can initialize a struct like this:
```
cat Cat = zero
cat.name = "Matz"
```
`zero` is a special keyword that represents a zeroed value of the type. So `zero` is equivalent to `Cat.{ name = ""; food_level = 0 }`. or just a row of zero bytes for the size of the type.

Another way to initialize a struct is to use the shorthand:
```rust
cat _ = Cat.{ name = "Matz"; food_level = 0 }
```
If you specify the type instead of inferring it you can omit the type from the shorthand:
```rust
cat Cat = .{ name = "Matz"; food_level = 0 }
```
As stated above you can use either `;` or line breaks interchangeably:
```rust
cat Cat = .{
	name = "Matz"
	food_level = 0
}
```

!NOTE TO SELF: not sure yet if I want to necessitate semicolons or linebreaks between member initializations. maybe commas could also be used interchangeably with line breaks here. But I don't want to allow commas at the end of statements so that would necessitate special handling and could cause confusion. I like having rules that are general and easy to remember and applicable consistently.

### Functions
```rust
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

### Methods

If a function has a pointer as its first argument to a type, it can be called as a method on that type:
```
// Our method "feed"
fn feed(cat &Cat) {
	cat.food_level += 10
}

fn main() {
	cat _ = zero // creating a new zeroed cat "object" on the stack
	cat.name = "Matz"
	
	cat.feed()
	// equivalent to:
	feed(&cat)
}
```
Functions can be overloaded:
```
fn feed(programmer &Human) {
	programmer.food_level += 10
}

fn main() {
	cat Cat = zero
	cat.name = "Matz"
	
	programmer Human = zero
	programmer.name = "Moritz"
		
	cat.feed()
	programmer.feed()
}

```

### Traits
Traits are basically just contracts you make with the compiler. You say "I implemented functions from this trait for this type" and the compiler ensures that's actually true and then gives you the ability to pass in that type to functions that expect a certain trait implementation.

#### Example:
```
trait Feedable[T] {
	fn feed(hungy &T)
}

impl Feedable[Cat] // writing this, the compiler enforces the "feed" function to be implemented with in this case T being replaced with the Cat type.

// The program now only compiles if this function is given
fn feed(cat &Cat) {
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
struct ConsList[T] {
	value T
	next_list &ConsList[T]
}

/// appends item to list
fn append[T](list &ConsList[T], item T) {
	// -- implementation --
}

fn init[T](list &ConsList[T]) {
	// -- implementation --
}
```
You could create an instance like this:
```
cats List[Cat] = undef
cats.init() // init() allocates memory for the list and initializes it to zero
```
The compiler just creates a separate variant of ConsList and its methods (in this case append and init) with Cat being replaced for the generic type T (monomorphization). 

Then you can use it like this
```rust
matz Cat = zero
matz.name = "Matz"

cats.append(matz)
```
You can also call a generic function with an explicit type with the `:[]`-syntax like. I call it "lil fish".
```
append:[Cat](&cats, matz)
```

Maybe, a desirable feature could be that the generic type could be inferred from the arguments and the correct generic method would be chosen at automatically at compile time. But I'm not sure if that's a good idea yet.
```
append[_](&cats, matz)
```
#### Generic Traits
There can be trait bounds on generic functions. All this does is require the type to have all the functions of a specific trait implemented.

```rust
trait Ord[T] {
  fn cmp[T](first &T, second &T) Ordering
}

fn max[T impl PartialOrd](a T, b T) T {
    ret if a > b { a } else { b }
}

struct NumWrapper { num i32 }
impl Ord[NumWrapper]
fn cmp[T](first &T, second &T) Ordering {
	ret first.num.cmp(&second.num)
}

fn main() {
	num1 NumWrapper = zero; num2 NumWrapper = zero
	num1.num = 1
	num2.num = 2
	
	print(max(&num1, &num2).num) // 2
}
```

### Arrays
Arrays are just a fixed-size list of values of the same type. Unlike in other languages, the array type does not have specific accessor syntax like `[]` in C, Java, Rust, Python etc. Instead there are built-in functions for accessing array elements like `get` and `set`. This makes the syntax more consistent and allows for a simpler language grammar. 

You can create an array like this:
```
array Array[i32] = arrays:new[i32](10) // probably a compiler intrinsic function
```

The type should be able to be inferred from the context, so you don't need to specify it explicitly: 
```
array _ = arrays:new[i32](10)
```
or create it with the special array initializer syntax like this:
```
array _ = Array[i32].{ 1, 2, 3 } // array with fixed length of 3
```
If the type is specified in the variable declaration, you can omit the type in the initializer syntax like this:
```
array Array[i32] = .{ 1, 2, 3 }
```
Then index into the array using `[]` syntax:
```
print(array[0]) // 1
array[1] = 42
print(array[1]) // 42
```

### Slices
Slices are a view into an array, allowing you to access a subset of the array's elements without copying them. They are created using the `slice` function:
```
slice Slice[i32] = arrays:slice[i32](array, 0, 3) // slice of the first 3 elements
```
You can also create a slice by using the `[]` syntax on an array:
```
slice Slice[i32] = array[0..3] // slice of the first 3 elements
```

### Pointers / References
Pointers are references to memory addresses, and are used to access values indirectly. In Molang, pointers are typed, so you must specify the type of the value they are pointing to. You can create a pointer by using the `&` operator on a value:
```
cat Cat = { name: "Algundo", food_level: 3 }
ptr &Cat = &cat
```
To read the value of a pointer, use the postfix `.*` operator:
```
print(ptr.*.name) // "Algundo"
```

Pointers can be cast to a number using the cast function:
```
addr u64 = cast:[u64](ptr)
```



### Memory management

