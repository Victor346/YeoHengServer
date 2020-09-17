use rusoto_core::Region;
use rusoto_s3::PutObjectRequest;
use rusoto_s3::util::PreSignedRequest;
use rusoto_core::credential::{DefaultCredentialsProvider, ProvideAwsCredentials};
use uuid::Uuid;

pub async fn create_presgigned_url(username: String, file_extension: String, folder: String) -> Result<(String, String), String>{
    let bucket_name = match std::env::var("S3_BUCKET") {
        Ok(bn) => bn,
        Err(e) => return Err("Error al obtener env".to_string()),
    };
    let file_uuid = Uuid::new_v4();
    let file_key = format!("{}/{}/{}.{}",
                           username,
                           folder,
                           file_uuid.to_hyphenated().to_string(),
                           file_extension);
    let req = PutObjectRequest {
        bucket: bucket_name,
        key: file_key.clone(),
        ..Default::default()
    };

    let credentials = DefaultCredentialsProvider::new()
        .unwrap()
        .credentials()
        .await
        .unwrap();

    let presigned_url = req.get_presigned_url(&Region::UsEast1, &credentials, &Default::default());
    Ok((presigned_url, file_key))
}