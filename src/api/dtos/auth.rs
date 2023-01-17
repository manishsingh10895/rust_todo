#[derive(Debug, serde::Deserialize)]
pub struct LoginDTO {
    pub email: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LoginResponseDTO {
    pub id: String,
    pub email: String,
    pub token: String,
}

pub type SignupResponseDTO = LoginResponseDTO;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SignupRequestDTO {
    pub email: String,
    pub password: String,
    pub name: String,
}
