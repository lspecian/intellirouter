//! Condition Evaluator
//!
//! This module provides functionality for evaluating conditions in chains.

use regex::Regex;
use std::collections::HashMap;

use crate::modules::chain_engine::context::ChainContext;
use crate::modules::chain_engine::definition::{ComparisonOperator, Condition};
use crate::modules::chain_engine::error::{ChainError, ChainResult};

/// Condition evaluator for chain execution
#[derive(Clone, Default)]
pub struct ConditionEvaluator;

impl ConditionEvaluator {
    /// Create a new condition evaluator
    pub fn new() -> Self {
        Self
    }

    /// Evaluate a condition
    pub fn evaluate_condition(
        &self,
        condition: &Condition,
        context: &ChainContext,
    ) -> ChainResult<bool> {
        match condition {
            Condition::Equals { variable, value } => {
                let var_value = context.variables.get(variable).ok_or_else(|| {
                    ChainError::VariableNotFound(format!("Variable not found: {}", variable))
                })?;

                Ok(var_value == value)
            }
            Condition::Contains { variable, value } => {
                let var_value = context.variables.get(variable).ok_or_else(|| {
                    ChainError::VariableNotFound(format!("Variable not found: {}", variable))
                })?;

                // Check if the variable contains the value
                match var_value {
                    serde_json::Value::String(s) => Ok(s.contains(&value.to_string())),
                    serde_json::Value::Array(arr) => Ok(arr.contains(value)),
                    serde_json::Value::Object(obj) => {
                        if let serde_json::Value::String(key) = value {
                            Ok(obj.contains_key(key))
                        } else {
                            Ok(false)
                        }
                    }
                    _ => Ok(false),
                }
            }
            Condition::Regex { variable, pattern } => {
                let var_value = context.variables.get(variable).ok_or_else(|| {
                    ChainError::VariableNotFound(format!("Variable not found: {}", variable))
                })?;

                let value_str = match var_value {
                    serde_json::Value::String(s) => s,
                    _ => {
                        return Err(ChainError::ValidationError(
                            "Cannot apply regex to non-string value".to_string(),
                        ));
                    }
                };

                let re = Regex::new(pattern).map_err(|e| {
                    ChainError::ValidationError(format!("Invalid regex pattern: {}", e))
                })?;

                Ok(re.is_match(value_str))
            }
            Condition::GreaterThan { variable, value } => {
                let var_value = context.variables.get(variable).ok_or_else(|| {
                    ChainError::VariableNotFound(format!("Variable not found: {}", variable))
                })?;

                match (var_value, value) {
                    (serde_json::Value::Number(n1), serde_json::Value::Number(n2)) => {
                        if let (Some(f1), Some(f2)) = (n1.as_f64(), n2.as_f64()) {
                            Ok(f1 > f2)
                        } else {
                            Ok(false)
                        }
                    }
                    _ => Ok(false),
                }
            }
            Condition::LessThan { variable, value } => {
                let var_value = context.variables.get(variable).ok_or_else(|| {
                    ChainError::VariableNotFound(format!("Variable not found: {}", variable))
                })?;

                match (var_value, value) {
                    (serde_json::Value::Number(n1), serde_json::Value::Number(n2)) => {
                        if let (Some(f1), Some(f2)) = (n1.as_f64(), n2.as_f64()) {
                            Ok(f1 < f2)
                        } else {
                            Ok(false)
                        }
                    }
                    _ => Ok(false),
                }
            }
            Condition::Comparison {
                left,
                operator,
                right,
            } => {
                let left_value = self.resolve_value(left, context)?;
                let right_value = self.resolve_value(right, context)?;

                self.evaluate_comparison(operator, &left_value, &right_value)
            }
            Condition::Expression { expression } => self.evaluate_expression(expression, context),
            Condition::And { conditions } => {
                for condition in conditions {
                    if !self.evaluate_condition(condition, context)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Condition::Or { conditions } => {
                for condition in conditions {
                    if self.evaluate_condition(condition, context)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Condition::Not { condition } => {
                let result = self.evaluate_condition(condition, context)?;
                Ok(!result)
            }
            Condition::Custom { .. } => {
                // Custom conditions are not implemented in this example
                Err(ChainError::ValidationError(
                    "Custom conditions are not implemented".to_string(),
                ))
            }
        }
    }

    /// Evaluate an expression
    fn evaluate_expression(&self, expression: &str, context: &ChainContext) -> ChainResult<bool> {
        // Simple expression evaluation
        // This is a simplified implementation that only handles variable references
        // and basic boolean expressions

        // Check if the expression is a variable reference
        if expression.starts_with("{{") && expression.ends_with("}}") {
            let var_name = expression[2..expression.len() - 2].trim();
            let var_value = context.variables.get(var_name).ok_or_else(|| {
                ChainError::VariableNotFound(format!("Variable not found: {}", var_name))
            })?;

            let value_str = match var_value {
                serde_json::Value::String(s) => s.clone(),
                _ => var_value.to_string(),
            };

            return match value_str.parse::<bool>() {
                Ok(value) => Ok(value),
                Err(_) => {
                    if value_str == "true" {
                        Ok(true)
                    } else if value_str == "false" {
                        Ok(false)
                    } else {
                        Ok(!value_str.is_empty())
                    }
                }
            };
        }

        // Handle basic boolean expressions
        let expr = expression.trim().to_lowercase();
        match expr.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => {
                // This is a simplified implementation
                // In a real implementation, you would parse and evaluate the expression
                Err(ChainError::ValidationError(format!(
                    "Unsupported expression: {}",
                    expression
                )))
            }
        }
    }

    /// Evaluate a comparison
    fn evaluate_comparison(
        &self,
        operator: &ComparisonOperator,
        left: &serde_json::Value,
        right: &serde_json::Value,
    ) -> ChainResult<bool> {
        match operator {
            ComparisonOperator::Eq => Ok(left == right),
            ComparisonOperator::Ne => Ok(left != right),
            ComparisonOperator::Lt => {
                let left_num = self.to_number(left)?;
                let right_num = self.to_number(right)?;
                Ok(left_num < right_num)
            }
            ComparisonOperator::Lte => {
                let left_num = self.to_number(left)?;
                let right_num = self.to_number(right)?;
                Ok(left_num <= right_num)
            }
            ComparisonOperator::Gt => {
                let left_num = self.to_number(left)?;
                let right_num = self.to_number(right)?;
                Ok(left_num > right_num)
            }
            ComparisonOperator::Gte => {
                let left_num = self.to_number(left)?;
                let right_num = self.to_number(right)?;
                Ok(left_num >= right_num)
            }
            ComparisonOperator::Contains => {
                let left_str = self.to_string(left)?;
                let right_str = self.to_string(right)?;
                Ok(left_str.contains(&right_str))
            }
            ComparisonOperator::StartsWith => {
                let left_str = self.to_string(left)?;
                let right_str = self.to_string(right)?;
                Ok(left_str.starts_with(&right_str))
            }
            ComparisonOperator::EndsWith => {
                let left_str = self.to_string(left)?;
                let right_str = self.to_string(right)?;
                Ok(left_str.ends_with(&right_str))
            }
            ComparisonOperator::Matches => {
                let left_str = self.to_string(left)?;
                let right_str = self.to_string(right)?;
                let re = Regex::new(&right_str).map_err(|e| {
                    ChainError::ValidationError(format!("Invalid regex pattern: {}", e))
                })?;
                Ok(re.is_match(&left_str))
            }
        }
    }

    /// Resolve a value from a string
    fn resolve_value(&self, value: &str, context: &ChainContext) -> ChainResult<serde_json::Value> {
        // Check if the value is a variable reference
        if value.starts_with("{{") && value.ends_with("}}") {
            let var_name = value[2..value.len() - 2].trim();
            return context.variables.get(var_name).cloned().ok_or_else(|| {
                ChainError::VariableNotFound(format!("Variable not found: {}", var_name))
            });
        }

        // Try to parse as JSON
        match serde_json::from_str(value) {
            Ok(value) => Ok(value),
            Err(_) => Ok(serde_json::Value::String(value.to_string())),
        }
    }

    /// Convert a value to a number
    fn to_number(&self, value: &serde_json::Value) -> ChainResult<f64> {
        match value {
            serde_json::Value::Number(n) => n.as_f64().ok_or_else(|| {
                ChainError::ValidationError(format!("Cannot convert to number: {}", value))
            }),
            serde_json::Value::String(s) => s.parse::<f64>().map_err(|_| {
                ChainError::ValidationError(format!("Cannot convert to number: {}", s))
            }),
            _ => Err(ChainError::ValidationError(format!(
                "Cannot convert to number: {}",
                value
            ))),
        }
    }

    /// Convert a value to a string
    fn to_string(&self, value: &serde_json::Value) -> ChainResult<String> {
        match value {
            serde_json::Value::String(s) => Ok(s.clone()),
            _ => Ok(value.to_string()),
        }
    }
}
