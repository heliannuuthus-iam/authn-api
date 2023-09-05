use std::result::Result;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use tera::{Context, Error};

#[derive(Deserialize, Serialize)]
pub struct SmsConfig {
    pub id: i64,
    pub name: String,
    pub template: String,
    pub template_type: String,
}

#[derive(Builder)]
pub struct SmsContext {
    #[builder(setter(skip))]
    pub subject: String,
    pub title: String,
    pub template: String,
    pub context: Context,
    #[builder(setter(skip))]
    pub receiver: String,
}

impl From<SmsConfig> for SmsContext {
    fn from(value: SmsConfig) -> Self {
        SmsContextBuilder::default()
            .template(value.template)
            .title(value.name)
            .context(Context::new())
            .build()
            .unwrap()
    }
}

impl SmsContext {
    pub fn render(&mut self) -> Result<String, Error> {
        tera::Tera::one_off(&self.template, &self.context, true)
    }
}
