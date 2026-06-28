pub mod create_user;
pub mod delete_user;
pub mod list_users;
pub mod set_role;

const HELPER: &str = "/usr/local/sbin/mcconsole-usermgmt";

pub enum Role {
    Viewer,
    Operator,
    Admin,
}

impl From<String> for Role {
    fn from(value: String) -> Self {
        if value == "viewer" {
            Role::Viewer
        } else if value == "operator" {
            Role::Operator
        } else if value == "admin" {
            Role::Admin
        } else {
            panic!("Role value {} not valid!", value)
        }
    }
}

impl From<Role> for String {
    fn from(value: Role) -> Self {
        match value {
            Role::Viewer => "viewer".into(),
            Role::Operator => "operator".into(),
            Role::Admin => "admin".into(),
        }
    }
}
