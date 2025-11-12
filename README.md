# PowderCore: Universal Physics Engine
> An engine extracted from my projects

See the [GameHub](https://github.com/RobertFlexx/Powder-Sandbox-GameHub) for detailed information on the editions themselves.

---

## Overview

**PowderCore** is the shared, language-agnostic **physics engine** behind all editions of the [Powder Sandbox](https://github.com/RobertFlexx/Powder-Sandbox-Classic) project family. C++, C, Rust, Kotlin, Scala, Groovy, C#, and F#.

It uses a consistent, modular simulation model to simulate everything.

it wasn’t even supposed to exist.

**PowderCore** was *accidentally born* from a bug in an ASCII image generator. The characters started behaving like particles, moving and interacting like fluid. The first “discovered” element was water. Instead of fixing that bug, I built on it, creating powdercore in the process.

Later, I rewrote the project in **C#** and **Rust** to test whether the strange physics behavior was language-dependent. It wasn’t. I kept creating new versions in different languages for fun and just to see how far I could go.

---

## Core Principles (and why it was made in Rust)

1. **Consistency**
   Identical behavior across all platforms.

2. **Portability**
   Easy to use in any runtime or language that supports C or Rust.

3. **Performance**
   Memory-safe, parallelizable, and cache-friendly.

4. **Emergent Chaos**
   Physics should feel alive and unpredictable in the best way.

---

## Engine Architecture

PowderCore operates on a **cellular automata model**, where each cell (or “particle”) is an independent unit with its own properties and update rules.

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

### Kotlin (JVM)

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

## Language Implementations

| Edition         | Language | Repo                                                                                          |
| --------------- | -------- | --------------------------------------------------------------------------------------------- |
| C++ Classic     | C++17    | [Powder-Sandbox-Classic](https://github.com/RobertFlexx/Powder-Sandbox-Classic)               |
| Fast Edition    | C        | [Powder-Sandbox-Fast-Edition](https://github.com/RobertFlexx/Powder-Sandbox-Fast-Edition)     |
| Rustbox Edition | Rust     | [Rustbox-Sandbox](https://github.com/RobertFlexx/Rustbox-Sandbox)                             |
| Kotlin Edition  | Kotlin   | [Powder-Sandbox-Kotlin-Edition](https://github.com/RobertFlexx/Powder-Sandbox-Kotlin-Edition) |
| Scala Edition   | Scala    | [Powder-Sandbox-Scala-Edition](https://github.com/RobertFlexx/Powder-Sandbox-Scala-Edition)   |
| Groovy Edition  | Groovy   | [Powder-Sandbox-Groovy-Edition](https://github.com/RobertFlexx/Powder-Sandbox-Groovy-Edition) |
| C# Edition      | C#/.NET  | [Powder-Sandbox-CS-Edition](https://github.com/RobertFlexx/Powder-Sandbox-CS-Edition)         |
| F# Edition      | F#/.NET  | [Power-Sandbox-F-Edition](https://github.com/RobertFlexx/Power-Sandbox-F-Edition)             |

---

## Future Goals/Features

* Shared reaction table (JSON-based)
* Engine-level save/load and serialization
* Parallelism and GPU acceleration
* Plugin system for custom materials
* Unified benchmarking across editions

---

## License

Released under the **BSD 3-Clause License**.
All derivative editions of Powder Sandbox and PowderCore-based projects inherit compatible licensing.

---

## Author

**Robert (@RobertFlexx)**
Creator of FerriteOS, Powder Sandbox, and a bunch of other random open-source shells, editors, and simulation frameworks.

GitHub: [https://github.com/RobertFlexx](https://github.com/RobertFlexx)

---

> **PowderCore** - Accidentally discovered, made awesome.
