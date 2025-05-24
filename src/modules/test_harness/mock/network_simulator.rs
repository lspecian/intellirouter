//! Network Simulator Module
//!
//! This module provides functionality for simulating network conditions
//! such as latency, packet loss, and bandwidth limitations.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::modules::test_harness::types::TestHarnessError;

/// Network condition type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkConditionType {
    /// Perfect network (no impairments)
    Perfect,
    /// Good network (minimal impairments)
    Good,
    /// Average network (some impairments)
    Average,
    /// Poor network (significant impairments)
    Poor,
    /// Very poor network (severe impairments)
    VeryPoor,
    /// Custom network conditions
    Custom,
}

impl fmt::Display for NetworkConditionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkConditionType::Perfect => write!(f, "Perfect"),
            NetworkConditionType::Good => write!(f, "Good"),
            NetworkConditionType::Average => write!(f, "Average"),
            NetworkConditionType::Poor => write!(f, "Poor"),
            NetworkConditionType::VeryPoor => write!(f, "Very Poor"),
            NetworkConditionType::Custom => write!(f, "Custom"),
        }
    }
}

/// Network simulator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSimulatorConfig {
    /// Whether to simulate network conditions
    pub enabled: bool,
    /// Network condition type
    pub condition_type: NetworkConditionType,
    /// Latency configuration
    pub latency: LatencyConfig,
    /// Packet loss configuration
    pub packet_loss: PacketLossConfig,
    /// Bandwidth configuration
    pub bandwidth: BandwidthConfig,
}

impl Default for NetworkSimulatorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            condition_type: NetworkConditionType::Perfect,
            latency: LatencyConfig::default(),
            packet_loss: PacketLossConfig::default(),
            bandwidth: BandwidthConfig::default(),
        }
    }
}

/// Latency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyConfig {
    /// Whether to simulate latency
    pub enabled: bool,
    /// Minimum latency in milliseconds
    pub min_ms: u64,
    /// Maximum latency in milliseconds
    pub max_ms: u64,
    /// Jitter in milliseconds (variation in latency)
    pub jitter_ms: u64,
}

impl Default for LatencyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_ms: 0,
            max_ms: 0,
            jitter_ms: 0,
        }
    }
}

/// Packet loss configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketLossConfig {
    /// Whether to simulate packet loss
    pub enabled: bool,
    /// Probability of packet loss (0.0 - 1.0)
    pub probability: f64,
    /// Burst size (number of consecutive packets lost)
    pub burst_size: u32,
}

impl Default for PacketLossConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            probability: 0.0,
            burst_size: 1,
        }
    }
}

/// Bandwidth configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthConfig {
    /// Whether to simulate bandwidth limitations
    pub enabled: bool,
    /// Bandwidth in kilobits per second
    pub kbps: u64,
}

impl Default for BandwidthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            kbps: 0,
        }
    }
}

/// Network simulator for simulating network conditions
pub struct NetworkSimulator {
    /// Configuration
    config: RwLock<NetworkSimulatorConfig>,
}

impl NetworkSimulator {
    /// Create a new network simulator
    pub fn new(config: NetworkSimulatorConfig) -> Self {
        Self {
            config: RwLock::new(config),
        }
    }

    /// Get the configuration
    pub async fn config(&self) -> NetworkSimulatorConfig {
        self.config.read().await.clone()
    }

    /// Update the configuration
    pub async fn update_config(&self, config: NetworkSimulatorConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    /// Set the network condition type
    pub async fn set_condition_type(&self, condition_type: NetworkConditionType) {
        let mut config = self.config.write().await;

        // Update the condition type
        config.condition_type = condition_type;

        // Update the configuration based on the condition type
        match condition_type {
            NetworkConditionType::Perfect => {
                config.latency = LatencyConfig {
                    enabled: false,
                    min_ms: 0,
                    max_ms: 0,
                    jitter_ms: 0,
                };
                config.packet_loss = PacketLossConfig {
                    enabled: false,
                    probability: 0.0,
                    burst_size: 1,
                };
                config.bandwidth = BandwidthConfig {
                    enabled: false,
                    kbps: 0,
                };
            }
            NetworkConditionType::Good => {
                config.latency = LatencyConfig {
                    enabled: true,
                    min_ms: 20,
                    max_ms: 50,
                    jitter_ms: 5,
                };
                config.packet_loss = PacketLossConfig {
                    enabled: true,
                    probability: 0.01,
                    burst_size: 1,
                };
                config.bandwidth = BandwidthConfig {
                    enabled: true,
                    kbps: 5000, // 5 Mbps
                };
            }
            NetworkConditionType::Average => {
                config.latency = LatencyConfig {
                    enabled: true,
                    min_ms: 50,
                    max_ms: 100,
                    jitter_ms: 10,
                };
                config.packet_loss = PacketLossConfig {
                    enabled: true,
                    probability: 0.03,
                    burst_size: 2,
                };
                config.bandwidth = BandwidthConfig {
                    enabled: true,
                    kbps: 1000, // 1 Mbps
                };
            }
            NetworkConditionType::Poor => {
                config.latency = LatencyConfig {
                    enabled: true,
                    min_ms: 100,
                    max_ms: 300,
                    jitter_ms: 30,
                };
                config.packet_loss = PacketLossConfig {
                    enabled: true,
                    probability: 0.05,
                    burst_size: 3,
                };
                config.bandwidth = BandwidthConfig {
                    enabled: true,
                    kbps: 500, // 500 Kbps
                };
            }
            NetworkConditionType::VeryPoor => {
                config.latency = LatencyConfig {
                    enabled: true,
                    min_ms: 300,
                    max_ms: 1000,
                    jitter_ms: 100,
                };
                config.packet_loss = PacketLossConfig {
                    enabled: true,
                    probability: 0.1,
                    burst_size: 5,
                };
                config.bandwidth = BandwidthConfig {
                    enabled: true,
                    kbps: 100, // 100 Kbps
                };
            }
            NetworkConditionType::Custom => {
                // Keep the current configuration
            }
        }
    }

    /// Simulate network conditions for a request
    pub async fn simulate(&self, data_size: usize) -> Result<(), TestHarnessError> {
        let config = self.config.read().await.clone();

        if !config.enabled {
            return Ok(());
        }

        // Simulate packet loss
        if config.packet_loss.enabled {
            self.simulate_packet_loss(&config.packet_loss).await?;
        }

        // Simulate latency
        if config.latency.enabled {
            self.simulate_latency(&config.latency).await?;
        }

        // Simulate bandwidth limitations
        if config.bandwidth.enabled {
            self.simulate_bandwidth(&config.bandwidth, data_size)
                .await?;
        }

        Ok(())
    }

    /// Simulate packet loss
    async fn simulate_packet_loss(
        &self,
        config: &PacketLossConfig,
    ) -> Result<(), TestHarnessError> {
        let mut rng = rand::thread_rng();

        if rng.gen::<f64>() < config.probability {
            // Simulate packet loss
            return Err(TestHarnessError::NetworkError(
                "Simulated packet loss".to_string(),
            ));
        }

        Ok(())
    }

    /// Simulate latency
    async fn simulate_latency(&self, config: &LatencyConfig) -> Result<(), TestHarnessError> {
        let mut rng = rand::thread_rng();

        // Calculate latency with jitter
        let base_latency = if config.min_ms == config.max_ms {
            config.min_ms
        } else {
            rng.gen_range(config.min_ms..=config.max_ms)
        };

        let jitter = if config.jitter_ms > 0 {
            rng.gen_range(0..=config.jitter_ms)
        } else {
            0
        };

        let latency = base_latency + jitter;

        // Sleep for the calculated latency
        sleep(Duration::from_millis(latency)).await;

        Ok(())
    }

    /// Simulate bandwidth limitations
    async fn simulate_bandwidth(
        &self,
        config: &BandwidthConfig,
        data_size: usize,
    ) -> Result<(), TestHarnessError> {
        if config.kbps == 0 {
            return Ok(());
        }

        // Calculate delay based on bandwidth and data size
        // Formula: delay (ms) = (data_size * 8) / (bandwidth * 1000) * 1000
        // This converts data_size from bytes to bits, and bandwidth from kbps to bps
        let delay_ms = (data_size as f64 * 8.0) / (config.kbps as f64) * 1000.0;

        // Sleep for the calculated delay
        sleep(Duration::from_millis(delay_ms as u64)).await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_simulator() {
        // Create a network simulator with a custom configuration
        let config = NetworkSimulatorConfig {
            enabled: true,
            condition_type: NetworkConditionType::Custom,
            latency: LatencyConfig {
                enabled: true,
                min_ms: 50,
                max_ms: 50,
                jitter_ms: 0,
            },
            packet_loss: PacketLossConfig {
                enabled: false,
                probability: 0.0,
                burst_size: 1,
            },
            bandwidth: BandwidthConfig {
                enabled: false,
                kbps: 0,
            },
        };

        let simulator = NetworkSimulator::new(config);

        // Measure the time it takes to simulate network conditions
        let start = std::time::Instant::now();
        simulator.simulate(1000).await.unwrap();
        let elapsed = start.elapsed();

        // Check that the latency was simulated
        assert!(elapsed.as_millis() >= 50);
    }

    #[tokio::test]
    async fn test_network_condition_types() {
        // Create a network simulator
        let simulator = NetworkSimulator::new(NetworkSimulatorConfig::default());

        // Test setting different network condition types
        simulator
            .set_condition_type(NetworkConditionType::Good)
            .await;
        let config = simulator.config().await;
        assert_eq!(config.condition_type, NetworkConditionType::Good);
        assert!(config.latency.enabled);
        assert!(config.packet_loss.enabled);
        assert!(config.bandwidth.enabled);

        simulator
            .set_condition_type(NetworkConditionType::Perfect)
            .await;
        let config = simulator.config().await;
        assert_eq!(config.condition_type, NetworkConditionType::Perfect);
        assert!(!config.latency.enabled);
        assert!(!config.packet_loss.enabled);
        assert!(!config.bandwidth.enabled);
    }
}
