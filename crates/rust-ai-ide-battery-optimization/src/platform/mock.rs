//! Mock battery monitor for development and testing
//!
//! Provides simulated battery behavior for desktop development environments
//! where actual battery hardware is not available.

use std::sync::Arc;
use tokio::sync::RwLock;
use rand::Rng;
use crate::{BatteryState, battery_monitor::PlatformBatteryMonitor};

pub struct MockBatteryMonitor {
    simulated_state: Arc<RwLock<SimulatedBatteryState>>,
}

struct SimulatedBatteryState {
    current_level: f32,
    is_plugged: bool,
    temperature: f32,
    cycle_count: u32,
    max_cycles: u32,
}

impl MockBatteryMonitor {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            simulated_state: Arc::new(RwLock::new(SimulatedBatteryState {
                current_level: rng.gen_range(0.1..1.0),
                is_plugged: rng.gen_bool(0.3), // 30% chance of being plugged in
                temperature: rng.gen_range(20.0..45.0),
                cycle_count: rng.gen_range(0..500),
                max_cycles: rng.gen_range(500..1000),
            })),
        }
    }

    /// Simulate battery drain over time
    fn simulate_drain(&self, state: &mut SimulatedBatteryState) {
        if !state.is_plugged && state.current_level > 0.0 {
            // Drain rate: 1-5% per hour
            let drain_rate = rand::thread_rng().gen_range(0.01..0.05) / 3600.0; // per second
            state.current_level = (state.current_level - drain_rate).max(0.0);
        } else if state.is_plugged && state.current_level < 1.0 {
            // Charge rate: 5-20% per hour
            let charge_rate = rand::thread_rng().gen_range(0.05..0.20) / 3600.0; // per second
            state.current_level = (state.current_level + charge_rate).min(1.0);
        }

        // Random temperature fluctuation
        let temp_change = rand::thread_rng().gen_range(-0.1..0.1);
        state.temperature = (state.temperature + temp_change).clamp(15.0, 60.0);

        // Occasional plug/unplug
        if rand::thread_rng().gen_bool(0.001) { // ~1 in 1000 calls
            state.is_plugged = !state.is_plugged;
            tracing::info!("Mock battery: Power source {}", if state.is_plugged { "connected" } else { "disconnected" });
        }
    }
}

#[async_trait::async_trait]
impl PlatformBatteryMonitor for MockBatteryMonitor {
    async fn get_battery_state(&self) -> anyhow::Result<BatteryState> {
        let mut state = self.simulated_state.write().await;
        self.simulate_drain(&mut state);

        let health_percentage = if state.cycle_count < state.max_cycles {
            Some((state.max_cycles - state.cycle_count) as f32 / state.max_cycles as f32)
        } else {
            Some(0.1) // Degraded
        };

        let time_remaining_minutes = if !state.is_plugged && state.current_level > 0.0 {
            // Estimate time remaining: assume 8-12 hour battery life
            let hours_left = state.current_level * rand::thread_rng().gen_range(8.0..12.0);
            Some((hours_left * 60.0) as u32)
        } else if state.is_plugged {
            None // Infinite when plugged in
        } else {
            Some(0) // No time left
        };

        Ok(BatteryState {
            level: state.current_level,
            voltage: Some(rand::thread_rng().gen_range(3.0..4.2)), // Typical Li-ion voltage
            temperature: Some(state.temperature),
            is_charging: state.is_plugged,
            health_percentage,
            time_remaining_minutes,
        })
    }

    async fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::info!("MockBatteryMonitor: Initialized mock battery monitoring");
        Ok(())
    }

    fn platform_name(&self) -> &'static str {
        "mock"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_battery_monitor() {
        let monitor = MockBatteryMonitor::new();
        monitor.initialize().await.unwrap();

        let state = monitor.get_battery_state().await.unwrap();
        assert!(state.level >= 0.0 && state.level <= 1.0);
        assert!(state.temperature.unwrap() >= 15.0 && state.temperature.unwrap() <= 60.0);
        assert!(state.voltage.unwrap() >= 3.0 && state.voltage.unwrap() <= 4.2);
    }
}