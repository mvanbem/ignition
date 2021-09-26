pub struct RpcClient {
    service_name: String,
}

impl RpcClient {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }
}
