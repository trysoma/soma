use tokio_rustls::rustls::crypto::CryptoProvider;

pub fn configure_crypto_provider() -> Result<(), anyhow::Error> {
    if CryptoProvider::get_default().is_some() {
        return Ok(());
    } else {
        tokio_rustls::rustls::crypto::ring::default_provider()
            .install_default()
            .map_err(|e| anyhow::anyhow!("Failed to install rustls crypto provider: {e:?}"))?;
    }

    Ok(())
}
