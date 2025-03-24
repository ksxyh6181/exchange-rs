use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone,ToSchema)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub symbol: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: f64,
    pub status: OrderStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq,ToSchema)]
pub enum OrderType {
    #[serde(rename = "limit")]
    Limit,
    #[serde(rename = "market")]
    Market,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq,ToSchema)]
pub enum OrderSide {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq,ToSchema)]
pub enum OrderStatus {
    #[serde(rename = "new")]
    New,
    #[serde(rename = "partially_filled")]
    PartiallyFilled,
    #[serde(rename = "filled")]
    Filled,
    #[serde(rename = "canceled")]
    Canceled,
    #[serde(rename = "rejected")]
    Rejected,
}

#[derive(Debug, Serialize, Deserialize, Clone,ToSchema)]
pub struct Trade {
    pub id: String,
    pub order_id: String,
    pub price: f64,
    pub quantity: f64,
    pub commission: f64,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct RiskEvaluation {
    pub user_id: String,
    pub risk_level: RiskLevel,
    pub max_order_value: f64,
    pub max_daily_volume: f64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
pub enum RiskLevel {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
} 