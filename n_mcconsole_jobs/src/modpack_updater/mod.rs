pub mod check_version;
pub mod list_versions;
pub mod log_reader;
pub mod update;

const HELPER: &str = "/usr/local/sbin/mcconsole-modpack-update";
const TAG: &str = "mcconsole-modpack-update";
const MANIFEST: &str = "https://downloads.gtnewhorizons.com/versions.json";
const VERSION_FILE: &str = "/opt/minecraft/.mcconsole-pack-version";
