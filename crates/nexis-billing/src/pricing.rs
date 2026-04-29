//! Model pricing table and cost calculation.

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Pricing info for a model.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelPricing {
    pub model: String,
    pub prompt_per_1m: f64,
    pub completion_per_1m: f64,
}

/// Static pricing table (built-in).
fn static_pricing() -> Vec<ModelPricing> {
    vec![
        ModelPricing {
            model: "gpt-4o".into(),
            prompt_per_1m: 2.5,
            completion_per_1m: 10.0,
        },
        ModelPricing {
            model: "gpt-4o-mini".into(),
            prompt_per_1m: 0.15,
            completion_per_1m: 0.6,
        },
        ModelPricing {
            model: "gpt-4-turbo".into(),
            prompt_per_1m: 10.0,
            completion_per_1m: 30.0,
        },
        ModelPricing {
            model: "gpt-3.5-turbo".into(),
            prompt_per_1m: 0.5,
            completion_per_1m: 1.5,
        },
        ModelPricing {
            model: "claude-sonnet-4-20250514".into(),
            prompt_per_1m: 3.0,
            completion_per_1m: 15.0,
        },
        ModelPricing {
            model: "claude-haiku-4-20250414".into(),
            prompt_per_1m: 0.8,
            completion_per_1m: 4.0,
        },
    ]
}

fn pricing_map() -> &'static HashMap<String, ModelPricing> {
    static MAP: OnceLock<HashMap<String, ModelPricing>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut map: HashMap<String, ModelPricing> = static_pricing()
            .into_iter()
            .chain(load_overrides())
            .map(|p| (p.model.clone(), p))
            .collect();

        // Apply overrides from NEXIS_PRICING_OVERRIDE env
        if let Ok(path) = std::env::var("NEXIS_PRICING_OVERRIDE") {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<Vec<ModelPricing>>(&content) {
                    Ok(overrides) => {
                        for p in overrides {
                            tracing::info!(model = %p.model, "Pricing override applied");
                            map.insert(p.model.clone(), p);
                        }
                    }
                    Err(e) => tracing::warn!("Failed to parse pricing override file: {e}"),
                },
                Err(e) => tracing::warn!("Failed to read pricing override file {path}: {e}"),
            }
        }
        map
    })
}

fn load_overrides() -> Vec<ModelPricing> {
    Vec::new()
}

/// Get pricing for a model, if known.
pub fn get_pricing(model: &str) -> Option<&'static ModelPricing> {
    pricing_map().get(model)
}

/// Calculate cost in USD for a model request.
/// Unknown models return 0.0.
pub fn calculate_cost(model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
    match get_pricing(model) {
        Some(p) => {
            (prompt_tokens as f64 * p.prompt_per_1m / 1_000_000.0)
                + (completion_tokens as f64 * p.completion_per_1m / 1_000_000.0)
        }
        None => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpt4o_cost() {
        // gpt-4o: prompt $2.5/1M, completion $10/1M
        let cost = calculate_cost("gpt-4o", 1_000_000, 500_000);
        assert!((cost - (2.5 + 5.0)).abs() < 0.001);
    }

    #[test]
    fn test_unknown_model_zero_cost() {
        assert_eq!(calculate_cost("unknown-model-xyz", 1000, 1000), 0.0);
    }

    #[test]
    fn test_zero_tokens() {
        assert_eq!(calculate_cost("gpt-4o-mini", 0, 0), 0.0);
    }

    #[test]
    fn test_claude_sonnet_cost() {
        let cost = calculate_cost("claude-sonnet-4-20250514", 1_000_000, 1_000_000);
        assert!((cost - 18.0).abs() < 0.001);
    }

    #[test]
    fn test_get_pricing_known_models() {
        assert!(get_pricing("gpt-4o").is_some());
        assert!(get_pricing("gpt-4o-mini").is_some());
        assert!(get_pricing("claude-haiku-4-20250414").is_some());
        assert!(get_pricing("nonexistent").is_none());
    }
}
