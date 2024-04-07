pub struct Secrets {
    kafka_url: String,
    kafka_sasl_username: String,
    kafka_sasl_password: String,
}

impl<'a> Secrets {
    pub fn from_env(
        kafka_url: String,
        kafka_sasl_username: String,
        kafka_sasl_password: String,
    ) -> Self {
        Self {
            kafka_url,
            kafka_sasl_username,
            kafka_sasl_password,
        }
    }

    pub fn from_dotenvy() -> Self {
        let kafka_url = dotenvy::var("KAFKA_URL").unwrap();
        let kafka_sasl_username = dotenvy::var("KAFKA_SASL_USERNAME").unwrap();
        let kafka_sasl_password = dotenvy::var("KAFKA_SASL_PASSWORD").unwrap();

        Self {
            kafka_url,
            kafka_sasl_username,
            kafka_sasl_password,
        }
    }

    pub fn url(&'a self) -> &'a str {
        &self.kafka_url
    }
    pub fn username(&'a self) -> &'a str {
        &self.kafka_sasl_username
    }
    pub fn password(&'a self) -> &'a str {
        &self.kafka_sasl_password
    }
}
