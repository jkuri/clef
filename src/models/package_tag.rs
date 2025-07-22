use crate::schema::package_tags;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug)]
#[diesel(table_name = package_tags)]
pub struct PackageTag {
    pub id: i32,
    pub package_name: String,
    pub tag_name: String,
    pub version: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = package_tags)]
pub struct NewPackageTag {
    pub package_name: String,
    pub tag_name: String,
    pub version: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl NewPackageTag {
    pub fn new(package_name: String, tag_name: String, version: String) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            package_name,
            tag_name,
            version,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = package_tags)]
pub struct UpdatePackageTag {
    pub version: String,
    pub updated_at: NaiveDateTime,
}
