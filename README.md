# ternary-membrane

Membrane computing with ternary object concentrations {-1, 0, +1} — diffusion, osmosis, ion channels, active transport, and multi-compartment simulation.

## Background

**Membrane computing** (Păun, 1998) is a computational paradigm inspired by the structure and function of biological cells. A cell is partitioned by membranes into compartments, each containing objects (molecules, ions) that evolve according to local rules. Objects cross membranes via diffusion (passive movement down concentration gradients), active transport (energy-consuming movement against gradients), and channel-mediated transport (selective pores for specific molecules).

The `ternary-membrane` crate models this system with concentrations clamped to {-1, 0, +1} per solute per compartment. This ternary restriction transforms continuous differential equations into discrete state transitions:

- **Diffusion**: Fick's law (J = −D · ∂c/∂x) becomes: if left concentration > right concentration, move one unit of solute from left to right. In ternary, this means the gradient can only be ±1 or 0, producing single-step equalization.
- **Osmosis**: The osmotic pressure π = c_left − c_right (van 't Hoff equation, simplified). In ternary, this is the difference of two ternary values, yielding {-2, −1, 0, 1, 2}.
- **Active transport**: Moving solute against its gradient requires energy. The `energy` parameter (ternary: 0 or 1) gates whether transport occurs. With energy=0, no transport happens—conserving the gradient.
- **Ion channels**: Selective pores that allow specific solutes through. Channels can be **gated** (open/close based on concentration thresholds, modeling voltage-gated or ligand-gated channels) or **constitutively open**.

The ternary model trades quantitative precision for computational simplicity and structural clarity. Each diffusion step is O(n_solutes) per membrane; each simulation step is O(membranes × solutes). The state space per compartment is 3^n_solutes, bounded and enumerable for small n.

## How It Works

### Architecture

Three core types:

- **`Compartment`**: Has `id`, `volume`, and a `Vec<i8>` of concentrations (one per solute, each in {-1, 0, +1}). Methods: `total_concentration()` (clamped sum), `is_empty()`.
- **`Membrane`**: Connects two compartments (`left`, `right`) with per-solute permeability {-1, 0, +1}. Permeability 0 = impermeable; ±1 = permeable (with potential directional semantics).
- **`Channel`**: A selective pore connecting two compartments for a single solute. Can be open/closed and optionally gated (opens when concentration exceeds a threshold).
- **`MembraneSystem`**: A multi-compartment simulator with compartments, membranes, and channels. Runs discrete time-step simulation.

### Diffusion

`diffuse(left, right, membrane)`: For each solute with non-zero permeability, compare concentrations. If left > right, transfer one unit from left to right (clamped). If right > left, reverse. This is a single-step gradient descent toward equilibrium.

### Active Transport

`active_transport(compartment, other, solute, direction, energy)`: If energy > 0, move one unit of solute in the specified direction (pump out: compartment → other; pump in: other → compartment), clamped to {-1, 0, +1}.

### Equilibrium Check

`is_equilibrium(left, right, membrane)`: For all permeable solutes, left concentration equals right concentration.

### System Simulation

`MembraneSystem::step()`: Process all membranes (diffusion), then all open channels (selective transport). `simulate(n)`: Run n steps, returning the full concentration history for visualization and analysis.

## Experimental Results

All 13 unit tests pass:

| Test | Result | Observation |
|------|--------|-------------|
| `test_compartment_new` | ✅ | Fresh compartment: empty, total_concentration = 0 |
| `test_compartment_concentration` | ✅ | [+1, −1, 0]: total = 0 (cancellation), not empty |
| `test_diffusion_equalizes` | ✅ | Left=[+1], Right=[0]: after diffusion steps, concentrations converge |
| `test_diffusion_impermeable` | ✅ | Left=[+1], impermeable membrane: concentration unchanged |
| `test_osmotic_pressure` | ✅ | Left=[+1], Right=[0]: osmotic pressure = +1 |
| `test_active_transport_pump` | ✅ | Pump solute from other to compartment: concentrations swap |
| `test_active_transport_no_energy` | ✅ | Energy=0: no transport occurs |
| `test_equilibrium` | ✅ | Two empty compartments: at equilibrium |
| `test_not_equilibrium` | ✅ | Left=[+1], Right=[0]: not at equilibrium |
| `test_system_new` | ✅ | 3-compartment system created correctly |
| `test_system_simulate` | ✅ | 2-compartment diffusion: 5-step history recorded |
| `test_channel` | ✅ | Constitutive channel: open=true, gated=false |
| `test_gated_channel` | ✅ | Gated channel: initially closed; opens when concentration ≥ threshold |

The `test_active_transport_pump` test demonstrates energy-dependent transport: pumping solute from compartment with concentration 0 to one with concentration 1 requires energy and moves *against* the gradient—impossible via diffusion alone.

The `test_gated_channel` test shows voltage-like gating: the channel opens only when concentration exceeds the threshold, modeling biological voltage-gated ion channels in a ternary regime.

## Impact of Ternary {-1, 0, +1}

The ternary concentration model provides a **qualitative** representation of chemical states:

- **+1**: High concentration / excitatory presence. The solute is abundant and driving reactions.
- **0**: Baseline / absent. The solute is at equilibrium or not present.
- **−1**: Low concentration / inhibitory presence. The solute is depleted or actively antagonistic.

This qualitative model is sufficient for many systems where the exact concentration matters less than the *direction* of the gradient. It also enables direct composition with other ternary systems—the output of a ternary membrane system can feed into ternary decision logic, ternary version vectors, or ternary PageRank without conversion.

## Use Cases

1. **Biological Cell Modeling**: Simulate simplified cell dynamics—ion gradients across cell membranes, sodium-potassium pump (active transport), and neurotransmitter-gated ion channels. The ternary model captures qualitative behavior (depolarization/repolarization/hyperpolarization) without continuous differential equations.

2. **Chemical Reaction Network Analysis**: Model a network of reaction vessels connected by semi-permeable membranes. Diffusion equalizes concentrations; active transport maintains gradients. Equilibrium analysis reveals steady-state distributions.

3. **Resource Flow in Distributed Systems**: Analogize compartments to servers and membranes to network links with bandwidth limits (permeability). Solutes represent resources (memory, compute, data). Diffusion is load balancing; active transport is priority scheduling.

4. **Environmental Contaminant Modeling**: Compartments = environmental zones (air, water, soil). Membranes = interfaces (air-water, water-soil) with different permeability for each contaminant. Simulate diffusion of pollutants and active remediation (pumping contaminants out).

5. **GPU Memory Hierarchy Simulation**: Compartments = GPU memory levels (registers, shared memory, L2, DRAM). Membranes = bus interfaces with bandwidth limits. Channels = DMA engines. Diffusion models data migration; active transport models prefetching.

## Open Questions

1. **Multi-Solute Interactions**: The current model treats solutes independently—diffusion of solute A doesn't affect solute B. Real chemical systems have coupled reactions (e.g., sodium-potassium exchange). How should ternary membrane computing model coupled solute dynamics?

2. **Non-Integer Permeability**: The current permeability is ternary {−1, 0, +1}. Could partial permeability (e.g., probability of transport per step) produce more realistic dynamics while staying within a discrete framework?

3. **Membrane Growth and Division**: Biological cells grow and divide, creating new compartments. Extending the model to dynamic membrane topology (adding/removing compartments and membranes during simulation) would enable self-replicating system modeling.

## Connection to Oxide Stack

Within the five-layer Oxide ternary architecture:

- **Layer 1 (Ternary Genome)**: Solute concentrations {-1, 0, +1} are genome bases, encoding the membrane system's state as genetic information. Mutations that change permeability or channel gating are genomic variations affecting phenotype.
- **Layer 2 (Cellular Computation)**: Each compartment is literally a cell in the membrane computing sense. Diffusion and active transport are the computational rules—ternary state transitions governed by local membrane properties.
- **Layer 3 (Organism Behavior)**: The multi-compartment system as a whole exhibits organism-level behavior—homeostasis (equilibrium seeking), response to perturbation (gradient restoration), and gated responses to environmental stimuli.
- **Layer 4 (Population Dynamics)**: Multiple membrane systems (organisms) interacting through shared membranes exchange solutes, creating population-level chemical communication. The ternary gradient structure determines which organisms are net donors vs. recipients.
- **Layer 5 (Ecosystem)**: The ecosystem's compartment structure—its membrane topology, permeability profile, and channel gating—defines the landscape of resource flow. Changes at this level (new membranes, altered permeability) represent ecosystem-level evolution.
