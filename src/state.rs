use std::error::Error;

use crate::https::{HttpsClient, ClientBuilder};
use crate::Args;
//use crate::error::Error as RestError;

type BoxResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Clone, Debug)]
pub struct State {
    pub client: HttpsClient,
}

impl State {
    pub async fn new(args: Args) -> BoxResult<Self> {

        let client = ClientBuilder::new().timeout(args.timeout).build()?;

        Ok(State {
            client,
        })
    }
}
