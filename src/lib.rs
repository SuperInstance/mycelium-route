//! # Mycelium Route
//!
//! Biological network routing inspired by mycorrhizal networks.
//! Models network links as hyphae, junctions as nodes, data as nutrients,
//! network expansion as growth, and link removal as decay.

use std::collections::HashMap;

// ── hypha ───────────────────────────────────────────────────────────────────

/// A single network link between two nodes.
#[derive(Debug, Clone)]
pub struct Hypha {
    pub id: u64,
    pub source: u64,
    pub target: u64,
    pub bandwidth: f64,
    pub latency: f64,
    pub health: f64, // 0.0 to 1.0
}

impl Hypha {
    pub fn new(id: u64, source: u64, target: u64, bandwidth: f64, latency: f64) -> Self {
        Self { id, source, target, bandwidth, latency, health: 1.0 }
    }

    pub fn effective_bandwidth(&self) -> f64 {
        self.bandwidth * self.health
    }

    pub fn effective_latency(&self) -> f64 {
        self.latency / self.health.max(0.01)
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.1
    }

    pub fn degrade(&mut self, amount: f64) {
        self.health = (self.health - amount).max(0.0);
    }

    pub fn regenerate(&mut self, amount: f64) {
        self.health = (self.health + amount).min(1.0);
    }

    pub fn cost(&self) -> f64 {
        self.effective_latency() / self.effective_bandwidth().max(0.001)
    }

    pub fn connects(&self, node_a: u64, node_b: u64) -> bool {
        (self.source == node_a && self.target == node_b) || (self.source == node_b && self.target == node_a)
    }

    pub fn other_end(&self, node_id: u64) -> Option<u64> {
        if self.source == node_id { Some(self.target) }
        else if self.target == node_id { Some(self.source) }
        else { None }
    }
}

// ── node ────────────────────────────────────────────────────────────────────

/// A junction point in the mycelial network.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: u64,
    pub capacity: f64,
    pub load: f64,
    pub x: f64,
    pub y: f64,
}

impl Node {
    pub fn new(id: u64, capacity: f64, x: f64, y: f64) -> Self {
        Self { id, capacity, load: 0.0, x, y }
    }

    pub fn available_capacity(&self) -> f64 {
        (self.capacity - self.load).max(0.0)
    }

    pub fn utilization(&self) -> f64 {
        if self.capacity == 0.0 { 0.0 } else { self.load / self.capacity }
    }

    pub fn is_overloaded(&self) -> bool {
        self.load > self.capacity
    }

    pub fn add_load(&mut self, amount: f64) -> bool {
        if self.load + amount <= self.capacity {
            self.load += amount;
            true
        } else {
            false
        }
    }

    pub fn remove_load(&mut self, amount: f64) {
        self.load = (self.load - amount).max(0.0);
    }

    pub fn distance_to(&self, other: &Node) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

// ── nutrient ────────────────────────────────────────────────────────────────

/// A data packet with priority traveling through the network.
#[derive(Debug, Clone)]
pub struct Nutrient {
    pub id: u64,
    pub source: u64,
    pub destination: u64,
    pub size: f64,
    pub priority: f64,
    pub ttl: u32,
}

impl Nutrient {
    pub fn new(id: u64, source: u64, destination: u64, size: f64, priority: f64) -> Self {
        Self { id, source, destination, size, priority, ttl: 64 }
    }

    pub fn high_priority(id: u64, source: u64, destination: u64, size: f64) -> Self {
        Self { id, source, destination, size, priority: 1.0, ttl: 64 }
    }

    pub fn low_priority(id: u64, source: u64, destination: u64, size: f64) -> Self {
        Self { id, source, destination, size, priority: 0.1, ttl: 64 }
    }

    pub fn tick_ttl(&mut self) -> bool {
        if self.ttl == 0 { return false; }
        self.ttl -= 1;
        self.ttl > 0
    }

    pub fn is_expired(&self) -> bool {
        self.ttl == 0
    }

    pub fn transmission_cost(&self, hypha: &Hypha) -> f64 {
        self.size * hypha.cost() / self.priority.max(0.01)
    }

    pub fn weighted_size(&self) -> f64 {
        self.size / self.priority.max(0.01)
    }
}

// ── growth ──────────────────────────────────────────────────────────────────

/// Network expansion algorithm that grows new connections.
pub struct Growth {
    pub nodes: HashMap<u64, Node>,
    pub hyphae: Vec<Hypha>,
    pub next_id: u64,
}

impl Default for Growth {
    fn default() -> Self {
        Self::new()
    }
}

impl Growth {
    pub fn new() -> Self {
        Self { nodes: HashMap::new(), hyphae: Vec::new(), next_id: 1 }
    }

    pub fn add_node(&mut self, capacity: f64, x: f64, y: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, Node::new(id, capacity, x, y));
        id
    }

    pub fn add_hypha(&mut self, source: u64, target: u64, bandwidth: f64, latency: f64) -> Option<u64> {
        if !self.nodes.contains_key(&source) || !self.nodes.contains_key(&target) {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.hyphae.push(Hypha::new(id, source, target, bandwidth, latency));
        Some(id)
    }

    pub fn grow_toward(&mut self, from_id: u64, target_x: f64, target_y: f64, bandwidth: f64) -> Option<u64> {
        let from = self.nodes.get(&from_id)?;
        let dx = target_x - from.x;
        let dy = target_y - from.y;
        let dist = (dx * dx + dy * dy).sqrt();
        let step = dist.min(5.0);
        let nx = from.x + dx / dist * step;
        let ny = from.y + dy / dist * step;
        let new_id = self.add_node(bandwidth, nx, ny);
        self.add_hypha(from_id, new_id, bandwidth, dist);
        Some(new_id)
    }

    pub fn neighbors(&self, node_id: u64) -> Vec<u64> {
        self.hyphae.iter()
            .filter_map(|h| h.other_end(node_id))
            .collect()
    }

    pub fn find_path(&self, source: u64, dest: u64) -> Vec<u64> {
        // BFS shortest path
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![(source, vec![source])];
        visited.insert(source);
        while let Some((current, path)) = queue.pop() {
            if current == dest { return path; }
            for &neighbor in &self.neighbors(current) {
                if visited.insert(neighbor) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor);
                    queue.insert(0, (neighbor, new_path));
                }
            }
        }
        vec![]
    }

    pub fn shortest_distance(&self, source: u64, dest: u64) -> f64 {
        let path = self.find_path(source, dest);
        if path.len() < 2 { return f64::INFINITY; }
        path.windows(2)
            .filter_map(|w| {
                let a = self.nodes.get(&w[0])?;
                let b = self.nodes.get(&w[1])?;
                Some(a.distance_to(b))
            })
            .sum()
    }

    pub fn prune_dead(&mut self) -> usize {
        let before = self.hyphae.len();
        self.hyphae.retain(|h| h.is_alive());
        before - self.hyphae.len()
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn hypha_count(&self) -> usize { self.hyphae.len() }
}

// ── decay ───────────────────────────────────────────────────────────────────

/// Link removal and rerouting logic.
pub struct Decay {
    pub rate: f64,
    pub threshold: f64,
}

impl Decay {
    pub fn new(rate: f64, threshold: f64) -> Self {
        Self { rate, threshold }
    }

    pub fn slow() -> Self { Self { rate: 0.01, threshold: 0.1 } }
    pub fn fast() -> Self { Self { rate: 0.1, threshold: 0.1 } } 

    pub fn apply(&self, hypha: &mut Hypha) {
        hypha.degrade(self.rate);
    }

    pub fn is_dead(&self, hypha: &Hypha) -> bool {
        hypha.health < self.threshold
    }

    pub fn decay_network(&self, network: &mut Growth) -> Vec<u64> {
        let mut dead = Vec::new();
        for hypha in &mut network.hyphae {
            self.apply(hypha);
            if self.is_dead(hypha) {
                dead.push(hypha.id);
            }
        }
        dead
    }

    pub fn reroute(&self, network: &mut Growth, dead_links: &[u64]) -> Vec<(u64, Vec<u64>)> {
        let mut rerouted = Vec::new();
        for &dead_id in dead_links {
            let dead = network.hyphae.iter().find(|h| h.id == dead_id).cloned();
            if let Some(d) = dead {
                let path = network.find_path(d.source, d.target);
                if !path.is_empty() {
                    rerouted.push((dead_id, path));
                }
            }
        }
        rerouted
    }

    pub fn selective_decay(&self, hypha: &mut Hypha, usage_factor: f64) {
        hypha.degrade(self.rate * usage_factor);
    }
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod hypha_tests {
    use super::*;

    #[test]
    fn test_new() {
        let h = Hypha::new(1, 10, 20, 100.0, 5.0);
        assert_eq!(h.id, 1);
        assert_eq!(h.source, 10);
        assert_eq!(h.target, 20);
        assert_eq!(h.health, 1.0);
    }

    #[test]
    fn test_effective_bandwidth() {
        let mut h = Hypha::new(1, 0, 1, 100.0, 5.0);
        h.health = 0.5;
        assert!((h.effective_bandwidth() - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_effective_latency() {
        let mut h = Hypha::new(1, 0, 1, 100.0, 5.0);
        h.health = 0.5;
        assert!((h.effective_latency() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_is_alive() {
        let mut h = Hypha::new(1, 0, 1, 100.0, 5.0);
        assert!(h.is_alive());
        h.health = 0.05;
        assert!(!h.is_alive());
    }

    #[test]
    fn test_degrade() {
        let mut h = Hypha::new(1, 0, 1, 100.0, 5.0);
        h.degrade(0.3);
        assert!((h.health - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_degrade_clamped() {
        let mut h = Hypha::new(1, 0, 1, 100.0, 5.0);
        h.degrade(2.0);
        assert_eq!(h.health, 0.0);
    }

    #[test]
    fn test_regenerate() {
        let mut h = Hypha::new(1, 0, 1, 100.0, 5.0);
        h.health = 0.5;
        h.regenerate(0.3);
        assert!((h.health - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_regenerate_clamped() {
        let mut h = Hypha::new(1, 0, 1, 100.0, 5.0);
        h.regenerate(0.5);
        assert_eq!(h.health, 1.0);
    }

    #[test]
    fn test_cost() {
        let h = Hypha::new(1, 0, 1, 100.0, 5.0);
        assert!((h.cost() - 0.05).abs() < 1e-10);
    }

    #[test]
    fn test_connects() {
        let h = Hypha::new(1, 10, 20, 100.0, 5.0);
        assert!(h.connects(10, 20));
        assert!(h.connects(20, 10));
        assert!(!h.connects(10, 30));
    }

    #[test]
    fn test_other_end() {
        let h = Hypha::new(1, 10, 20, 100.0, 5.0);
        assert_eq!(h.other_end(10), Some(20));
        assert_eq!(h.other_end(20), Some(10));
        assert_eq!(h.other_end(30), None);
    }
}

#[cfg(test)]
mod node_tests {
    use super::*;

    #[test]
    fn test_new() {
        let n = Node::new(1, 100.0, 5.0, 10.0);
        assert_eq!(n.id, 1);
        assert_eq!(n.capacity, 100.0);
        assert_eq!(n.load, 0.0);
    }

    #[test]
    fn test_available_capacity() {
        let mut n = Node::new(1, 100.0, 0.0, 0.0);
        n.load = 30.0;
        assert!((n.available_capacity() - 70.0).abs() < 1e-10);
    }

    #[test]
    fn test_utilization() {
        let mut n = Node::new(1, 100.0, 0.0, 0.0);
        n.load = 50.0;
        assert!((n.utilization() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_is_overloaded() {
        let mut n = Node::new(1, 100.0, 0.0, 0.0);
        assert!(!n.is_overloaded());
        n.load = 150.0;
        assert!(n.is_overloaded());
    }

    #[test]
    fn test_add_load_success() {
        let mut n = Node::new(1, 100.0, 0.0, 0.0);
        assert!(n.add_load(50.0));
        assert!((n.load - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_add_load_fail() {
        let mut n = Node::new(1, 100.0, 0.0, 0.0);
        assert!(!n.add_load(150.0));
        assert_eq!(n.load, 0.0);
    }

    #[test]
    fn test_remove_load() {
        let mut n = Node::new(1, 100.0, 0.0, 0.0);
        n.load = 50.0;
        n.remove_load(30.0);
        assert!((n.load - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_remove_load_clamped() {
        let mut n = Node::new(1, 100.0, 0.0, 0.0);
        n.load = 10.0;
        n.remove_load(50.0);
        assert_eq!(n.load, 0.0);
    }

    #[test]
    fn test_distance_to() {
        let a = Node::new(1, 100.0, 0.0, 0.0);
        let b = Node::new(2, 100.0, 3.0, 4.0);
        assert!((a.distance_to(&b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_zero_capacity_utilization() {
        let n = Node::new(1, 0.0, 0.0, 0.0);
        assert_eq!(n.utilization(), 0.0);
    }
}

#[cfg(test)]
mod nutrient_tests {
    use super::*;

    #[test]
    fn test_new() {
        let n = Nutrient::new(1, 10, 20, 50.0, 0.5);
        assert_eq!(n.source, 10);
        assert_eq!(n.destination, 20);
        assert_eq!(n.ttl, 64);
    }

    #[test]
    fn test_high_priority() {
        let n = Nutrient::high_priority(1, 0, 1, 10.0);
        assert!((n.priority - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_low_priority() {
        let n = Nutrient::low_priority(1, 0, 1, 10.0);
        assert!((n.priority - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_tick_ttl() {
        let mut n = Nutrient::new(1, 0, 1, 10.0, 0.5);
        assert!(n.tick_ttl());
        assert_eq!(n.ttl, 63);
    }

    #[test]
    fn test_ttl_expires() {
        let mut n = Nutrient::new(1, 0, 1, 10.0, 0.5);
        n.ttl = 1;
        assert!(!n.tick_ttl());
    }

    #[test]
    fn test_is_expired() {
        let mut n = Nutrient::new(1, 0, 1, 10.0, 0.5);
        assert!(!n.is_expired());
        n.ttl = 0;
        assert!(n.is_expired());
    }

    #[test]
    fn test_transmission_cost() {
        let n = Nutrient::new(1, 0, 1, 10.0, 1.0);
        let h = Hypha::new(1, 0, 1, 100.0, 5.0);
        assert!(n.transmission_cost(&h) > 0.0);
    }

    #[test]
    fn test_weighted_size() {
        let n = Nutrient::new(1, 0, 1, 10.0, 0.5);
        assert!((n.weighted_size() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_priority_affects_cost() {
        let high = Nutrient::high_priority(1, 0, 1, 10.0);
        let low = Nutrient::low_priority(2, 0, 1, 10.0);
        let h = Hypha::new(1, 0, 1, 100.0, 5.0);
        assert!(high.transmission_cost(&h) < low.transmission_cost(&h));
    }
}

#[cfg(test)]
mod growth_tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut g = Growth::new();
        let id = g.add_node(100.0, 0.0, 0.0);
        assert_eq!(id, 1);
        assert_eq!(g.node_count(), 1);
    }

    #[test]
    fn test_add_hypha() {
        let mut g = Growth::new();
        let a = g.add_node(100.0, 0.0, 0.0);
        let b = g.add_node(100.0, 3.0, 4.0);
        let h = g.add_hypha(a, b, 50.0, 1.0);
        assert!(h.is_some());
        assert_eq!(g.hypha_count(), 1);
    }

    #[test]
    fn test_add_hypha_invalid() {
        let mut g = Growth::new();
        let a = g.add_node(100.0, 0.0, 0.0);
        assert!(g.add_hypha(a, 999, 50.0, 1.0).is_none());
    }

    #[test]
    fn test_neighbors() {
        let mut g = Growth::new();
        let a = g.add_node(100.0, 0.0, 0.0);
        let b = g.add_node(100.0, 1.0, 0.0);
        let c = g.add_node(100.0, 2.0, 0.0);
        g.add_hypha(a, b, 50.0, 1.0);
        g.add_hypha(a, c, 50.0, 1.0);
        let nb = g.neighbors(a);
        assert_eq!(nb.len(), 2);
    }

    #[test]
    fn test_find_path_simple() {
        let mut g = Growth::new();
        let a = g.add_node(100.0, 0.0, 0.0);
        let b = g.add_node(100.0, 1.0, 0.0);
        let c = g.add_node(100.0, 2.0, 0.0);
        g.add_hypha(a, b, 50.0, 1.0);
        g.add_hypha(b, c, 50.0, 1.0);
        let path = g.find_path(a, c);
        assert_eq!(path, vec![a, b, c]);
    }

    #[test]
    fn test_find_path_no_path() {
        let mut g = Growth::new();
        let a = g.add_node(100.0, 0.0, 0.0);
        let b = g.add_node(100.0, 10.0, 0.0);
        let path = g.find_path(a, b);
        assert!(path.is_empty());
    }

    #[test]
    fn test_find_path_same_node() {
        let mut g = Growth::new();
        let a = g.add_node(100.0, 0.0, 0.0);
        let path = g.find_path(a, a);
        assert_eq!(path, vec![a]);
    }

    #[test]
    fn test_grow_toward() {
        let mut g = Growth::new();
        let a = g.add_node(100.0, 0.0, 0.0);
        let new = g.grow_toward(a, 10.0, 0.0, 50.0);
        assert!(new.is_some());
        assert_eq!(g.node_count(), 2);
        assert_eq!(g.hypha_count(), 1);
    }

    #[test]
    fn test_prune_dead() {
        let mut g = Growth::new();
        let a = g.add_node(100.0, 0.0, 0.0);
        let b = g.add_node(100.0, 1.0, 0.0);
        g.add_hypha(a, b, 50.0, 1.0);
        g.hyphae[0].health = 0.01;
        let pruned = g.prune_dead();
        assert_eq!(pruned, 1);
        assert_eq!(g.hypha_count(), 0);
    }

    #[test]
    fn test_shortest_distance() {
        let mut g = Growth::new();
        let a = g.add_node(100.0, 0.0, 0.0);
        let b = g.add_node(100.0, 3.0, 4.0);
        g.add_hypha(a, b, 50.0, 1.0);
        let dist = g.shortest_distance(a, b);
        assert!((dist - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_ids_increment() {
        let mut g = Growth::new();
        let a = g.add_node(100.0, 0.0, 0.0);
        let b = g.add_node(100.0, 1.0, 0.0);
        assert!(b > a);
    }
}

#[cfg(test)]
mod decay_tests {
    use super::*;

    #[test]
    fn test_slow_decay() {
        let d = Decay::slow();
        assert!((d.rate - 0.01).abs() < 1e-10);
    }

    #[test]
    fn test_fast_decay() {
        let d = Decay::fast();
        assert!((d.rate - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_apply() {
        let d = Decay::new(0.1, 0.1);
        let mut h = Hypha::new(1, 0, 1, 100.0, 5.0);
        d.apply(&mut h);
        assert!((h.health - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_is_dead() {
        let d = Decay::new(0.1, 0.5);
        let h = Hypha::new(1, 0, 1, 100.0, 5.0);
        assert!(!d.is_dead(&h));
        let mut h2 = h.clone();
        h2.health = 0.3;
        assert!(d.is_dead(&h2));
    }

    #[test]
    fn test_decay_network() {
        let d = Decay::new(0.5, 0.1);
        let mut g = Growth::new();
        let a = g.add_node(100.0, 0.0, 0.0);
        let b = g.add_node(100.0, 1.0, 0.0);
        g.add_hypha(a, b, 50.0, 1.0);
        let dead = d.decay_network(&mut g);
        // After one decay of 0.5, health=0.5 which is >= 0.1 threshold
        assert!(dead.is_empty() || dead.len() == 1);
    }

    #[test]
    fn test_reroute() {
        let d = Decay::new(0.1, 0.1);
        let mut g = Growth::new();
        let a = g.add_node(100.0, 0.0, 0.0);
        let b = g.add_node(100.0, 5.0, 0.0);
        let c = g.add_node(100.0, 2.5, 3.0);
        let h1 = g.add_hypha(a, b, 50.0, 1.0).unwrap();
        g.add_hypha(a, c, 50.0, 1.0);
        g.add_hypha(c, b, 50.0, 1.0);
        let rerouted = d.reroute(&mut g, &[h1]);
        assert!(!rerouted.is_empty());
    }

    #[test]
    fn test_selective_decay() {
        let d = Decay::new(0.1, 0.1);
        let mut h = Hypha::new(1, 0, 1, 100.0, 5.0);
        d.selective_decay(&mut h, 2.0);
        assert!((h.health - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_decay_to_zero() {
        let d = Decay::new(1.0, 0.1);
        let mut g = Growth::new();
        let a = g.add_node(100.0, 0.0, 0.0);
        let b = g.add_node(100.0, 1.0, 0.0);
        g.add_hypha(a, b, 50.0, 1.0);
        for _ in 0..5 {
            d.decay_network(&mut g);
        }
        assert!(g.hyphae.iter().all(|h| h.health <= 0.01));
    }

    #[test]
    fn test_new_custom() {
        let d = Decay::new(0.05, 0.2);
        assert!((d.rate - 0.05).abs() < 1e-10);
        assert!((d.threshold - 0.2).abs() < 1e-10);
    }
}
