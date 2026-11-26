use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LineItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sr_no: Option<u32>,
    pub description: String,
    
    // Frontend field aliases for seamless compatibility
    #[serde(default)]
    pub quality: String,
    #[serde(default)]
    pub name: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hsn_sac: Option<String>,
    #[serde(default)]
    pub design: String,
    #[serde(default)]
    pub shade: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    #[serde(default)]
    pub qty: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<f64>,
    #[serde(default)]
    pub price: String,
    
    #[serde(default)]
    pub disc: String,
    #[serde(default)]
    pub gst: String,
    #[serde(default)]
    pub tax: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
pub struct ExtractedFields {
    pub document_type: String,
    #[serde(default = "default_true")]
    pub is_valid_document: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_message: Option<String>,
    pub voucher_no: String,
    #[serde(default)]
    pub documentNumber: String,
    
    pub date: String,
    #[serde(default)]
    pub documentDate: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    
    pub party_name: String,
    #[serde(default)]
    pub party: String,
    #[serde(default, alias = "gstin", alias = "gstNo", alias = "party_gstin")]
    pub gst_no: String,
    #[serde(default)]
    pub supplier_gstin: String,
    #[serde(default)]
    pub company: String,
    #[serde(default)]
    pub broker: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtotal: Option<f64>,
    #[serde(default)]
    pub taxable: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_amount: Option<f64>,
    #[serde(default)]
    pub tax: String,
    
    #[serde(default)]
    pub cgst: String,
    #[serde(default)]
    pub sgst: String,
    #[serde(default)]
    pub igst: String,
    
    pub total_amount: String,
    #[serde(default)]
    pub totalAmount: String,
    
    #[serde(default)]
    pub rollsQty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_qty: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_in_words: Option<String>,
    
    pub items: Vec<LineItem>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResponse {
    pub success: bool,
    #[serde(default = "default_true")]
    pub is_valid_document: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub fields: ExtractedFields,
    pub raw_text: String,
    pub confidence: f32,
    pub processing_time_ms: u128,
    pub page_count: usize,
    pub source_type: String,
}
