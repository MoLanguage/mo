- Base64 literals


# Idea 2
Grouping symbols of item declarations into a module / scope that can be imported deputy for the specified symbols.

Example:

```
struct Player { ... }
fn heal(player *Player) { ... }

mod player { Player, heal(*Player) } // Note to dev: function args are just type expressions here


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
serializable:log:[User](&user)
  
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
fn create_list() *List { ... } 

// You do this (explicit dependency): 
fn create_list(alloc *Allocator) *List { 
	mem := alloc.alloc(size_of(List)) 
	// ... 
}
```
# Struct initializer shorthand

```
struct Cat { name String }

cat := Cat.{ name = "Matz" }
// equivalent to:

cat := Cat
cat.name = "Matz"
```