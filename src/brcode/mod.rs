use crate::brcode::emv::EmvCode;

mod emv;
pub struct BrCode {
    key: String,
    amount: f64,
    merchant_name: String,
    merchant_city: String,
    // transaction_id: String,
}

impl BrCode {
    pub fn new(key: String, amount: f64, name: String, city: String) -> Self {
        BrCode {
            key,
            amount,
            merchant_name: name,
            // transaction_id: id,
            merchant_city: city,
        }
    }

    pub fn to_emv_code(&self) -> EmvCode {
        EmvCode::new(
            "br.gov.bcb.pix".into(),
            self.key.clone(),
            "0000".into(),
            "986".into(),
            self.amount,
            "BR".into(),
            self.merchant_name.clone(),
            self.merchant_city.clone(),
            "***".into(),
        )
    }
}

