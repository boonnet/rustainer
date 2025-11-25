use crate::docker::DockerImageConfig;
use anyhow::Result;
use flate2::read::GzDecoder;
use oci_spec::image::ImageManifest;
use reqwest::RequestBuilder;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::io::{copy, Cursor};
use tar::Archive;
use tracing::info;

pub const IMAGE_PATH: &str = "/var/lib/rustainer/image";
pub const CONTAINER_PATH: &str = "/var/lib/rustainer/container";

pub fn store_manifest(
    image: &str,
    tag: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = format!("{}/{}/{}", IMAGE_PATH, image, tag);
    create_dir_all(&path)?;
    let manifest_path = format!("{}/{}", path, "manifest.json");
    let mut f = File::create(manifest_path)?;
    f.write_all(content.as_bytes())?;

    Ok(())
}

pub fn load_manifest(image: &str, tag: &str) -> Result<ImageManifest, Box<dyn std::error::Error>> {
    let path = format!("{}/{}/{}/manifest.json", IMAGE_PATH, image, tag);
    Ok(ImageManifest::from_file(path)?)
}

pub fn store_config(
    image: &str,
    tag: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = format!("{}/{}/{}", IMAGE_PATH, image, tag);
    create_dir_all(&path)?;
    let config_path = format!("{}/{}", path, "config.json");
    let mut f = File::create(config_path)?;
    f.write_all(content.as_bytes())?;

    Ok(())
}

pub fn load_config(
    image: &str,
    tag: &str,
) -> Result<DockerImageConfig, Box<dyn std::error::Error>> {
    let path = format!("{}/{}/{}/config.json", IMAGE_PATH, image, tag);
    let mut f = File::open(path)?;

    let mut data = String::new();
    f.read_to_string(&mut data)?;

    Ok(serde_json::from_str(&data)?)
}

pub async fn save_layer(
    image: &str,
    tag: &str,
    digest: &str,
    req: RequestBuilder,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = format!("{}/{}/{}/layers", IMAGE_PATH, image, tag);

    let resp = req.send().await?;
    let mut content = Cursor::new(resp.bytes().await?);

    let mut dest = File::create(format!("{}/{}", path, digest))?;
    copy(&mut content, &mut dest)?;

    Ok(())
}

pub fn extract_image(
    image: &str,
    tag: &str,
    name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let manifest = load_manifest(image, tag)?;
    let path = format!("{}/{}", CONTAINER_PATH, name);
    create_dir_all(path.clone())?;

    for layer in manifest.layers().iter() {
        let digest = layer.digest();
        let layer_path = format!("{}/{}/{}/layers/{}", IMAGE_PATH, image, tag, digest);

        let gz_decoder = GzDecoder::new(File::open(layer_path)?);
        let mut tar_archive = Archive::new(gz_decoder);

        tar_archive.unpack(path.clone())?;
        info!("Successfully extracted layer {}", digest);
    }

    Ok(path)
}

