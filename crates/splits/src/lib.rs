mod config;
pub use config::Config;

mod split_provider_ext;
pub use split_provider_ext::{SplitMainContract, SplitProviderExt};

mod types;
pub use types::Account;

mod query;
pub use query::get_split_accounts;
