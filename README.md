# ternary-membrane

Compartment transport dynamics with ternary concentrations. Diffusion, osmosis, active transport, gated channels, and full simulation.

Biological membranes are selective barriers: they let some molecules through, block others, and actively pump against gradients using energy. This crate models that exact mechanism with ternary concentrations {-1, 0, +1}. Compartments hold solute concentrations. Membranes control permeability. Channels provide selective, gated transport. And the simulation engine runs it all forward in discrete steps.

Built for `#![no_std]` with only `alloc`. Runs anywhere you can allocate a `Vec`.

## Why this exists

Multi-agent systems, neural networks, and economic markets all exhibit *membrane-like* dynamics: selective flow between regions, equilibrium-seeking through diffusion, and energy-consuming active transport that maintains concentration gradients against entropy. This crate captures that abstraction in a reusable simulation engine.

The ternary constraint ({-1, 0, +1} concentrations) makes the dynamics tractable and interpretable. You can reason about diffusion in a few lines of code instead of solving PDEs. The simulation converges fast because the state space is small.

## The key insight

Diffusion in ternary space is trivial: if the concentration difference across a membrane is non-zero, move one unit from high to low. That's it. No floating-point diffusion coefficients, no stability constraints, no CFL conditions. One comparison, one decrement, one increment, one clamp.

This simplicity extends to every operation:
- **Osmotic pressure** = difference in total concentration (one subtraction)
- **Equilibrium** = all permeable concentrations match (one comparison per solute)
- **Active transport** = move one unit against gradient, if energy > 0

The biological accuracy comes from the *structure* (compartments, membranes, channels), not the math.

## Quick start

```rust
use ternary_membrane::*;

// Create a system with 2 compartments, 1 solute type
let mut sys = MembraneSystem::new(2, 1);

// Set initial concentrations
sys.compartments[0].concentrations[0] = 1;  // high concentration
sys.compartments[1].concentrations[0] = -1; // low concentration

// Add a permeable membrane between them
sys.add_membrane(0, 1);

// Run simulation: diffusion will equalize concentrations
let history = sys.simulate(5);

// After enough steps, concentrations converge toward 0
println!("Final state: {:?}", sys.compartments[0].concentrations);
```

## API reference

### Compartment

A container with per-solute concentrations:

```rust
let mut c = Compartment::new(id, volume, n_solutes);
c.concentrations[0] = 1;           // set solute 0 to +1
c.total_concentration();            // → i8, clamped sum
c.is_empty();                       // → bool, all concentrations 0?
```

### Membrane

A selective barrier between two compartments:

```rust
let mut mem = Membrane::new(left_id, right_id, n_solutes);
mem.impermeable_to(solute_idx);    // block a solute
mem.selective(solute_idx, perm);   // set permeability: -1, 0, or 1
```

### Channel

A selective, optionally gated, transport pathway:

```rust
let ch = Channel::new(left, right, solute_idx);          // always open
let mut gated = Channel::gated_channel(left, right, idx); // starts closed
gated.update_gate(concentration, threshold);               // opens when conc >= threshold
```

### Standalone functions

```rust
// Diffuse solutes across a membrane (high → low)
diffuse(&mut left, &mut right, &membrane);

// Osmotic pressure (concentration difference)
osmotic_pressure(&left, &right);  // → i8

// Pump against gradient using energy
active_transport(&mut src, &mut dst, solute, direction, energy);

// Check equilibrium
is_equilibrium(&left, &right, &membrane);  // → bool
```

### MembraneSystem

Full simulation engine:

```rust
let mut sys = MembraneSystem::new(n_compartments, n_solutes);
sys.add_membrane(left, right);
sys.add_channel(left, right, solute);

sys.step();                    // one simulation step
let history = sys.simulate(100);  // run N steps, return concentration history
```

Each step:
1. Diffusion through all membranes (parallel per membrane)
2. Channel transport through all open channels

## Architecture

```
┌──────────────┐    Membrane     ┌──────────────┐
│  Compartment │◄───────────────►│  Compartment │
│  [0, 1, -1]  │   permeability  │  [1, 0,  0]  │
└──────────────┘                 └──────────────┘
        ▲                               ▲
        │         ┌───────────┐         │
        └─────────│  Channel  │─────────┘
                  │  gated?   │
                  └───────────┘
```

The `MembraneSystem` owns all compartments, membranes, and channels. Each `step()` clones the membrane and channel lists for borrow-checker safety, then applies diffusion and channel transport in sequence.

Concentrations are clamped to {-1, 0, +1} after every operation. This means diffusion can only equalize—it can never overshoot. Active transport also respects the clamp, preventing concentrations from exceeding the ternary range.

## Real-world example: Signal cascade

```rust
use ternary_membrane::*;

// Model a signal cascade: receptor → cytoplasm → nucleus
let mut sys = MembraneSystem::new(3, 2);  // 3 compartments, 2 solute types

// Receptor receives signal
sys.compartments[0].concentrations[0] = 1;  // signal molecule at +1

// Cytoplasm at rest
sys.compartments[1].concentrations[0] = 0;
sys.compartments[1].concentrations[1] = -1; // inhibitor at -1

// Nucleus at rest
sys.compartments[2].concentrations[0] = 0;

// Receptor → cytoplasm membrane (permeable to signal only)
sys.add_membrane(0, 1);
// Make solute 1 impermeable at this membrane
sys.membranes[0].impermeable_to(1);

// Cytoplasm → nucleus membrane with a gated channel
sys.add_membrane(1, 2);
let mut gate = Channel::gated_channel(1, 2, 0); // gated channel for signal
gate.update_gate(1, 0);  // opens when signal concentration >= 0
sys.channels.push(gate);

// Run the cascade
let history = sys.simulate(10);

// Watch the signal propagate:
// Step 0: [1,0,0] → [0,-1,0] → [0,0,0]
// Step 1: signal diffuses into cytoplasm
// Step 2: cytoplasm signal opens gated channel
// Step 3: signal enters nucleus
for (step, state) in history.iter().enumerate() {
    let nuclear_signal = state[2][0];
    if nuclear_signal > 0 {
        println!("Step {}: Signal reached nucleus!", step);
        break;
    }
}
```

## Ecosystem connections

- **ternary-gauge** — monitor compartment concentrations over time to detect when equilibrium is reached or when the system is oscillating
- **ternary-warp** — transform concentration sequences (smooth noisy diffusion, differentiate to find concentration fronts)
- **ternary-resilience** — the compartment membrane graph *is* a network; use resilience analysis to check if removing a membrane disconnects any compartments

## Open questions

- **Reaction kinetics**: Currently, solutes only move. Adding reactions (A + B → C) would make the model much more powerful for biochemical simulations.
- **Variable volumes**: All compartments have unit volume. Variable volumes would affect diffusion rates and concentration calculations.
- **Stochastic transport**: Current diffusion is deterministic (always moves one unit). A stochastic version would randomly decide whether to transport based on the gradient magnitude.

## Stats

| Metric | Value |
|--------|-------|
| Tests | 13 |
| Lines of code | 335 |
| Public API surface | 22 items |
| License | Apache-2.0 |
| Unsafe | 0 |
| `no_std` | Yes |

## Installation

```toml
[dependencies]
ternary-membrane = "0.1.0"
```

## License

Apache-2.0
