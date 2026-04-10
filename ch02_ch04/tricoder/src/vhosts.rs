use crate::{
    error::Error,
    model::{
        ScanTarget,
        ScanTargetType::{Domain, Ip},
    },
};

pub async fn enumerate() -> Result<Vec<ScanTarget>, Error> {
    Ok(vec![ScanTarget::new(Domain("".to_string()))])
}
