This document is mostly just a collection of ideas and thoughts about the Mo programming language. Some of them are just brainstorming, others are more concrete proposals.

# Idea 2
Grouping symbols of item declarations into a module / scope that can be imported deputy for the specified symbols.

Example:

```
struct Player { ... }
fn heal(player &Player) { ... }

mod player { Player, heal(&Player) } // Note to dev: function args are just type expressions here to identify the correct function signature. 

We could have a language server action here that can add methods of structs to the module automatically. Or maybe have a designated syntax, but maybe that makes it too complicated. But I dont want to repeat every method signature manually. But maybe that's the price to pay for keeping the language simple. Or we just define the modules at the point where the symbols are first defined. Maybe we just force the user to define the symbols in a file, and the file name or directory makes the module name.

// in another file:

use player // brings symbols of player module into scope
```

### Thoughts about impling two traits with two methods with same method signatures

If two traits with two methods with identical function signature are implemented on a type, yes I would just throw a compiler error because you cannot define two methods with the exact definition twice. That would be the V1 of the language at least.

But that COULD maybe be quite annoying, yes. It's tricky. But I mean maybe there are ways. Maybe impls could be scoped. And the functions that it requires to be implemented are then contained in that same scope/module having to be called explicitly with that scope prefix like

```
mod serializable {
	impl Serializable[User] // requires methods to be defined in this scope
}

// then call like this:
serializable:log[User](&user)
  
// or alternatively
user.serializable:log()
```

### A thought on the `impl` block

If you go with the scoped `impl`, you open up a cool possibility: **Conditional Implementations.**

```
mod debug_stuff {
    // Only available if this module is used
    impl Serializable[User] 
}
```

# Memory management
```
// Instead of this (hidden dependency): 
fn create_list() &List { ... } 

// You do this (explicit dependency): 
fn create_list(alloc &Allocator) &List { 
	mem RawPtr[List] = alloc.alloc[List](size_of(List)) // returns a typed raw pointer to the allocated memory
	// ... 
}
```
Maybe there could be context allocators like in Odin where the allocator is passed implicitly via a context. But it should be obvious from the call site that an allocator is used.

# Struct initializer shorthand

```
struct Cat { name String }

cat := Cat.{ name = "Matz" }
// equivalent to:

cat := Cat
cat.name = "Matz"
```

Instead of Arrays being their own built-in type with special syntax, they are just a generic type like Array\[T] with compiler intrinsic functions like `len`, `get`, `set` etc. I would like to have a syntax for initializing with values though. And it's hard to not make it collide with other syntax. I want to keep the compiler orthogonal thats why a syntax like `Array[Int].{ 1, 2, 3 }` similar to the struct initializer shorthand syntax would be nice.

Thinking further I don't like the idea of allowing non-explicit zero-initialization. It's too easy to forget to initialize a variable and end up with a garbage value. That's why a variable that's meant to be zeroed (for low-level purposes) should require an explicit zero-initialization like: my_variable _ = zero

### Auto traits
Instead of having to manually implement traits for every type, Mo could automatically consider a trait as implemented for types that implement the trait's methods. The trait implementation could be marked as `auto` so the compiler knows to automatically implement it for types that implement the trait's methods. This might cause problems though. Maybe this is only useful for built-in traits. In Rust they have the Sync or Unsized? traits that are automatically implemented for types that implement the trait's methods. 

### What if you pass a value into a function?
Ideally, I would like to have a move system like in Rust. Right?
Another way would be to just copy the value into the function and work with the copy like in C. BUT this would require deep copying for complex types, which is not ideal. Im not sure if move requires a borrowing system like in Rust. Maybe we dont need to make the variable invalid after it's moved into a function. This might cause some errors, but it's a good tradeoff for a simple language. I need to think about this further. 