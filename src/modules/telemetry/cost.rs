use std::collections::HashMap;
use std::sync::RwLock;

/// Cost calculator for LLM API calls
pub struct CostCalculator {
    /// Cost per 1K tokens for input (prompt) by model
    input_costs: RwLock<HashMap<String, f64>>,
    /// Cost per 1K tokens for output (completion) by model
    output_costs: RwLock<HashMap<String, f64>>,
}

impl CostCalculator {
    /// Create a new cost calculator with default costs
    pub fn new() -> Self {
        let mut input_costs = HashMap::new();
        let mut output_costs = HashMap::new();

        // OpenAI models
        input_costs.insert("gpt-4".to_string(), 0.03);
        output_costs.insert("gpt-4".to_string(), 0.06);

        input_costs.insert("gpt-4-32k".to_string(), 0.06);
        output_costs.insert("gpt-4-32k".to_string(), 0.12);

        input_costs.insert("gpt-3.5-turbo".to_string(), 0.0015);
        output_costs.insert("gpt-3.5-turbo".to_string(), 0.002);

        input_costs.insert("gpt-3.5-turbo-16k".to_string(), 0.003);
        output_costs.insert("gpt-3.5-turbo-16k".to_string(), 0.004);

        // Anthropic models
        input_costs.insert("claude-2".to_string(), 0.01102);
        output_costs.insert("claude-2".to_string(), 0.03268);

        input_costs.insert("claude-instant-1".to_string(), 0.00163);
        output_costs.insert("claude-instant-1".to_string(), 0.00551);

        // Default for unknown models
        input_costs.insert("default".to_string(), 0.001);
        output_costs.insert("default".to_string(), 0.002);

        Self {
            input_costs: RwLock::new(input_costs),
            output_costs: RwLock::new(output_costs),
        }
    }

    /// Add or update cost for a model
    pub fn set_model_cost(
        &self,
        model_id: &str,
        input_cost: f64,
        output_cost: f64,
    ) -> Result<(), String> {
        let mut input_costs = self.input_costs.write().map_err(|e| e.to_string())?;
        let mut output_costs = self.output_costs.write().map_err(|e| e.to_string())?;

        input_costs.insert(model_id.to_string(), input_cost);
        output_costs.insert(model_id.to_string(), output_cost);

        Ok(())
    }

    /// Calculate the cost of an LLM API call
    pub fn calculate_cost(
        &self,
        model_id: &str,
        prompt_tokens: usize,
        completion_tokens: usize,
    ) -> Result<f64, String> {
        let input_costs = self.input_costs.read().map_err(|e| e.to_string())?;
        let output_costs = self.output_costs.read().map_err(|e| e.to_string())?;

        let input_cost = input_costs
            .get(model_id)
            .unwrap_or_else(|| input_costs.get("default").unwrap());
        let output_cost = output_costs
            .get(model_id)
            .unwrap_or_else(|| output_costs.get("default").unwrap());

        let prompt_cost = (*input_cost * prompt_tokens as f64) / 1000.0;
        let completion_cost = (*output_cost * completion_tokens as f64) / 1000.0;

        Ok(prompt_cost + completion_cost)
    }
}

impl Default for CostCalculator {
    fn default() -> Self {
        Self::new()
    }
}
