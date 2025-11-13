// PowderCore - bare physics engine for Powder Sandbox
//
// This file contains the *simulation logic only*:
// - Element definitions
// - Cell and World structures
// - Helpers for classification, names, glyphs, colors
// - Brush placement and explosion logic
// - One simulation step (tick)
//
// There is NO rendering, input, or terminal code here.
// You are expected to call World::step() from your own loop
// and render cells however you like (ncurses, ANSI, GUI, etc).

// ===== Elements =====

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Element {
    Empty,
    // powders
    Sand,
    Gunpowder,
    Ash,
    Snow,
    // liquids
    Water,
    SaltWater,
    Oil,
    Ethanol,
    Acid,
    Lava,
    Mercury,
    // solids / terrain
    Stone,
    Glass,
    Wall,
    Wood,
    Plant,
    Metal,
    Wire,
    Ice,
    Coal,
    Dirt,
    WetDirt,
    Seaweed,
    // gases
    Smoke,
    Steam,
    Gas,
    ToxicGas,
    Hydrogen,
    Chlorine,
    // actors / special
    Fire,
    Lightning,
    Human,
    Zombie,
}

#[derive(Copy, Clone, Debug)]
pub struct Cell {
    pub elem: Element,
    pub life: i32, // age / gas lifetime / charge / wetness / anim tick
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            elem: Element::Empty,
            life: 0,
        }
    }
}

// ===== Very simple PRNG (no external crate) =====
//
// We use a tiny LCG so the engine is self-contained and deterministic.

#[derive(Clone)]
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        let s = if seed == 0 {
            0xDEADBEEFCAFEBABE
        } else {
            seed
        };
        Rng { state: s }
    }

    fn next_u32(&mut self) -> u32 {
        // LCG: same style as classic rand() implementations
        self.state = self
            .state
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        (self.state >> 16) as u32
    }

    fn range_i32(&mut self, min: i32, max: i32) -> i32 {
        // inclusive range [min, max]
        let span = (max - min + 1).max(1) as u32;
        let v = self.next_u32() % span;
        min + v as i32
    }

    fn chance(&mut self, pct: u32) -> bool {
        if pct == 0 {
            return false;
        }
        if pct >= 100 {
            return true;
        }
        (self.next_u32() % 100) < pct
    }
}

// ===== World: core engine state =====

pub struct World {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
    rng: Rng,
}

impl World {
    /// Create a new world with given width/height and RNG seed.
    /// All cells start as Empty.
    pub fn new(width: i32, height: i32, seed: u64) -> Self {
        let size = (width.max(0) * height.max(0)) as usize;
        World {
            width: width.max(0),
            height: height.max(0),
            cells: vec![Cell::default(); size],
            rng: Rng::new(seed),
        }
    }

    /// Resize the world, clearing all contents.
    pub fn resize(&mut self, width: i32, height: i32) {
        self.width = width.max(0);
        self.height = height.max(0);
        let size = (self.width * self.height).max(0) as usize;
        self.cells = vec![Cell::default(); size];
    }

    /// World width.
    pub fn width(&self) -> i32 {
        self.width
    }

    /// World height.
    pub fn height(&self) -> i32 {
        self.height
    }

    /// Get an immutable view of a cell (returns Empty for out-of-bounds).
    pub fn get_cell(&self, x: i32, y: i32) -> Cell {
        if !self.in_bounds(x, y) {
            return Cell::default();
        }
        self.cells[self.idx(x, y)]
    }

    /// Get a mutable reference to a cell. Returns None for out-of-bounds.
    pub fn get_cell_mut(&mut self, x: i32, y: i32) -> Option<&mut Cell> {
        if !self.in_bounds(x, y) {
            return None;
        }
        let i = self.idx(x, y);
        Some(&mut self.cells[i])
    }

    /// Clear the world to Empty.
    pub fn clear(&mut self) {
        for c in &mut self.cells {
            *c = Cell::default();
        }
    }

    /// Place a circular brush of element `elem` at (cx, cy) with radius `rad`.
    /// Lightning is treated specially (vertical bolt).
    pub fn place_brush(&mut self, cx: i32, cy: i32, rad: i32, elem: Element) {
        if elem == Element::Lightning {
            self.place_lightning(cx, cy);
            return;
        }

        let r2 = rad * rad;
        for dy in -rad..=rad {
            for dx in -rad..=rad {
                if dx * dx + dy * dy > r2 {
                    continue;
                }
                let x = cx + dx;
                let y = cy + dy;
                if !self.in_bounds(x, y) {
                    continue;
                }
                let idx = self.idx(x, y);
                self.cells[idx].elem = elem;
                self.cells[idx].life = match elem {
                    Element::Fire => 20,
                    e if is_gas(e) => 25,
                    _ => 0,
                };
            }
        }
    }

    /// Single simulation tick: updates all cells in-place.
    ///
    /// Call this once per frame from your game loop.
    pub fn step(&mut self) {
        if self.width <= 0 || self.height <= 0 {
            return;
        }

        let w = self.width;
        let h = self.height;
        let mut updated = vec![false; (w * h) as usize];

        // Bottom-up traversal matches original C++ stepping order
        for y in (0..h).rev() {
            for x in 0..w {
                let idx0 = self.idx(x, y);
                if updated[idx0] {
                    continue;
                }

                let elem = self.cells[idx0].elem;
                if elem == Element::Empty || elem == Element::Wall {
                    updated[idx0] = true;
                    continue;
                }

                // POWDERS
                if is_sand_like(elem) {
                    self.step_powder(x, y, &mut updated);
                    continue;
                }

                // LIQUIDS
                if is_liquid(elem) {
                    self.step_liquid(x, y, &mut updated);
                    continue;
                }

                // GASES
                if is_gas(elem) {
                    self.step_gas(x, y, &mut updated);
                    continue;
                }

                // FIRE
                if elem == Element::Fire {
                    self.step_fire(x, y, &mut updated);
                    continue;
                }

                // LIGHTNING
                if elem == Element::Lightning {
                    self.step_lightning(x, y, &mut updated);
                    continue;
                }

                // HUMANS
                if elem == Element::Human {
                    self.step_human(x, y, &mut updated);
                    continue;
                }

                // ZOMBIES
                if elem == Element::Zombie {
                    self.step_zombie(x, y, &mut updated);
                    continue;
                }

                // WET DIRT
                if elem == Element::WetDirt {
                    self.step_wet_dirt(x, y, &mut updated);
                    continue;
                }

                // PLANTS / SEAWEED
                if elem == Element::Plant || elem == Element::Seaweed {
                    self.step_plant_like(x, y, &mut updated);
                    continue;
                }

                // WOOD / COAL BURN
                if elem == Element::Wood || elem == Element::Coal {
                    self.step_burnable_solid(x, y, &mut updated);
                    continue;
                }

                // GUNPOWDER
                if elem == Element::Gunpowder {
                    self.step_gunpowder(x, y, &mut updated);
                    continue;
                }

                // WIRE / METAL conduction
                if elem == Element::Wire || elem == Element::Metal {
                    self.step_conductor(x, y, &mut updated);
                    continue;
                }

                // ICE
                if elem == Element::Ice {
                    self.step_ice(x, y, &mut updated);
                    continue;
                }

                // Default: static
                updated[idx0] = true;
            }
        }
    }

    // ===== Internal helpers =====

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    fn idx(&self, x: i32, y: i32) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    /// Place a vertical lightning bolt that travels downward until it hits
    /// non-air / non-gas or the bottom.
    fn place_lightning(&mut self, cx: i32, cy: i32) {
        if !self.in_bounds(cx, cy) {
            return;
        }

        let mut x = cx;
        let mut y = cy;

        // Fall straight down through empty or gas
        while y + 1 < self.height {
            let below_idx = self.idx(x, y + 1);
            let below = self.cells[below_idx].elem;
            if below != Element::Empty && !is_gas(below) {
                break;
            }
            y += 1;
        }

        // Fill the bolt path
        for yy in cy..=y {
            let idx = self.idx(x, yy);
            self.cells[idx].elem = Element::Lightning;
            self.cells[idx].life = 2;
        }

        // Electrify water directly below impact
        if y + 1 < self.height {
            let idx_below = self.idx(x, y + 1);
            let cell = &mut self.cells[idx_below];
            if cell.elem == Element::Water || cell.elem == Element::SaltWater {
                cell.life = cell.life.max(8);
            }
        }
    }

    fn explode(&mut self, cx: i32, cy: i32, r: i32) {
        let r2 = r * r;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r2 {
                    continue;
                }
                let x = cx + dx;
                let y = cy + dy;
                if !self.in_bounds(x, y) {
                    continue;
                }
                let idx = self.idx(x, y);
                let cell = &mut self.cells[idx];
                match cell.elem {
                    Element::Wall
                    | Element::Stone
                    | Element::Glass
                    | Element::Metal
                    | Element::Wire
                    | Element::Ice => {
                        // Explosion can't destroy these
                    }
                    _ => {
                        let roll = self.rng.range_i32(1, 100);
                        if roll <= 50 {
                            cell.elem = Element::Fire;
                            cell.life = 15 + self.rng.range_i32(0, 10);
                        } else if roll <= 80 {
                            cell.elem = Element::Smoke;
                            cell.life = 20;
                        } else {
                            cell.elem = Element::Gas;
                            cell.life = 20;
                        }
                    }
                }
            }
        }
    }

    // ===== Step categories =====

    fn step_powder(&mut self, x: i32, y: i32, updated: &mut [bool]) {
        let idx0 = self.idx(x, y);
        let t = self.cells[idx0].elem;
        let mut moved = false;

        // Fall straight down into empty or liquid
        if self.in_bounds(x, y + 1) {
            let idx_below = self.idx(x, y + 1);
            let below = self.cells[idx_below].elem;
            if below == Element::Empty || is_liquid(below) {
                self.cells.swap(idx0, idx_below);
                updated[idx_below] = true;
                moved = true;
            }
        }

        // Slide diagonally if straight down is blocked
        if !moved {
            let dir = if self.rng.chance(50) { 1 } else { -1 };
            for i in 0..2 {
                let nx = x + if i == 0 { dir } else { -dir };
                let ny = y + 1;
                if !self.in_bounds(nx, ny) {
                    continue;
                }
                let idx_n = self.idx(nx, ny);
                let e = self.cells[idx_n].elem;
                if e == Element::Empty || is_liquid(e) {
                    self.cells.swap(idx0, idx_n);
                    updated[idx_n] = true;
                    moved = true;
                    break;
                }
            }
        }

        if !moved {
            updated[idx0] = true;
        }

        // Snow melts near heat
        if t == Element::Snow {
            let mut melt = false;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = x + dx;
                    let ny = y + dy;
                    if !self.in_bounds(nx, ny) {
                        continue;
                    }
                    let e = self.cells[self.idx(nx, ny)].elem;
                    if e == Element::Fire || e == Element::Lava {
                        melt = true;
                        break;
                    }
                }
                if melt {
                    break;
                }
            }
            if melt {
                let c = &mut self.cells[idx0];
                c.elem = Element::Water;
                c.life = 0;
            }
        }

        // Seaweed seed: sand under persistent water, spaced apart
        if t == Element::Sand {
            let mut life = self.cells[idx0].life;
            if self.in_bounds(x, y - 1)
                && self.cells[self.idx(x, y - 1)].elem == Element::Water
            {
                life += 1;
                if life > 220 {
                    let mut nearby_weed = false;
                    for wy in -2..=2 {
                        for wx in -2..=2 {
                            let sx = x + wx;
                            let sy = y + wy;
                            if !self.in_bounds(sx, sy) {
                                continue;
                            }
                            if self.cells[self.idx(sx, sy)].elem == Element::Seaweed {
                                nearby_weed = true;
                                break;
                            }
                        }
                        if nearby_weed {
                            break;
                        }
                    }
                    if !nearby_weed
                        && self.in_bounds(x, y - 1)
                        && self.cells[self.idx(x, y - 1)].elem == Element::Water
                    {
                        let idx_above = self.idx(x, y - 1);
                        self.cells[idx_above].elem = Element::Seaweed;
                        self.cells[idx_above].life = 0;
                    }
                    life = 0;
                }
            } else {
                life = 0;
            }
            self.cells[idx0].life = life;
        }
    }

    fn step_liquid(&mut self, x: i32, y: i32, updated: &mut [bool]) {
        let idx0 = self.idx(x, y);
        let t = self.cells[idx0].elem;
        let mut moved = false;

        // Fall down through empty or gas, or heavier-over-lighter liquid swap
        if self.in_bounds(x, y + 1) {
            let idx_b = self.idx(x, y + 1);
            let b = self.cells[idx_b].elem;
            if b == Element::Empty || is_gas(b) {
                self.cells.swap(idx0, idx_b);
                updated[idx_b] = true;
                moved = true;
            } else if is_liquid(b) && density(t) > density(b) {
                self.cells.swap(idx0, idx_b);
                updated[idx_b] = true;
                moved = true;
            }
        }

        // Lateral flow
        if !moved {
            let mut order = [-1, 1];
            if self.rng.chance(50) {
                order.swap(0, 1);
            }
            for &dx in &order {
                let nx = x + dx;
                if !self.in_bounds(nx, y) {
                    continue;
                }
                let idx_n = self.idx(nx, y);
                let e = self.cells[idx_n].elem;
                if e == Element::Empty || is_gas(e) {
                    self.cells.swap(idx0, idx_n);
                    updated[idx_n] = true;
                    moved = true;
                    break;
                } else if is_liquid(e) && density(t) > density(e) && self.rng.chance(50) {
                    self.cells.swap(idx0, idx_n);
                    updated[idx_n] = true;
                    moved = true;
                    break;
                }
            }
        }

        if !moved {
            updated[idx0] = true;
        }

        // Neighbor interactions based on original element type
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if !self.in_bounds(nx, ny) {
                    continue;
                }
                let n_idx = self.idx(nx, ny);
                let n = self.cells[n_idx];

                // Water vs fire/lava
                if t == Element::Water || t == Element::SaltWater {
                    if n.elem == Element::Fire {
                        let c = &mut self.cells[n_idx];
                        c.elem = Element::Smoke;
                        c.life = 15;
                    } else if n.elem == Element::Lava {
                        // cool lava, sometimes big steam
                        {
                            let c = &mut self.cells[n_idx];
                            c.elem = Element::Stone;
                            c.life = 0;
                        }
                        let self_cell = &mut self.cells[idx0];
                        if self.rng.chance(50) {
                            self_cell.elem = Element::Steam;
                            self_cell.life = 20;
                        } else {
                            self_cell.elem = Element::Stone;
                            self_cell.life = 0;
                        }
                    }
                }

                // Oil / Ethanol ignite
                if t == Element::Oil || t == Element::Ethanol {
                    if n.elem == Element::Fire || n.elem == Element::Lava {
                        let self_cell = &mut self.cells[idx0];
                        self_cell.elem = Element::Fire;
                        self_cell.life = 25;
                    }
                }

                // Acid eats stuff
                if t == Element::Acid {
                    if is_dissolvable(n.elem) {
                        if self.rng.chance(30) {
                            let c = &mut self.cells[n_idx];
                            c.elem = Element::ToxicGas;
                            c.life = 25;
                        } else {
                            let c = &mut self.cells[n_idx];
                            c.elem = Element::Empty;
                            c.life = 0;
                        }
                        if self.rng.chance(25) {
                            let c = &mut self.cells[idx0];
                            c.elem = Element::Empty;
                            c.life = 0;
                        }
                    }
                    if n.elem == Element::Water && self.rng.chance(30) {
                        {
                            let c = &mut self.cells[idx0];
                            c.elem = Element::SaltWater;
                            c.life = 0;
                        }
                        if self.rng.chance(30) {
                            let c = &mut self.cells[n_idx];
                            c.elem = Element::Steam;
                            c.life = 20;
                        }
                    }
                }

                // Lava interactions
                if t == Element::Lava {
                    if is_flammable(n.elem) {
                        let c = &mut self.cells[n_idx];
                        c.elem = Element::Fire;
                        c.life = 25;
                    } else if n.elem == Element::Sand || n.elem == Element::Snow {
                        let c = &mut self.cells[n_idx];
                        c.elem = Element::Glass;
                        c.life = 0;
                    } else if n.elem == Element::Water || n.elem == Element::SaltWater {
                        {
                            let c = &mut self.cells[n_idx];
                            c.elem = Element::Stone;
                            c.life = 0;
                        }
                        let self_cell = &mut self.cells[idx0];
                        if self.rng.chance(50) {
                            self_cell.elem = Element::Steam;
                            self_cell.life = 20;
                        } else {
                            self_cell.elem = Element::Stone;
                            self_cell.life = 0;
                        }
                    } else if n.elem == Element::Ice {
                        let c = &mut self.cells[n_idx];
                        c.elem = Element::Water;
                        c.life = 0;
                    }
                }
            }
        }

        // Lava cools over time into stone
        if t == Element::Lava {
            let c = &mut self.cells[idx0];
            c.life += 1;
            if c.life > 200 {
                c.elem = Element::Stone;
                c.life = 0;
            }
        }

        // Hydrate dirt around water/saltwater
        if t == Element::Water || t == Element::SaltWater {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = x + dx;
                    let ny = y + dy;
                    if !self.in_bounds(nx, ny) {
                        continue;
                    }
                    let idx_n = self.idx(nx, ny);
                    let n = &mut self.cells[idx_n];
                    if n.elem == Element::Dirt || n.elem == Element::WetDirt {
                        n.elem = Element::WetDirt;
                        n.life = 300;
                    }
                }
            }
        }

        // Electrified water pulse: spreads charge + kills humans/zombies
        if (t == Element::Water || t == Element::SaltWater) && self.cells[idx0].life > 0 {
            let q = self.cells[idx0].life;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x + dx;
                    let ny = y + dy;
                    if !self.in_bounds(nx, ny) {
                        continue;
                    }
                    let idx_n = self.idx(nx, ny);
                    let mut n = self.cells[idx_n];

                    if n.elem == Element::Water || n.elem == Element::SaltWater {
                        if n.life < q - 1 {
                            n.life = q - 1;
                        }
                    }
                    if n.elem == Element::Human || n.elem == Element::Zombie {
                        n.elem = Element::Ash;
                        n.life = 0;
                    }

                    self.cells[idx_n] = n;
                }
            }
            let c = &mut self.cells[idx0];
            c.life -= 1;
            if c.life < 0 {
                c.life = 0;
            }
        }
    }

    fn step_gas(&mut self, x: i32, y: i32, updated: &mut [bool]) {
        let idx0 = self.idx(x, y);
        let t = self.cells[idx0].elem;
        let mut moved = false;

        // Try moving up (Hydrogen faster)
        let tries = if t == Element::Hydrogen { 2 } else { 1 };
        for _ in 0..tries {
            if self.in_bounds(x, y - 1)
                && self.cells[self.idx(x, y - 1)].elem == Element::Empty
            {
                let idx_up = self.idx(x, y - 1);
                self.cells.swap(idx0, idx_up);
                updated[idx_up] = true;
                moved = true;
                break;
            }
        }

        // Random sideways drift
        if !moved {
            let mut order = [-1, 1];
            if self.rng.chance(50) {
                order.swap(0, 1);
            }
            for &dx in &order {
                let nx = x + dx;
                let ny = y - if self.rng.chance(50) { 1 } else { 0 };
                if self.in_bounds(nx, ny)
                    && self.cells[self.idx(nx, ny)].elem == Element::Empty
                {
                    let idx_n = self.idx(nx, ny);
                    self.cells.swap(idx0, idx_n);
                    updated[idx_n] = true;
                    moved = true;
                    break;
                }
            }
        }

        // Hydrogen / generic Gas ignite near fire/lava
        if t == Element::Hydrogen || t == Element::Gas {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x + dx;
                    let ny = y + dy;
                    if !self.in_bounds(nx, ny) {
                        continue;
                    }
                    let e = self.cells[self.idx(nx, ny)].elem;
                    if e == Element::Fire || e == Element::Lava {
                        if t == Element::Hydrogen {
                            self.explode(x, y, 4);
                        } else {
                            let c = &mut self.cells[idx0];
                            c.elem = Element::Fire;
                            c.life = 12;
                        }
                    }
                }
            }
        }

        // Chlorine harms plants -> toxic gas
        if t == Element::Chlorine {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = x + dx;
                    let ny = y + dy;
                    if !self.in_bounds(nx, ny) {
                        continue;
                    }
                    let idx_n = self.idx(nx, ny);
                    let n = &mut self.cells[idx_n];
                    if n.elem == Element::Plant && self.rng.chance(35) {
                        n.elem = Element::ToxicGas;
                        n.life = 25;
                    }
                }
            }
        }

        // Lifetime decay & condensation
        let c = &mut self.cells[idx0];
        c.life -= 1;
        if c.life <= 0 {
            match t {
                Element::Steam => {
                    if self.rng.chance(15) {
                        c.elem = Element::Water;
                        c.life = 0;
                    } else {
                        c.elem = Element::Empty;
                        c.life = 0;
                    }
                }
                Element::Smoke => {
                    if self.rng.chance(8) {
                        c.elem = Element::Ash;
                        c.life = 0;
                    } else {
                        c.elem = Element::Empty;
                        c.life = 0;
                    }
                }
                _ => {
                    c.elem = Element::Empty;
                    c.life = 0;
                }
            }
        } else if !moved {
            updated[idx0] = true;
        }
    }

    fn step_fire(&mut self, x: i32, y: i32, updated: &mut [bool]) {
        let idx0 = self.idx(x, y);

        // Chance to flicker upward
        if self.in_bounds(x, y - 1) {
            let idx_up = self.idx(x, y - 1);
            let e_up = self.cells[idx_up].elem;
            if (e_up == Element::Empty || is_gas(e_up)) && self.rng.chance(50) {
                self.cells.swap(idx0, idx_up);
                updated[idx_up] = true;
            }
        }

        // Interact with neighbors
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if !self.in_bounds(nx, ny) {
                    continue;
                }
                let idx_n = self.idx(nx, ny);
                let mut n = self.cells[idx_n];

                if is_flammable(n.elem) && self.rng.chance(40) {
                    if n.elem == Element::Gunpowder {
                        self.explode(nx, ny, 5);
                    } else {
                        n.elem = Element::Fire;
                        n.life = 15 + self.rng.range_i32(0, 10);
                    }
                }
                if n.elem == Element::Water || n.elem == Element::SaltWater {
                    let c = &mut self.cells[idx0];
                    c.elem = Element::Smoke;
                    c.life = 15;
                }
                if n.elem == Element::Wire || n.elem == Element::Metal {
                    if self.rng.chance(5) {
                        n.life = n.life.max(5);
                    }
                }

                self.cells[idx_n] = n;
            }
        }

        // Fire burns out into smoke
        let c = &mut self.cells[idx0];
        c.life -= 1;
        if c.life <= 0 {
            c.elem = Element::Smoke;
            c.life = 15;
        }
        updated[idx0] = true;
    }

    fn step_lightning(&mut self, x: i32, y: i32, updated: &mut [bool]) {
        let idx0 = self.idx(x, y);

        for dy in -2..=2 {
            for dx in -2..=2 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if !self.in_bounds(nx, ny) {
                    continue;
                }
                let idx_n = self.idx(nx, ny);
                let mut n = self.cells[idx_n];
                let e = n.elem;

                if e == Element::Wire || e == Element::Metal {
                    n.life = n.life.max(12);
                }
                if e == Element::Water || e == Element::SaltWater {
                    n.life = n.life.max(8);
                }
                if is_flammable(e) {
                    if e == Element::Gunpowder {
                        self.explode(nx, ny, 6);
                    } else {
                        n.elem = Element::Fire;
                        n.life = 20 + self.rng.range_i32(0, 10);
                    }
                }
                if e == Element::Hydrogen || e == Element::Gas {
                    self.explode(nx, ny, 4);
                }

                self.cells[idx_n] = n;
            }
        }

        let c = &mut self.cells[idx0];
        c.life -= 1;
        if c.life <= 0 {
            c.elem = Element::Empty;
            c.life = 0;
        }
        updated[idx0] = true;
    }

    fn step_human(&mut self, x: i32, y: i32, updated: &mut [bool]) {
        let idx0 = self.idx(x, y);

        // Hazard kills human (including electrified water)
        let mut killed = false;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = x + dx;
                let ny = y + dy;
                if !self.in_bounds(nx, ny) {
                    continue;
                }
                let idx_n = self.idx(nx, ny);
                let n = self.cells[idx_n];
                if is_hazard(n.elem)
                    || ((n.elem == Element::Water || n.elem == Element::SaltWater) && n.life > 0)
                {
                    let c = &mut self.cells[idx0];
                    c.elem = Element::Ash;
                    c.life = 0;
                    killed = true;
                    break;
                }
            }
            if killed {
                break;
            }
        }
        if killed {
            updated[idx0] = true;
            return;
        }

        // Animation tick
        {
            let c = &mut self.cells[idx0];
            c.life += 1;
        }

        // Gravity: only fall through air/gas
        if self.in_bounds(x, y + 1) {
            let idx_b = self.idx(x, y + 1);
            let b = self.cells[idx_b].elem;
            if b == Element::Empty || is_gas(b) {
                self.cells.swap(idx0, idx_b);
                updated[idx_b] = true;
                return;
            }
        }

        // Look for nearest zombie in a radius
        let mut zx = 0;
        let mut zy = 0;
        let mut seen = false;
        for ry in -6..=6 {
            for rx in -6..=6 {
                let nx = x + rx;
                let ny = y + ry;
                if !self.in_bounds(nx, ny) {
                    continue;
                }
                if self.cells[self.idx(nx, ny)].elem == Element::Zombie {
                    zx = nx;
                    zy = ny;
                    seen = true;
                    break;
                }
            }
            if seen {
                break;
            }
        }

        // Attack adjacent zombies
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if !self.in_bounds(nx, ny) {
                    continue;
                }
                let idx_n = self.idx(nx, ny);
                let mut n = self.cells[idx_n];
                if n.elem == Element::Zombie && self.rng.chance(35) {
                    if self.rng.chance(60) {
                        n.elem = Element::Fire;
                        n.life = 10 + self.rng.range_i32(0, 10);
                    } else {
                        n.elem = Element::Ash;
                        n.life = 0;
                    }
                }
                self.cells[idx_n] = n;
            }
        }

        // Movement: run away from zombie if seen
        let mut dir = if self.rng.chance(50) { 1 } else { -1 };
        if seen {
            dir = if zx < x { 1 } else { -1 };
        }

        if !self.try_walk(x, y, x + dir, y) {
            // small jump over obstacle
            if self.in_bounds(x + dir, y - 1)
                && self.cells[self.idx(x + dir, y - 1)].elem == Element::Empty
                && self.cells[self.idx(x, y - 1)].elem == Element::Empty
                && self.rng.chance(70)
            {
                let idx_up = self.idx(x, y - 1);
                self.cells.swap(idx0, idx_up);
            } else {
                let alt_dir = if self.rng.chance(50) { 1 } else { -1 };
                self.try_walk(x, y, x + alt_dir, y);
            }
        }

        updated[idx0] = true;
    }

    fn step_zombie(&mut self, x: i32, y: i32, updated: &mut [bool]) {
        let idx0 = self.idx(x, y);

        // Hazards (including electrified water)
        {
            let mut killed = false;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = x + dx;
                    let ny = y + dy;
                    if !self.in_bounds(nx, ny) {
                        continue;
                    }
                    let idx_n = self.idx(nx, ny);
                    let n = self.cells[idx_n];
                    if is_hazard(n.elem)
                        || ((n.elem == Element::Water || n.elem == Element::SaltWater)
                            && n.life > 0)
                    {
                        let c = &mut self.cells[idx0];
                        c.elem = Element::Fire;
                        c.life = 15;
                        killed = true;
                        break;
                    }
                }
                if killed {
                    break;
                }
            }
            if self.cells[idx0].elem != Element::Zombie {
                updated[idx0] = true;
                return;
            }
        }

        {
            let c = &mut self.cells[idx0];
            c.life += 1;
        }

        // Gravity
        if self.in_bounds(x, y + 1) {
            let idx_b = self.idx(x, y + 1);
            let b = self.cells[idx_b].elem;
            if b == Element::Empty || is_gas(b) {
                self.cells.swap(idx0, idx_b);
                updated[idx_b] = true;
                return;
            }
        }

        // Look for human
        let mut hx = 0;
        let mut hy = 0;
        let mut seen = false;
        for ry in -6..=6 {
            for rx in -6..=6 {
                let nx = x + rx;
                let ny = y + ry;
                if !self.in_bounds(nx, ny) {
                    continue;
                }
                if self.cells[self.idx(nx, ny)].elem == Element::Human {
                    hx = nx;
                    hy = ny;
                    seen = true;
                    break;
                }
            }
            if seen {
                break;
            }
        }

        // Infect / attack adjacent humans
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if !self.in_bounds(nx, ny) {
                    continue;
                }
                let idx_n = self.idx(nx, ny);
                let mut n = self.cells[idx_n];
                if n.elem == Element::Human {
                    if self.rng.chance(70) {
                        n.elem = Element::Zombie;
                        n.life = 0;
                    } else {
                        n.elem = Element::Fire;
                        n.life = 10;
                    }
                }
                self.cells[idx_n] = n;
            }
        }

        // Move towards human or wander
        let mut dir = if self.rng.chance(50) { 1 } else { -1 };
        if seen {
            dir = if hx > x { 1 } else { -1 };
        }

        if !self.try_walk(x, y, x + dir, y) {
            if self.in_bounds(x + dir, y - 1)
                && self.cells[self.idx(x + dir, y - 1)].elem == Element::Empty
                && self.cells[self.idx(x, y - 1)].elem == Element::Empty
                && self.rng.chance(70)
            {
                let idx_up = self.idx(x, y - 1);
                self.cells.swap(idx0, idx_up);
            } else {
                let alt_dir = if self.rng.chance(50) { 1 } else { -1 };
                self.try_walk(x, y, x + alt_dir, y);
            }
        }

        updated[idx0] = true;
    }

    fn step_wet_dirt(&mut self, x: i32, y: i32, updated: &mut [bool]) {
        let idx0 = self.idx(x, y);
        let mut near_water = false;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = x + dx;
                let ny = y + dy;
                if !self.in_bounds(nx, ny) {
                    continue;
                }
                let e = self.cells[self.idx(nx, ny)].elem;
                if e == Element::Water || e == Element::SaltWater {
                    near_water = true;
                    break;
                }
            }
            if near_water {
                break;
            }
        }

        if !near_water {
            let c = &mut self.cells[idx0];
            c.life -= 1;
            if c.life <= 0 {
                c.elem = Element::Dirt;
                c.life = 0;
            }
        }

        updated[idx0] = true;
    }

    fn step_plant_like(&mut self, x: i32, y: i32, updated: &mut [bool]) {
        let idx0 = self.idx(x, y);
        let t = self.cells[idx0].elem;

        // Burning check
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if !self.in_bounds(nx, ny) {
                    continue;
                }
                let e = self.cells[self.idx(nx, ny)].elem;
                if e == Element::Fire || e == Element::Lava {
                    let c = &mut self.cells[idx0];
                    c.elem = Element::Fire;
                    c.life = 20;
                }
            }
        }

        if self.cells[idx0].elem == Element::Fire {
            updated[idx0] = true;
            return;
        }

        if t == Element::Plant {
            // Grows upward from WetDirt
            let good_soil = self.in_bounds(x, y + 1)
                && self.cells[self.idx(x, y + 1)].elem == Element::WetDirt;
            if good_soil && self.rng.chance(2) {
                let gx = x;
                let gy = y - 1;
                if self.in_bounds(gx, gy)
                    && self.cells[self.idx(gx, gy)].elem == Element::Empty
                {
                    let idx_g = self.idx(gx, gy);
                    self.cells[idx_g].elem = Element::Plant;
                    self.cells[idx_g].life = 0;
                }
            }
        } else {
            // Seaweed: grows upward while underwater
            let underwater = self.in_bounds(x, y - 1)
                && (self.cells[self.idx(x, y - 1)].elem == Element::Water
                    || self.cells[self.idx(x, y - 1)].elem == Element::SaltWater);
            let is_top = !self.in_bounds(x, y - 1)
                || self.cells[self.idx(x, y - 1)].elem != Element::Seaweed;
            if underwater && is_top && self.rng.chance(2) {
                let gy = y - 1;
                if self.in_bounds(x, gy) {
                    let idx_g = self.idx(x, gy);
                    let e = self.cells[idx_g].elem;
                    if e == Element::Water || e == Element::SaltWater {
                        self.cells[idx_g].elem = Element::Seaweed;
                        self.cells[idx_g].life = 0;
                    }
                }
            }
        }

        updated[idx0] = true;
    }

    fn step_burnable_solid(&mut self, x: i32, y: i32, updated: &mut [bool]) {
        let idx0 = self.idx(x, y);
        let t = self.cells[idx0].elem;

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if !self.in_bounds(nx, ny) {
                    continue;
                }
                let e = self.cells[self.idx(nx, ny)].elem;
                if e == Element::Fire || e == Element::Lava {
                    let c = &mut self.cells[idx0];
                    c.elem = Element::Fire;
                    c.life = if t == Element::Coal { 35 } else { 25 };
                }
            }
        }

        updated[idx0] = true;
    }

    fn step_gunpowder(&mut self, x: i32, y: i32, updated: &mut [bool]) {
        let idx0 = self.idx(x, y);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if !self.in_bounds(nx, ny) {
                    continue;
                }
                let e = self.cells[self.idx(nx, ny)].elem;
                if e == Element::Fire || e == Element::Lava {
                    self.explode(x, y, 5);
                    break;
                }
            }
        }
        updated[idx0] = true;
    }

    fn step_conductor(&mut self, x: i32, y: i32, updated: &mut [bool]) {
        let idx0 = self.idx(x, y);
        let life = self.cells[idx0].life;
        if life > 0 {
            let q = life;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x + dx;
                    let ny = y + dy;
                    if !self.in_bounds(nx, ny) {
                        continue;
                    }
                    let idx_n = self.idx(nx, ny);
                    let mut n = self.cells[idx_n];

                    if n.elem == Element::Wire || n.elem == Element::Metal {
                        if n.life < q - 1 {
                            n.life = q - 1;
                        }
                    }
                    if n.elem == Element::Water || n.elem == Element::SaltWater {
                        if n.life < q - 1 {
                            n.life = q - 1;
                        }
                    }
                    if is_flammable(n.elem) && self.rng.chance(15) {
                        if n.elem == Element::Gunpowder {
                            self.explode(nx, ny, 5);
                        } else {
                            n.elem = Element::Fire;
                            n.life = 15 + self.rng.range_i32(0, 10);
                        }
                    }
                    if n.elem == Element::Hydrogen || n.elem == Element::Gas {
                        if self.rng.chance(35) {
                            self.explode(nx, ny, 4);
                        }
                    }

                    self.cells[idx_n] = n;
                }
            }
            let c = &mut self.cells[idx0];
            c.life -= 1;
            if c.life < 0 {
                c.life = 0;
            }
        }

        updated[idx0] = true;
    }

    fn step_ice(&mut self, x: i32, y: i32, updated: &mut [bool]) {
        let idx0 = self.idx(x, y);
        let mut melt = false;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = x + dx;
                let ny = y + dy;
                if !self.in_bounds(nx, ny) {
                    continue;
                }
                let e = self.cells[self.idx(nx, ny)].elem;
                if e == Element::Fire || e == Element::Lava || e == Element::Steam {
                    if self.rng.chance(25) {
                        melt = true;
                        break;
                    }
                }
            }
            if melt {
                break;
            }
        }

        if melt {
            let c = &mut self.cells[idx0];
            c.elem = Element::Water;
            c.life = 0;
        }

        updated[idx0] = true;
    }

    /// Try to walk from (x, y) to (tx, ty) if destination is empty or gas.
    fn try_walk(&mut self, x: i32, y: i32, tx: i32, ty: i32) -> bool {
        if !self.in_bounds(tx, ty) {
            return false;
        }
        let idx_from = self.idx(x, y);
        let idx_to = self.idx(tx, ty);
        let dst = self.cells[idx_to].elem;
        if dst == Element::Empty || is_gas(dst) {
            self.cells.swap(idx_from, idx_to);
            true
        } else {
            false
        }
    }
}

// ===== Element classification & meta =====

fn is_sand_like(e: Element) -> bool {
    matches!(e, Element::Sand | Element::Gunpowder | Element::Ash | Element::Snow)
}

fn is_liquid(e: Element) -> bool {
    matches!(
        e,
        Element::Water
            | Element::SaltWater
            | Element::Oil
            | Element::Ethanol
            | Element::Acid
            | Element::Lava
            | Element::Mercury
    )
}

fn is_gas(e: Element) -> bool {
    matches!(
        e,
        Element::Smoke
            | Element::Steam
            | Element::Gas
            | Element::ToxicGas
            | Element::Hydrogen
            | Element::Chlorine
    )
}

fn is_flammable(e: Element) -> bool {
    matches!(
        e,
        Element::Wood
            | Element::Plant
            | Element::Oil
            | Element::Ethanol
            | Element::Gunpowder
            | Element::Coal
            | Element::Seaweed
    )
}

fn is_dissolvable(e: Element) -> bool {
    matches!(
        e,
        Element::Sand
            | Element::Stone
            | Element::Glass
            | Element::Wood
            | Element::Plant
            | Element::Metal
            | Element::Wire
            | Element::Ash
            | Element::Coal
            | Element::Seaweed
            | Element::Dirt
            | Element::WetDirt
    )
}

/// Relative density for liquids and gases (same values as C++ engine).
fn density(e: Element) -> i32 {
    match e {
        Element::Ethanol => 85,
        Element::Oil => 90,
        Element::Gas | Element::Hydrogen => 1,
        Element::Steam => 2,
        Element::Smoke => 3,
        Element::Chlorine => 5,
        Element::Water => 100,
        Element::SaltWater => 103,
        Element::Acid => 110,
        Element::Lava => 160,
        Element::Mercury => 200,
        _ => 999,
    }
}

fn is_hazard(e: Element) -> bool {
    matches!(
        e,
        Element::Fire
            | Element::Lava
            | Element::Acid
            | Element::ToxicGas
            | Element::Chlorine
            | Element::Lightning
    )
}

// ===== Public helpers for UI layers =====

/// Human-readable element name (same text as C++ version).
pub fn name_of(e: Element) -> &'static str {
    match e {
        Element::Empty => "Empty",
        Element::Sand => "Sand",
        Element::Gunpowder => "Gunpowder",
        Element::Ash => "Ash",
        Element::Snow => "Snow",
        Element::Water => "Water",
        Element::SaltWater => "Salt Water",
        Element::Oil => "Oil",
        Element::Ethanol => "Ethanol",
        Element::Acid => "Acid",
        Element::Lava => "Lava",
        Element::Mercury => "Mercury",
        Element::Stone => "Stone",
        Element::Glass => "Glass",
        Element::Wall => "Wall",
        Element::Wood => "Wood",
        Element::Plant => "Plant",
        Element::Metal => "Metal",
        Element::Wire => "Wire",
        Element::Ice => "Ice",
        Element::Coal => "Coal",
        Element::Dirt => "Dirt",
        Element::WetDirt => "Wet Dirt",
        Element::Seaweed => "Seaweed",
        Element::Smoke => "Smoke",
        Element::Steam => "Steam",
        Element::Gas => "Gas",
        Element::ToxicGas => "Toxic Gas",
        Element::Hydrogen => "Hydrogen",
        Element::Chlorine => "Chlorine",
        Element::Fire => "Fire",
        Element::Lightning => "Lightning",
        Element::Human => "Human",
        Element::Zombie => "Zombie",
    }
}

/// Simple numeric "palette index" the frontend can map to colors.
/// Values mirror the C++ classic ncurses color pairs (1..9).
pub fn color_of(e: Element, life: i32) -> u8 {
    // Electrified water pulses are yellow (9)
    if (e == Element::Water || e == Element::SaltWater) && life > 0 {
        return 9;
    }

    match e {
        Element::Empty => 1,
        // yellow-ish powders / dirt
        Element::Sand | Element::Gunpowder | Element::Snow | Element::Dirt => 2,
        // cyan water-ish
        Element::Water
        | Element::SaltWater
        | Element::Steam
        | Element::Ice
        | Element::Ethanol => 3,
        // white solids
        Element::Stone
        | Element::Glass
        | Element::Wall
        | Element::Metal
        | Element::Wire
        | Element::Coal
        | Element::WetDirt => 4,
        // green stuff & humans
        Element::Wood | Element::Plant | Element::Seaweed | Element::Human => 5,
        // red danger
        Element::Fire | Element::Lava | Element::Zombie => 6,
        // magenta haze
        Element::Smoke | Element::Ash | Element::Gas | Element::Hydrogen => 7,
        // blue heavy liquids
        Element::Oil | Element::Mercury => 8,
        // green/yellow chem/bolt
        Element::Acid | Element::ToxicGas | Element::Chlorine | Element::Lightning => 9,
    }
}

/// ASCII glyphs for drawing in a text UI.
pub fn glyph_of(e: Element, life: i32) -> char {
    match e {
        Element::Empty => ' ',
        Element::Sand => '.',
        Element::Gunpowder => '%',
        Element::Ash => ';',
        Element::Snow => ',',
        Element::Water => '~',
        Element::SaltWater => ':',
        Element::Oil => 'o',
        Element::Ethanol => 'e',
        Element::Acid => 'a',
        Element::Lava => 'L',
        Element::Mercury => 'm',
        Element::Stone => '#',
        Element::Glass => '=',
        Element::Wall => '@',
        Element::Wood => 'w',
        Element::Plant => 'p',
        Element::Seaweed => 'v',
        Element::Metal => 'M',
        Element::Wire => '-',
        Element::Ice => 'I',
        Element::Coal => 'c',
        Element::Dirt => 'd',
        Element::WetDirt => 'D',
        Element::Smoke => '^',
        Element::Steam => '"',
        Element::Gas => '`',
        Element::ToxicGas => 'x',
        Element::Hydrogen => '\'',
        Element::Chlorine => 'X',
        Element::Fire => '*',
        Element::Lightning => '|',
        Element::Human => {
            // little stick-figure animation
            if (life / 6) % 2 != 0 {
                'y'
            } else {
                'Y'
            }
        }
        Element::Zombie => {
            if (life / 6) % 2 != 0 {
                't'
            } else {
                'T'
            }
        }
    }
}
