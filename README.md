# PowderCore™: Universal Physics Engine

> A homemade, physics engine extracted from my projects, to make this an independent, scalable, integrable, library made in Rust for stability, and memory safety.

## The well-engineered core of Powder Sandbox

See the [GameHub](https://github.com/RobertFlexx/Powder-Sandbox-GameHub) for detailed information on the editions themselves.

---

## Overview

**PowderCore** is the shared, language-agnostic **physics engine** behind all editions of the [Powder Sandbox](https://github.com/RobertFlexx/Powder-Sandbox-Classic) project family. C++, C, Rust, Kotlin, Scala, Groovy, C#, and F#.

It uses a consistent, modular simulation model to simulate everything.

it wasn’t even supposed to exist.

**PowderCore** was accidentally born from a bug in an ASCII image generator. The characters started behaving like particles, moving and interacting like fluid. The first "discovered" element was water. Instead of fixing that bug, I built on it, creating PowderCore in the process.

Later, I rewrote the project in **C#** and **Rust** to test whether the strange physics behavior was language-dependent. It wasn’t. I kept creating new versions in different languages just to see how far I could push it.

---

## Core Principles and why I chose Rust

1. **Consistency**
   Identical behavior across all platforms.

2. **Portability**
   Easy to use in any runtime or language that supports C or Rust.

3. **Performance**
   Memory-safe, parallelizable, and cache-friendly.

4. **Emergent Chaos**
   Physics should feel alive and unpredictable.

5. **Memory-safety aspects**
   Prevents common memory-driven bugs with fast performance.

6. **Elegant Flow**
   Rust code flows nicely and stays readable long term.

---

## Engine Architecture (Basics)

PowderCore operates on a **cellular automata model**, where each cell (or particle) is an independent unit with its own properties and update rules.

| Module               | Responsibility                                      |
| -------------------- | --------------------------------------------------- |
| **Core Grid**        | Maintains the 2D world and handles ticks            |
| **Element System**   | Defines all materials and their behaviors           |
| **Reaction Engine**  | Handles interactions like melting, burning, cooling |
| **AI Behavior**      | Simulates humans, zombies, and their instincts      |
| **Thermal System**   | Temperature, combustion, and heat spread            |
| **Electrical Model** | Conductivity, wire logic, lightning                 |
| **Integration API**  | Hooks for host languages and renderers              |

---

## Integration Examples

PowderCore is designed to integrate almost anywhere. The Rust backend exposes a clean, minimal C API that can be consumed from almost any environment.

### Note on the C API

The engine now ships with an FFI friendly interface built around a small C header. It exposes plain functions and stable data types, so any language that can call C code can hook into the engine without trouble. This includes Python ctypes, Ruby FFI, LuaJIT, Go cgo, .NET PInvoke, Zig, Nim, and JVM JNI.

### Rust

```rust
use powdercore::{World, Element};

fn main() {
    let mut world = World::new(200, 120);
    for x in 90..110 {
        world.set(x, 10, Element::Sand);
    }
    loop {
        world.step();
    }
}
```

### C / C++

```c
#include "powdercore.h"
int main() {
    World* world = powder_world_new(200, 120);
    powder_world_set_cell(world, 50, 10, ELEMENT_SAND);
    for (;;) powder_world_step(world);
}
```

### C# / F# (.NET)

```csharp
[DllImport("powdercore")] static extern IntPtr powder_world_new(uint w, uint h);
[DllImport("powdercore")] static extern void powder_world_step(IntPtr world);
```

### Kotlin

```kotlin
val core = Native.load("powdercore", PowderCoreLib::class.java)
val world = core.powder_world_new(200, 100)
while (true) core.powder_world_step(world)
```

### Python

```python
from ctypes import *
lib = CDLL("libpowdercore.so")
world = lib.powder_world_new(200, 100)
while True:
    lib.powder_world_step(world)
```

### Go

```go
// #include "powdercore.h"
import "C"
world := C.powder_world_new(200, 100)
for { C.powder_world_step(world) }
```

### Ruby

```ruby
require 'ffi'
module PowderCore; extend FFI::Library; ffi_lib 'powdercore'; end
```

### Haskell

```haskell
foreign import ccall "powder_world_step" c_powder_world_step :: Ptr World -> IO ()
```

---

## Extra Notes for Developers

This is also where I keep the extra info for folks who want a little more than the quick overview. Nothing complicated, just straight to the point.

### WASM Notes

You can compile the engine to WebAssembly using the usual Rust target. The engine itself stays the same. The only real step is exposing the C API or a JS friendly wrapper. Most people will compile the library like this:

```
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
```

Then you take the generated wasm file and plug it into your frontend with whatever bundler or wasm loader you like. The simulation loop works the same way, just driven by requestAnimationFrame or a manual tick timer.

### Basic Build Commands

Linux and macOS builds are simple. Most folks will do:

```
cargo build --release
```

This creates `libpowdercore.so` or `libpowdercore.dylib` depending on your OS. Windows will produce a DLL. If someone needs a static build, they would do:

```
cargo build --release --features static
```

If you want the header next to the library for convenience, you can copy it out during your build script.

### More Import Examples

These are short examples that match the exact way people usually bring in a C library.

#### Zig

```
const std = @import("std");
const pc = @cImport({
    @cInclude("powdercore.h");
});

pub fn main() void {
    const w = pc.powder_world_new(200, 150, 1234);
    pc.powder_world_step(w);
}
```

#### Nim

```
{.passL: "-lpowdercore".}
proc world_new(w, h: cint, seed: culong): pointer {.importc: "powder_world_new".}
proc world_step(p: pointer) {.importc: "powder_world_step".}

let w = world_new(200, 150, 1234)
world_step(w)
```

#### Node.js (ffi-napi)

```
const ffi = require('ffi-napi');
const core = ffi.Library('powdercore', {
  'powder_world_new': ['pointer', ['int', 'int', 'ulong']],
  'powder_world_step': ['void', ['pointer']]
});

const w = core.powder_world_new(200, 100, 1234);
setInterval(() => core.powder_world_step(w), 16);
```

#### LuaJIT

```
local ffi = require('ffi')
ffi.cdef[[
  void* powder_world_new(int w, int h, uint64_t seed);
  void  powder_world_step(void* w);
]]
local core = ffi.load('powdercore')

local w = core.powder_world_new(200, 150, 1234)
while true do
  core.powder_world_step(w)
end
```

If wanted, the README can be expanded with a short optional section that explains the new C API in more detail, the portability guarantees it provides, a link to a downloadable header file, a couple build command examples, quick WASM notes, and some broader multi-language snippets. This is optional and only added if someone wants a deeper breakdown.

---

## Language Implementations

| Edition         | Language | Repo                                                                                                                         |
| --------------- | -------- | ---------------------------------------------------------------------------------------------------------------------------- |
| C++ Classic     | C++17    | [https://github.com/RobertFlexx/Powder-Sandbox-Classic](https://github.com/RobertFlexx/Powder-Sandbox-Classic)               |
| Fast Edition    | C        | [https://github.com/RobertFlexx/Powder-Sandbox-Fast-Edition](https://github.com/RobertFlexx/Powder-Sandbox-Fast-Edition)     |
| Rustbox Edition | Rust     | [https://github.com/RobertFlexx/Rustbox-Sandbox](https://github.com/RobertFlexx/Rustbox-Sandbox)                             |
| Kotlin Edition  | Kotlin   | [https://github.com/RobertFlexx/Powder-Sandbox-Kotlin-Edition](https://github.com/RobertFlexx/Powder-Sandbox-Kotlin-Edition) |
| Scala Edition   | Scala    | [https://github.com/RobertFlexx/Powder-Sandbox-Scala-Edition](https://github.com/RobertFlexx/Powder-Sandbox-Scala-Edition)   |
| Groovy Edition  | Groovy   | [https://github.com/RobertFlexx/Powder-Sandbox-Groovy-Edition](https://github.com/RobertFlexx/Powder-Sandbox-Groovy-Edition) |
| C# Edition      | C#/.NET  | [https://github.com/RobertFlexx/Powder-Sandbox-CS-Edition](https://github.com/RobertFlexx/Powder-Sandbox-CS-Edition)         |
| F# Edition      | F#/.NET  | [https://github.com/RobertFlexx/Powder-Sandbox-F-Edition](https://github.com/RobertFlexx/Powder-Sandbox-F-Edition)           |

---

## Future Goals

* Shared reaction table (JSON based)
* Engine level save load and serialization
* Parallelism and GPU acceleration
* Plugin system for custom materials
* Unified benchmarking across editions

---

## License

Released under the BSD 3 Clause License. All derivative editions inherit compatible licensing.

---

## Author

**Robert (@RobertFlexx)**
Creator of FerriteOS, Powder Sandbox, and other open source shells, editors, and simulation frameworks.

GitHub: [https://github.com/RobertFlexx](https://github.com/RobertFlexx)

---

> PowderCore. Accidentally discovered, designed with precision, and made awesome. (cuz im da goat)
