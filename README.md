# PowderCore™ – The Universal Physics Engine

### *The engine that powers every grain, spark, and flame across the Powder Sandbox universe.*

---

## 🌌 Overview

**PowderCore™** is the shared, language-agnostic **physics engine** that powers all editions of the [Powder Sandbox](https://github.com/RobertFlexx/Powder-Sandbox-Classic) project family — from C++ and C to Rust, Kotlin, Scala, Groovy, C#, and F#.

It defines how every grain of sand falls, how fire spreads, how water cools lava, and how lightning electrifies metal — bringing every sandbox to life through a consistent and modular simulation model.

Whether you’re running the **Ferrite-based Linux shell**, a **JVM edition**, or a **.NET build**, PowderCore is the beating heart underneath it all.

---

## ⚙️ Core Principles

PowderCore’s design revolves around four guiding pillars:

1. **Consistency** — identical elemental logic across all language editions.
2. **Portability** — easy to integrate with any runtime or platform.
3. **Performance** — optimized loops, predictable simulation ticks, and minimal overhead.
4. **Chaos** — physics should feel *alive*, emergent, and unpredictable in the best way.

---

## 🔬 Engine Architecture

PowderCore operates on a **cellular automata model**, where each cell (or “particle”) is an independent unit with its own properties and update rules.

### Major Components:

| Module               | Responsibility                                                                     |
| -------------------- | ---------------------------------------------------------------------------------- |
| **Core Grid**        | Maintains the 2D/3D simulation space and handles updates per tick.                 |
| **Element System**   | Defines all materials and their reactions (fire, water, acid, gas, etc).           |
| **Reaction Engine**  | Executes inter-element interactions such as melting, burning, cooling, and fusing. |
| **AI Behavior**      | Handles logic for actors (humans, zombies, etc).                                   |
| **Thermal System**   | Manages temperature, heat transfer, and combustion.                                |
| **Electrical Model** | Simulates wire, metal, and conductive interactions.                                |
| **Rendering API**    | Provides hooks for TUIs, ASCII renderers, and color displays.                      |

---

## 🧱 Key Features

* Cross-language elemental model (C, Rust, .NET, JVM)
* Modular architecture – easy to port or embed
* Deterministic update order with pseudo-randomized diffusion
* Realistic reactions: melting, dissolving, condensing, shocking, etc.
* Support for AI-driven entities (humans, zombies, etc.)
* Configurable grid resolution and tick speed
* Plug-in style extensibility for new materials
* Color-agnostic rendering hooks (you choose your graphics backend)

---

## 💡 Integration Examples

**For C/C++ Projects:**

```c
#include "powdercore.h"
world_init(WIDTH, HEIGHT);
world_tick();
world_draw();
```

**For Rust Projects:**

```rust
use powdercore::World;

let mut world = World::new(200, 100);
world.update();
world.render();
```

**For .NET Projects (C#/F#):**

```csharp
using PowderCore;
var sim = new World(200, 100);
sim.Update();
sim.Render();
```

**For JVM Projects:**

```kotlin
val world = PowderCore.World(200, 100)
world.tick()
world.draw()
```

PowderCore is designed to be language-neutral — the logic can be directly ported or wrapped as a native or shared library depending on your target environment.

---

## 🌍 Language Implementations

| Edition         | Language | Repo                                                                                                                         |
| --------------- | -------- | ---------------------------------------------------------------------------------------------------------------------------- |
| C++ Classic     | C++17    | [https://github.com/RobertFlexx/Powder-Sandbox-Classic](https://github.com/RobertFlexx/Powder-Sandbox-Classic)               |
| Fast Edition    | C        | [https://github.com/RobertFlexx/Powder-Sandbox-Fast-Edition](https://github.com/RobertFlexx/Powder-Sandbox-Fast-Edition)     |
| Rustbox Edition | Rust     | [https://github.com/RobertFlexx/Rustbox-Sandbox](https://github.com/RobertFlexx/Rustbox-Sandbox)                             |
| Kotlin Edition  | Kotlin   | [https://github.com/RobertFlexx/Powder-Sandbox-Kotlin-Edition](https://github.com/RobertFlexx/Powder-Sandbox-Kotlin-Edition) |
| Scala Edition   | Scala    | [https://github.com/RobertFlexx/Powder-Sandbox-Scala-Edition](https://github.com/RobertFlexx/Powder-Sandbox-Scala-Edition)   |
| Groovy Edition  | Groovy   | [https://github.com/RobertFlexx/Powder-Sandbox-Groovy-Edition](https://github.com/RobertFlexx/Powder-Sandbox-Groovy-Edition) |
| C# Edition      | C#/.NET  | [https://github.com/RobertFlexx/Powder-Sandbox-CS-Edition](https://github.com/RobertFlexx/Powder-Sandbox-CS-Edition)         |
| F# Edition      | F#/.NET  | [https://github.com/RobertFlexx/Power-Sandbox-F-Edition](https://github.com/RobertFlexx/Power-Sandbox-F-Edition)             |

---

## 🧩 Future Goals

* Shared simulation benchmarks between languages
* Common reaction table JSON definition
* Engine-level save/load and serialization
* Parallel processing for multicore performance
* GPU offload experiments (CUDA, Vulkan compute)
* Plugin API for custom element packs
* Shared test suite across all bindings

---

## 🧠 Philosophy

PowderCore™ isn’t just about simulating physics — it’s about capturing **emergent behavior**. Every pixel tells a story: water extinguishing fire, lava cooling into stone, plants growing through dirt, and lightning snaking across the grid.

The goal is to make the chaos *consistent* across languages — a universal standard for sandbox simulations.

---

## 📜 License

Released under the **BSD 3-Clause License**.
All derivative editions of Powder Sandbox inherit this license.

---

## 👤 Author

**Robert (@RobertFlexx)**
Creator of FerriteOS, Powder Sandbox, and the Ferrite ecosystem of shells, editors, and experimental languages.

GitHub: [https://github.com/RobertFlexx](https://github.com/RobertFlexx)

---

### ✨ Tagline

> **PowderCore™** — *The engine of entropy.*
