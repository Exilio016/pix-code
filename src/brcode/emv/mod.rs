use crc16::CCITT_FALSE;

#[allow(dead_code)]
pub enum EmvValue {
    Text(String),
    Emv(Box<EmvField>),
    Array(Vec<EmvField>),
}
pub struct EmvField {
    kind: u8,
    len: u8,
    value: EmvValue,
}

pub struct EmvCode(Vec<EmvField>);

impl EmvValue {
    pub fn len(&self) -> u8 {
        match self {
            EmvValue::Text(s) => s.chars().count() as u8,
            EmvValue::Emv(f) => 4 + f.len,
            EmvValue::Array(fields) => {
                let mut size = 0;
                for f in fields {
                    size += 4;
                    size += f.len;
                }
                size
            }
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            EmvValue::Text(str) => str.clone(),
            EmvValue::Emv(field) => field.to_string(),
            EmvValue::Array(fields) => {
                let mut str = String::new();
                for f in fields {
                    str.push_str(&f.to_string());
                }
                str
            }
        }
    }
}

impl EmvField {
    pub fn new(kind: u8, value: EmvValue) -> Self {
        EmvField {
            kind,
            len: value.len(),
            value,
        }
    }
    pub fn version() -> Self {
        EmvField::new(00, EmvValue::Text("01".into()))
    }
    pub fn merchant_account(gui: String, key: String) -> Self {
        let vec = vec![
            EmvField::new(00, EmvValue::Text(gui)),
            EmvField::new(01, EmvValue::Text(key)),
        ];
        let val = EmvValue::Array(vec);
        EmvField::new(26, val)
    }
    pub fn merchant_category(val: String) -> Self {
        EmvField::new(52, EmvValue::Text(val))
    }
    pub fn currency(val: String) -> Self {
        EmvField::new(53, EmvValue::Text(val))
    }
    pub fn amount(val: f64) -> Self {
        EmvField::new(54, EmvValue::Text(format!("{:.2}", val)))
    }
    pub fn country(val: String) -> Self {
        EmvField::new(58, EmvValue::Text(val))
    }
    pub fn merchant_name(val: String) -> Self {
        EmvField::new(59, EmvValue::Text(val))
    }
    pub fn merchant_city(val: String) -> Self {
        EmvField::new(60, EmvValue::Text(val))
    }
    pub fn additional_info(transaction_id: String) -> Self {
        let vec = vec![
            EmvField::new(05, EmvValue::Text(transaction_id)),
        ];
        let val = EmvValue::Array(vec);
        EmvField::new(62, val)
    }

    pub fn to_string(&self) -> String {
        format!("{:02}{:02}{}", self.kind, self.len, self.value.to_string())
    }
}

impl EmvCode {
    pub fn new(
        account_gui: String,
        account_key: String,
        merchant_category: String,
        currency: String,
        amount: f64,
        country: String,
        merchant_name: String,
        merchant_city: String,
        transaction_id: String,
    ) -> Self {
        let v = vec![
            EmvField::version(),
            EmvField::merchant_account(account_gui, account_key),
            EmvField::merchant_category(merchant_category),
            EmvField::currency(currency),
            EmvField::amount(amount),
            EmvField::country(country),
            EmvField::merchant_name(merchant_name),
            EmvField::merchant_city(merchant_city),
            EmvField::additional_info(transaction_id),
        ];
        EmvCode(v)
    }

    pub fn to_string(&self) -> String {
        let mut str = String::new();
        for field in &self.0 {
            str.push_str(&field.to_string());
        }
        str.push_str("6304");
        let crc = crc16::State::<CCITT_FALSE>::calculate(str.as_bytes());
        str.push_str(format!("{:X}", crc).as_str());
        return str;
    }
}
