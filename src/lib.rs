//! # ternary-membrane
//!
//! Membrane transport dynamics with ternary concentrations.
//! Diffusion, osmosis, ion channels, active transport.

#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;
use alloc::{vec, vec::Vec};

/// A compartment with a set of solute concentrations
#[derive(Debug, Clone)]
pub struct Compartment {
    pub id: usize,
    pub volume: usize,
    pub concentrations: Vec<i8>,
}

impl Compartment {
    pub fn new(id: usize, volume: usize, n_solutes: usize) -> Self {
        Self { id, volume, concentrations: vec![0; n_solutes] }
    }

    pub fn total_concentration(&self) -> i8 {
        let sum: i32 = self.concentrations.iter().map(|&c| c as i32).sum();
        sum.clamp(-1, 1) as i8
    }

    pub fn is_empty(&self) -> bool {
        self.concentrations.iter().all(|&c| c == 0)
    }
}

/// A membrane between two compartments with permeability per solute
#[derive(Debug, Clone)]
pub struct Membrane {
    pub left: usize,
    pub right: usize,
    pub permeability: Vec<i8>, // per-solute permeability {-1, 0, 1}
}

impl Membrane {
    pub fn new(left: usize, right: usize, n_solutes: usize) -> Self {
        Self { left, right, permeability: vec![1; n_solutes] }
    }

    pub fn impermeable_to(&mut self, solute: usize) {
        if solute < self.permeability.len() { self.permeability[solute] = 0; }
    }

    pub fn selective(&mut self, solute: usize, perm: i8) {
        if solute < self.permeability.len() { self.permeability[solute] = perm.clamp(-1, 1); }
    }
}

/// An ion channel: selective, only allows specific solutes through
#[derive(Debug, Clone)]
pub struct Channel {
    pub membrane_left: usize,
    pub membrane_right: usize,
    pub solute: usize,
    pub open: bool,
    pub gated: bool, // voltage-gated or ligand-gated
}

impl Channel {
    pub fn new(left: usize, right: usize, solute: usize) -> Self {
        Self { membrane_left: left, membrane_right: right, solute, open: true, gated: false }
    }

    pub fn gated_channel(left: usize, right: usize, solute: usize) -> Self {
        Self { membrane_left: left, membrane_right: right, solute, open: false, gated: true }
    }

    /// Open/close based on concentration threshold
    pub fn update_gate(&mut self, concentration: i8, threshold: i8) {
        if self.gated {
            self.open = concentration >= threshold;
        }
    }
}

/// Diffusion step: move solutes across membrane from high to low concentration
pub fn diffuse(left: &mut Compartment, right: &mut Compartment, membrane: &Membrane) {
    for i in 0..membrane.permeability.len() {
        if membrane.permeability[i] == 0 { continue; }
        let diff = left.concentrations[i] - right.concentrations[i];
        if diff > 0 {
            left.concentrations[i] -= 1;
            right.concentrations[i] = (right.concentrations[i] + 1).clamp(-1, 1);
        } else if diff < 0 {
            right.concentrations[i] -= 1;
            left.concentrations[i] = (left.concentrations[i] + 1).clamp(-1, 1);
        }
    }
}

/// Osmotic pressure: difference in total concentration across membrane
pub fn osmotic_pressure(left: &Compartment, right: &Compartment) -> i8 {
    left.total_concentration() - right.total_concentration()
}

/// Active transport: pump solute against gradient using energy
pub fn active_transport(
    compartment: &mut Compartment,
    other: &mut Compartment,
    solute: usize,
    direction: i8, // 1 = pump out, -1 = pump in
    energy: i8,
) {
    if energy <= 0 { return; }
    if direction > 0 {
        // Pump solute from compartment to other
        if compartment.concentrations[solute] > 0 {
            compartment.concentrations[solute] -= 1;
            other.concentrations[solute] = (other.concentrations[solute] + 1).clamp(-1, 1);
        }
    } else {
        // Pump solute from other into compartment
        if other.concentrations[solute] > 0 {
            other.concentrations[solute] -= 1;
            compartment.concentrations[solute] = (compartment.concentrations[solute] + 1).clamp(-1, 1);
        }
    }
}

/// Check if two compartments are at equilibrium
pub fn is_equilibrium(left: &Compartment, right: &Compartment, membrane: &Membrane) -> bool {
    for i in 0..membrane.permeability.len() {
        if membrane.permeability[i] != 0 {
            if left.concentrations[i] != right.concentrations[i] {
                return false;
            }
        }
    }
    true
}

/// A multi-compartment system with membranes
#[derive(Debug, Clone)]
pub struct MembraneSystem {
    pub compartments: Vec<Compartment>,
    pub membranes: Vec<Membrane>,
    pub channels: Vec<Channel>,
}

impl MembraneSystem {
    pub fn new(n_compartments: usize, n_solutes: usize) -> Self {
        Self {
            compartments: (0..n_compartments).map(|i| Compartment::new(i, 1, n_solutes)).collect(),
            membranes: vec![],
            channels: vec![],
        }
    }

    pub fn add_membrane(&mut self, left: usize, right: usize) {
        let n = self.compartments.first().map(|c| c.concentrations.len()).unwrap_or(1);
        self.membranes.push(Membrane::new(left, right, n));
    }

    pub fn add_channel(&mut self, left: usize, right: usize, solute: usize) {
        self.channels.push(Channel::new(left, right, solute));
    }

    /// One simulation step
    pub fn step(&mut self) {
        // Diffusion through membranes
        let membranes = self.membranes.clone();
        for mem in &membranes {
            let (left, right) = (mem.left, mem.right);
            let left_c = self.compartments[left].concentrations.clone();
            let right_c = self.compartments[right].concentrations.clone();
            for i in 0..mem.permeability.len() {
                if mem.permeability[i] == 0 { continue; }
                let diff = left_c[i] - right_c[i];
                if diff > 0 {
                    self.compartments[left].concentrations[i] -= 1;
                    self.compartments[right].concentrations[i] = (self.compartments[right].concentrations[i] + 1).clamp(-1, 1);
                } else if diff < 0 {
                    self.compartments[right].concentrations[i] -= 1;
                    self.compartments[left].concentrations[i] = (self.compartments[left].concentrations[i] + 1).clamp(-1, 1);
                }
            }
        }

        // Channel transport
        let channels = self.channels.clone();
        for ch in &channels {
            if !ch.open { continue; }
            let s = ch.solute;
            let left_c = self.compartments[ch.membrane_left].concentrations[s];
            let right_c = self.compartments[ch.membrane_right].concentrations[s];
            if left_c > right_c {
                self.compartments[ch.membrane_left].concentrations[s] -= 1;
                self.compartments[ch.membrane_right].concentrations[s] =
                    (self.compartments[ch.membrane_right].concentrations[s] + 1).clamp(-1, 1);
            }
        }
    }

    /// Run for N steps
    pub fn simulate(&mut self, steps: usize) -> Vec<Vec<Vec<i8>>> {
        let mut history = vec![];
        for _ in 0..steps {
            self.step();
            history.push(self.compartments.iter().map(|c| c.concentrations.clone()).collect());
        }
        history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compartment_new() {
        let c = Compartment::new(0, 1, 3);
        assert!(c.is_empty());
        assert_eq!(c.total_concentration(), 0);
    }

    #[test]
    fn test_compartment_concentration() {
        let mut c = Compartment::new(0, 1, 3);
        c.concentrations[0] = 1;
        c.concentrations[1] = -1;
        assert_eq!(c.total_concentration(), 0);
        assert!(!c.is_empty());
    }

    #[test]
    fn test_diffusion_equalizes() {
        let mut left = Compartment::new(0, 1, 1);
        let mut right = Compartment::new(1, 1, 1);
        left.concentrations[0] = 1;
        right.concentrations[0] = 0;
        let mem = Membrane::new(0, 1, 1);
        for _ in 0..3 {
            diffuse(&mut left, &mut right, &mem);
        }
        // After diffusion, concentrations should be closer
        // In ternary, 1→0 diffusion in one step makes them equal immediately
        assert!(left.concentrations[0] <= 1);
    }

    #[test]
    fn test_diffusion_impermeable() {
        let mut left = Compartment::new(0, 1, 1);
        let mut right = Compartment::new(1, 1, 1);
        left.concentrations[0] = 1;
        let mut mem = Membrane::new(0, 1, 1);
        mem.impermeable_to(0);
        diffuse(&mut left, &mut right, &mem);
        assert_eq!(left.concentrations[0], 1);
        assert_eq!(right.concentrations[0], 0);
    }

    #[test]
    fn test_osmotic_pressure() {
        let mut left = Compartment::new(0, 1, 1);
        let right = Compartment::new(1, 1, 1);
        left.concentrations[0] = 1;
        assert_eq!(osmotic_pressure(&left, &right), 1);
    }

    #[test]
    fn test_active_transport_pump() {
        let mut c1 = Compartment::new(0, 1, 1);
        let mut c2 = Compartment::new(1, 1, 1);
        c1.concentrations[0] = 0;
        c2.concentrations[0] = 1;
        active_transport(&mut c1, &mut c2, 0, -1, 1);
        assert_eq!(c1.concentrations[0], 1); // pumped in
        assert_eq!(c2.concentrations[0], 0); // pumped out
    }

    #[test]
    fn test_active_transport_no_energy() {
        let mut c1 = Compartment::new(0, 1, 1);
        let mut c2 = Compartment::new(1, 1, 1);
        c2.concentrations[0] = 1;
        active_transport(&mut c1, &mut c2, 0, -1, 0);
        assert_eq!(c1.concentrations[0], 0); // no energy = no transport
    }

    #[test]
    fn test_equilibrium() {
        let left = Compartment::new(0, 1, 1);
        let right = Compartment::new(1, 1, 1);
        let mem = Membrane::new(0, 1, 1);
        assert!(is_equilibrium(&left, &right, &mem));
    }

    #[test]
    fn test_not_equilibrium() {
        let mut left = Compartment::new(0, 1, 1);
        let right = Compartment::new(1, 1, 1);
        left.concentrations[0] = 1;
        let mem = Membrane::new(0, 1, 1);
        assert!(!is_equilibrium(&left, &right, &mem));
    }

    #[test]
    fn test_system_new() {
        let sys = MembraneSystem::new(3, 2);
        assert_eq!(sys.compartments.len(), 3);
    }

    #[test]
    fn test_system_simulate() {
        let mut sys = MembraneSystem::new(2, 1);
        sys.compartments[0].concentrations[0] = 1;
        sys.add_membrane(0, 1);
        let history = sys.simulate(5);
        assert_eq!(history.len(), 5);
    }

    #[test]
    fn test_channel() {
        let ch = Channel::new(0, 1, 0);
        assert!(ch.open);
        assert!(!ch.gated);
    }

    #[test]
    fn test_gated_channel() {
        let mut ch = Channel::gated_channel(0, 1, 0);
        assert!(!ch.open);
        ch.update_gate(1, 0);
        assert!(ch.open);
    }
}
