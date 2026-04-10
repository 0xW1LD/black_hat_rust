use crate::{error::Error, model::ScanTarget};

pub async fn enumerate() -> Result<Vec<ScanTarget>, Error> {
    Ok(vec![ScanTarget::new()])
}
