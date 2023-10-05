use std::io::Cursor;
use std::path::Path;
use reqwest::{Client};
use secrecy::{ExposeSecret, Secret};
use crate::domain::Uri;
use serde::Deserialize;
use qrcode::QrCode;
use tokio::fs;
use uuid::Uuid;
use base64::encode;
use image::{Luma, DynamicImage};

pub struct QRClient {
    http_client: Client,
    api_url: Uri,
    api_key: Secret<String>,
    base_url: Uri,
    base_image_path: String,
}

#[derive(Deserialize, Debug)]
pub struct ShortUrlResponse {
    pub hash: String,
    pub short_url: Uri,
    pub long_url: Uri,
}

impl QRClient {
    pub fn new(
        api_url: String,
        api_key: Secret<String>,
        base_url: String,
        base_image_path: String,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        let api_url = Uri::parse(api_url).unwrap();
        let base_url =  Uri::parse(base_url).unwrap();
        Self {
            http_client,
            api_url,
            api_key,
            base_url,
            base_image_path,
        }
    }

    pub async fn get_short_url(&self, token: String) -> Result<ShortUrlResponse, anyhow::Error> {
        let response = self.http_client.post(self.api_url.as_ref())
            .header("apikey", self.api_key.expose_secret())
            .body(format!("{}/{}", self.base_url, token))
            .send()
            .await
            .map_err(|e| { println!("{}", e); anyhow::anyhow!(e) })
            ?;

        let result: ShortUrlResponse = response.json().await?;
        println!("{:?}", result);

        Ok(result)
    }

    pub async fn generate_qr_code(
        &self,
        short_url: Uri,
        appt_id: Uuid,
        invintation_id: Uuid,
    ) -> Result<String, anyhow::Error> {
        let filename = format!("{}.png", invintation_id.to_string());
        let path_str = format!("{}/{}", self.base_image_path, appt_id.to_string());
        let path = Path::new(&path_str).join(filename);

        // Asynchronously ensure the directory exists
        fs::create_dir_all(path.parent().unwrap()).await?;

        let code = QrCode::new(short_url.into_inner().as_bytes())?;
        let image = code.render::<Luma<u8>>().build();

        image.save(&path)?;

        Ok(path.to_str().unwrap().to_string())
    }


    pub async fn generate_qr_code_base64(
        &self,
        short_url: Uri,
    ) -> Result<String, anyhow::Error> {

        let code = QrCode::new(short_url.into_inner().as_bytes()).unwrap();
        let image = code.render::<Luma<u8>>().build();

        // Convert the image into a byte vector using a Cursor
        let mut buffer = Cursor::new(Vec::new());
        let dynamic_image = DynamicImage::ImageLuma8(image);
        dynamic_image.write_to(&mut buffer, image::ImageOutputFormat::Png)?;

        // Encode the byte vector into a Base64 string
        let base64_image = encode(buffer.into_inner());

        Ok(base64_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};
    use secrecy::Secret;

    fn get_qr_client() -> QRClient {
        QRClient::new(
            format!("{}/short_url/hash", server_url()),  // Use the full URL when initializing the QRClient
            Secret::new("xxxxxxxxx".to_string()),
            "https://c443-94-45-49-154.ngrok-free.app/api/validations".to_string(),
            "./../testqr/".to_string(),
            std::time::Duration::from_millis(10000),
        )
    }

    #[tokio::test]
    async fn test_generate_qr_code() {
        let qr_client = get_qr_client();

        let result = qr_client.generate_qr_code(
            Uri::parse("https://google.com.ua".to_string()).unwrap(),
            Uuid::new_v4(),
            Uuid::new_v4()
        ).await;

        match &result {
            Ok(path) => println!("Saved QR code to: {}", path),
            Err(error) => eprintln!("Error: {}", error),
        }
    }

    #[tokio::test]
    async fn test_generate_qr_code_base64() {
        let qr_client = get_qr_client();

        let result = qr_client.generate_qr_code_base64(
            Uri::parse("https://google.com.ua".to_string()).unwrap(),
        ).await;

        match &result {
            Ok(path) => println!("Saved QR code to: {}", path),
            Err(error) => eprintln!("Error: {}", error),
        }
    }

    #[tokio::test]
    async fn test_get_short_url() {
        // Setup the mock server
        let m = mock("POST", "/short_url/hash")  // Only specify the path here
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"hash":"testhash", "short_url":"https://mocked.url/testhash", "long_url":"https://www.example.com"}"#)
            .create();

        // Initialize the QRClient
        let qr_client = get_qr_client();

        // Call the function
        let result = qr_client.get_short_url("token".to_string()).await;

        // Check that the mock was called
        m.assert();

        // Assert results based on your requirements
        assert!(result.is_ok());
    }
}



