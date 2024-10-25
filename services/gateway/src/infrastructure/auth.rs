#[derive(Clone)]
pub struct TokenService;

impl TokenService {
    pub fn new() -> Self {
        TokenService
    }

    pub async fn validate_token(&self, token: &str) -> bool {
        token == "test.valid.token"
    }
}
