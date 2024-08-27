use std::sync::Arc;

use auth_service::{Application, AppState, HashmapUserStore, HashsetBannedTokenStore};
use tokio::sync::RwLock;
use auth_service::utils::constants::prod;

#[tokio::main]
async fn main() {

    let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
    let banner_token_store =
        Arc::new(RwLock::new(HashsetBannedTokenStore::default()));

    let app_state = AppState::new(user_store, banner_token_store);

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
