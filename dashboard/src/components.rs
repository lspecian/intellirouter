//! Dashboard components
//!
//! This module defines the components used to build the dashboard UI.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::data::{Status, StatusLevel};

/// Dashboard component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardComponent {
    /// ID
    pub id: String,
    /// Title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Type
    pub component_type: ComponentType,
    /// Data
    pub data: ComponentData,
    /// Status
    pub status: Status,
    /// Options
    pub options: HashMap<String, String>,
    /// Width (1-12, for grid layout)
    pub width: u8,
    /// Height (in rows, for grid layout)
    pub height: u8,
    /// Order (for sorting)
    pub order: u32,
}

/// Component type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    /// Card
    Card,
    /// Chart
    Chart,
    /// Table
    Table,
    /// Status
    Status,
    /// Metric
    Metric,
    /// List
    List,
}

/// Component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentData {
    /// Chart data
    Chart {
        /// Chart type
        chart_type: ChartType,
        /// Labels
        labels: Vec<String>,
        /// Datasets
        datasets: Vec<ChartDataset>,
    },
    /// Table data
    Table {
        /// Headers
        headers: Vec<String>,
        /// Rows
        rows: Vec<Vec<String>>,
    },
    /// Status data
    Status {
        /// Status
        status: Status,
        /// Details
        details: Option<String>,
    },
    /// Metric data
    Metric {
        /// Value
        value: String,
        /// Unit
        unit: Option<String>,
        /// Previous value
        previous_value: Option<String>,
        /// Change
        change: Option<f64>,
        /// Change is positive
        change_is_positive: Option<bool>,
    },
    /// List data
    List {
        /// Items
        items: Vec<ListItem>,
    },
}

/// Chart type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartType {
    /// Line chart
    Line,
    /// Bar chart
    Bar,
    /// Pie chart
    Pie,
    /// Doughnut chart
    Doughnut,
    /// Radar chart
    Radar,
    /// Polar area chart
    PolarArea,
    /// Bubble chart
    Bubble,
    /// Scatter chart
    Scatter,
}

/// Chart dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataset {
    /// Label
    pub label: String,
    /// Data
    pub data: Vec<f64>,
    /// Background color
    pub background_color: Option<String>,
    /// Border color
    pub border_color: Option<String>,
    /// Fill
    pub fill: Option<bool>,
}

/// List item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    /// Title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Status
    pub status: Option<Status>,
    /// Icon
    pub icon: Option<String>,
    /// Link
    pub link: Option<String>,
}

/// Create a metric component
pub fn create_metric_component(
    id: &str,
    title: &str,
    value: &str,
    unit: Option<&str>,
    previous_value: Option<&str>,
    change: Option<f64>,
    status_level: StatusLevel,
    width: u8,
    height: u8,
    order: u32,
) -> DashboardComponent {
    let change_is_positive = change.map(|c| c > 0.0);

    DashboardComponent {
        id: id.to_string(),
        title: title.to_string(),
        description: None,
        component_type: ComponentType::Metric,
        data: ComponentData::Metric {
            value: value.to_string(),
            unit: unit.map(|u| u.to_string()),
            previous_value: previous_value.map(|p| p.to_string()),
            change,
            change_is_positive,
        },
        status: Status {
            level: status_level,
            message: title.to_string(),
            details: None,
            timestamp: chrono::Utc::now(),
        },
        options: HashMap::new(),
        width,
        height,
        order,
    }
}

/// Create a chart component
pub fn create_chart_component(
    id: &str,
    title: &str,
    chart_type: ChartType,
    labels: Vec<String>,
    datasets: Vec<ChartDataset>,
    status_level: StatusLevel,
    width: u8,
    height: u8,
    order: u32,
) -> DashboardComponent {
    DashboardComponent {
        id: id.to_string(),
        title: title.to_string(),
        description: None,
        component_type: ComponentType::Chart,
        data: ComponentData::Chart {
            chart_type,
            labels,
            datasets,
        },
        status: Status {
            level: status_level,
            message: title.to_string(),
            details: None,
            timestamp: chrono::Utc::now(),
        },
        options: HashMap::new(),
        width,
        height,
        order,
    }
}

/// Create a table component
pub fn create_table_component(
    id: &str,
    title: &str,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    status_level: StatusLevel,
    width: u8,
    height: u8,
    order: u32,
) -> DashboardComponent {
    DashboardComponent {
        id: id.to_string(),
        title: title.to_string(),
        description: None,
        component_type: ComponentType::Table,
        data: ComponentData::Table { headers, rows },
        status: Status {
            level: status_level,
            message: title.to_string(),
            details: None,
            timestamp: chrono::Utc::now(),
        },
        options: HashMap::new(),
        width,
        height,
        order,
    }
}

/// Create a status component
pub fn create_status_component(
    id: &str,
    title: &str,
    status: Status,
    details: Option<String>,
    width: u8,
    height: u8,
    order: u32,
) -> DashboardComponent {
    DashboardComponent {
        id: id.to_string(),
        title: title.to_string(),
        description: None,
        component_type: ComponentType::Status,
        data: ComponentData::Status {
            status: status.clone(),
            details,
        },
        status,
        options: HashMap::new(),
        width,
        height,
        order,
    }
}

/// Create a list component
pub fn create_list_component(
    id: &str,
    title: &str,
    items: Vec<ListItem>,
    status_level: StatusLevel,
    width: u8,
    height: u8,
    order: u32,
) -> DashboardComponent {
    DashboardComponent {
        id: id.to_string(),
        title: title.to_string(),
        description: None,
        component_type: ComponentType::List,
        data: ComponentData::List { items },
        status: Status {
            level: status_level,
            message: title.to_string(),
            details: None,
            timestamp: chrono::Utc::now(),
        },
        options: HashMap::new(),
        width,
        height,
        order,
    }
}
