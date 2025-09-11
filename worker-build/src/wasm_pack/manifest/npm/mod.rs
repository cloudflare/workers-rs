mod commonjs;
mod esmodules;
mod nomodules;
pub mod repository;

use serde::Serialize;

pub use self::commonjs::CommonJSPackage;
pub use self::esmodules::ESModulesPackage;
pub use self::nomodules::NoModulesPackage;

#[derive(Serialize)]
#[serde(untagged)]
#[allow(clippy::enum_variant_names)]
pub enum NpmPackage {
    CommonJSPackage(CommonJSPackage),
    ESModulesPackage(ESModulesPackage),
    NoModulesPackage(NoModulesPackage),
}
