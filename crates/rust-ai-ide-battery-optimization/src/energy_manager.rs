//! Energy Management System
//!
//! Provides intelligent energy management strategies for balancing
//! performance with battery life across different operating modes.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::{BatteryState, BatteryConfig, AIPowerMode};

/// Energy management service
pub struct EnergyManager {
    state: Arc<RwLock<EnergyState>>,
    energy_profiles: Arc<RwLock<HashMap<String, EnergyProfile>>>,
    config: BatteryConfig,
}

/// Internal energy state
pub struct EnergyState {
    pub current_power_mode: PowerMode,
    pub energy_budget: EnergyBudget,
    pub resource_limits: ResourceLimits,
    pub thermal_state: ThermalState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyProfile {
    pub name: String,
    pub power_mode: PowerMode,
    pub cpu_scaling: f32,          // 0.0 to 1.0 (1.0 = max performance)
    pub gpu_scaling: Option<f32>,  // GPU scaling if available
    pub memory_pressure: f32,      // 0.0 to 1.0 (0.0 = aggressive freeing)
    pub network_throttling: bool,
    pub background_task_throttle: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PowerMode {
    HighPerformance,
    Balanced,
    PowerSaver,
    Emergency,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyBudget {
    pub time_window_minutes: u32,
    pub target_energy_usage_mah: f32,
    pub current_usage_mah: f32,
    pub efficiency_score: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_cpu_usage_percent: f32,
    pub max_memory_usage_mb: Option<u64>,
    pub max_network_bandwidth_kbps: Option<u64>,
    pub background_task_cpu_limit: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalState {
    pub current_temperature_celsius: f32,
    pub throttle_threshold_celsius: f32,
    pub cooling_required: bool,
}

impl EnergyManager {
    pub fn new() -> Self {
        let config = BatteryConfig::default();
        let energy_profiles = Self::create_default_profiles();

        Self {
            state: Arc::new(RwLock::new(EnergyState {
                current_power_mode: PowerMode::Balanced,
                energy_budget: EnergyBudget {
                    time_window_minutes: 60,
                    target_energy_usage_mah: 1000.0,
                    current_usage_mah: 0.0,
                    efficiency_score: 0.8,
                },
                resource_limits: ResourceLimits::balanced(),
                thermal_state: ThermalState::default(),
            })),
            energy_profiles: Arc::new(RwLock::new(energy_profiles)),
            config,
        }
    }

    /// Create default energy profiles
    fn create_default_profiles() -> HashMap<String, EnergyProfile> {
        let mut profiles = HashMap::new();

        profiles.insert("high_performance".to_string(), EnergyProfile {
            name: "High Performance".to_string(),
            power_mode: PowerMode::HighPerformance,
            cpu_scaling: 1.0,
            gpu_scaling: Some(1.0),
            memory_pressure: 0.2,
            network_throttling: false,
            background_task_throttle: false,
            description: "Maximum performance, minimal power optimization".to_string(),
        });

        profiles.insert("balanced".to_string(), EnergyProfile {
            name: "Balanced".to_string(),
            power_mode: PowerMode::Balanced,
            cpu_scaling: 0.8,
            gpu_scaling: Some(0.8),
            memory_pressure: 0.5,
            network_throttling: false,
            background_task_throttle: true,
            description: "Good balance between performance and power efficiency".to_string(),
        });

        profiles.insert("power_saver".to_string(), EnergyProfile {
            name: "Power Saver".to_string(),
            power_mode: PowerMode::PowerSaver,
            cpu_scaling: 0.5,
            gpu_scaling: Some(0.3),
            memory_pressure: 0.8,
            network_throttling: true,
            background_task_throttle: true,
            description: "Aggressive power saving with reduced performance".to_string(),
        });

        profiles.insert("emergency".to_string(), EnergyProfile {
            name: "Emergency".to_string(),
            power_mode: PowerMode::Emergency,
            cpu_scaling: 0.3,
            gpu_scaling: Some(0.1),
            memory_pressure: 0.9,
            network_throttling: true,
            background_task_throttle: true,
            description: "Critical battery mode for maximum power preservation".to_string(),
        });

        profiles
    }

    pub async fn get_current_profile(&self) -> anyhow::Result<EnergyProfile> {
        let state = self.state.read().await;
        let profiles = self.energy_profiles.read().await;

        let profile_key = match state.current_power_mode {
            PowerMode::HighPerformance => "high_performance",
            PowerMode::Balanced => "balanced",
            PowerMode::PowerSaver => "power_saver",
            PowerMode::Emergency => "emergency",
            PowerMode::Custom(ref name) => name,
        };

        profiles.get(profile_key).cloned()
            .ok_or_else(|| anyhow::anyhow!("Profile not found: {}", profile_key))
    }

    pub async fn switch_power_mode(&self, battery_state: &BatteryState, ai_mode: &AIPowerMode) -> anyhow::Result<()> {
        let new_mode = self.calculate_optimal_mode(battery_state, ai_mode);

        let mut state = self.state.write().await;
        state.current_power_mode = new_mode.clone();

        // Update resource limits based on new mode
        state.resource_limits = match new_mode {
            PowerMode::HighPerformance => ResourceLimits::high_performance(),
            PowerMode::Balanced => ResourceLimits::balanced(),
            PowerMode::PowerSaver => ResourceLimits::power_saver(),
            PowerMode::Emergency => ResourceLimits::emergency(),
            PowerMode::Custom(_) => ResourceLimits::balanced(), // Default to balanced
        };

        tracing::info!("Switched to power mode: {:?}", new_mode);
        Ok(())
    }

    fn calculate_optimal_mode(&self, battery_state: &BatteryState, ai_mode: &AIPowerMode) -> PowerMode {
        // Intelligent mode selection based on battery state and AI requirements
        if battery_state.level < self.config.emergency_mode_threshold {
            return PowerMode::Emergency;
        }

        if ai_mode == &AIPowerMode::HighPerformance {
            return PowerMode::HighPerformance;
        }

        if ai_mode == &AIPowerMode::Emergency {
            return PowerMode::Emergency;
        }

        if ai_mode == &AIPowerMode::PowerSaver {
            return PowerMode::PowerSaver;
        }

        // Battery-based decision
        if battery_state.level < 0.2 {
            PowerMode::PowerSaver
        } else if battery_state.level < 0.5 {
            PowerMode::Balanced
        } else if battery_state.level > 0.8 {
            PowerMode::HighPerformance
        } else {
            PowerMode::Balanced
        }
    }

    pub async fn update_energy_budget(&self, battery_state: &BatteryState) -> anyhow::Result<()> {
        let mut state = self.state.write().await;
        let profile = self.energy_profiles.read().await;

        let profile_key = match state.current_power_mode {
            PowerMode::HighPerformance => "high_performance",
            PowerMode::Balanced => "balanced",
            PowerMode::PowerSaver => "power_saver",
            PowerMode::Emergency => "emergency",
            PowerMode::Custom(ref name) => name,
        };

        if let Some(current_profile) = profile.get(profile_key) {
            // Estimate energy usage based on profile and battery state
            let estimated_usage = self.calculate_energy_usage(current_profile, battery_state)?;
            state.energy_budget.current_usage_mah = estimated_usage;

            // Calculate efficiency score
            state.energy_budget.efficiency_score = self.calculate_efficiency_score(
                &state.energy_budget,
                battery_state
            );
        }

        Ok(())
    }

    fn calculate_energy_usage(&self, profile: &EnergyProfile, battery_state: &BatteryState) -> anyhow::Result<f32> {
        // Simplified energy calculation - in real implementation this would be more sophisticated
        let base_usage_per_hour = match profile.power_mode {
            PowerMode::HighPerformance => 1500.0, // mAh/hour
            PowerMode::Balanced => 1000.0,
            PowerMode::PowerSaver => 600.0,
            PowerMode::Emergency => 300.0,
            PowerMode::Custom(_) => 1000.0,
        };

        // Adjust for battery level and temperature
        let level_factor = if battery_state.level < 0.2 { 1.3 } else { 1.0 }; // Higher drain at low battery
        let temp_factor = if let Some(temp) = battery_state.temperature {
            if temp > 35.0 { 1.2 } else if temp < 20.0 { 0.9 } else { 1.0 }
        } else { 1.0 };

        Ok(base_usage_per_hour * level_factor * temp_factor)
    }

    fn calculate_efficiency_score(&self, budget: &EnergyBudget, battery_state: &BatteryState) -> f32 {
        let efficiency = if budget.target_energy_usage_mah > 0.0 {
            (budget.target_energy_usage_mah - budget.current_usage_mah).max(0.0) / budget.target_energy_usage_mah
        } else {
            0.5
        };

        // Adjust based on battery health
        let health_bonus = battery_state.health_percentage.unwrap_or(1.0) * 0.2; // Max 20% bonus
        (efficiency + health_bonus).min(1.0).max(0.0)
    }

    pub async fn get_energy_budget(&self) -> anyhow::Result<EnergyBudget> {
        let state = self.state.read().await;
        Ok(state.energy_budget.clone())
    }

    pub async fn get_resource_limits(&self) -> anyhow::Result<ResourceLimits> {
        let state = self.state.read().await;
        Ok(state.resource_limits.clone())
    }
}

impl ResourceLimits {
    pub fn high_performance() -> Self {
        Self {
            max_cpu_usage_percent: 95.0,
            max_memory_usage_mb: None,
            max_network_bandwidth_kbps: None,
            background_task_cpu_limit: 80.0,
        }
    }

    pub fn balanced() -> Self {
        Self {
            max_cpu_usage_percent: 75.0,
            max_memory_usage_mb: None,
            max_network_bandwidth_kbps: None,
            background_task_cpu_limit: 50.0,
        }
    }

    pub fn power_saver() -> Self {
        Self {
            max_cpu_usage_percent: 50.0,
            max_memory_usage_mb: Some(2048), // 2GB limit
            max_network_bandwidth_kbps: Some(1024), // 1Mbps limit
            background_task_cpu_limit: 20.0,
        }
    }

    pub fn emergency() -> Self {
        Self {
            max_cpu_usage_percent: 30.0,
            max_memory_usage_mb: Some(1024), // 1GB limit
            max_network_bandwidth_kbps: Some(256), // 256Kbps limit
            background_task_cpu_limit: 5.0,
        }
    }
}

impl Default for ThermalState {
    fn default() -> Self {
        Self {
            current_temperature_celsius: 25.0,
            throttle_threshold_celsius: 40.0,
            cooling_required: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_energy_manager_creation() {
        let manager = EnergyManager::new();
        let budget = manager.get_energy_budget().await.unwrap();
        assert!(budget.efficiency_score >= 0.0 && budget.efficiency_score <= 1.0);
    }

    #[tokio::test]
    async fn test_power_mode_switching() {
        let manager = EnergyManager::new();
        let battery_state = BatteryState {
            level: 0.15, // Critical battery
            voltage: Some(3.5),
            temperature: Some(30.0),
            is_charging: false,
            health_percentage: Some(0.9),
            time_remaining_minutes: Some(30),
        };

        manager.switch_power_mode(&battery_state, &AIPowerMode::Balanced).await.unwrap();

        // Should switch to emergency mode due to low battery
        let profile = manager.get_current_profile().await.unwrap();
        match profile.power_mode {
            PowerMode::Emergency => {}, // Expected
            other => panic!("Expected Emergency mode, got {:?}", other),
        }
    }
}