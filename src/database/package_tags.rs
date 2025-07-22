use crate::models::{NewPackageTag, PackageTag, UpdatePackageTag};
use crate::schema::package_tags;
use diesel::prelude::*;
use std::collections::HashMap;

impl crate::database::DatabaseService {
    /// Create or update a package tag
    pub fn create_or_update_package_tag(
        &self,
        package_name: &str,
        tag_name: &str,
        version: &str,
    ) -> Result<PackageTag, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Try to update existing tag first
        let update_result = diesel::update(package_tags::table)
            .filter(package_tags::package_name.eq(package_name))
            .filter(package_tags::tag_name.eq(tag_name))
            .set(&UpdatePackageTag {
                version: version.to_string(),
                updated_at: chrono::Utc::now().naive_utc(),
            })
            .get_result::<PackageTag>(&mut conn);

        match update_result {
            Ok(tag) => Ok(tag),
            Err(diesel::result::Error::NotFound) => {
                // Tag doesn't exist, create it
                let new_tag = NewPackageTag::new(
                    package_name.to_string(),
                    tag_name.to_string(),
                    version.to_string(),
                );

                diesel::insert_into(package_tags::table)
                    .values(&new_tag)
                    .get_result::<PackageTag>(&mut conn)
            }
            Err(e) => Err(e),
        }
    }

    /// Get all tags for a package
    pub fn get_package_tags(
        &self,
        package_name: &str,
    ) -> Result<Vec<PackageTag>, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        package_tags::table
            .filter(package_tags::package_name.eq(package_name))
            .load::<PackageTag>(&mut conn)
    }

    /// Get tags as a HashMap for metadata generation
    pub fn get_package_tags_map(
        &self,
        package_name: &str,
    ) -> Result<HashMap<String, String>, diesel::result::Error> {
        let tags = self.get_package_tags(package_name)?;
        let mut tags_map = HashMap::new();

        for tag in tags {
            tags_map.insert(tag.tag_name, tag.version);
        }

        Ok(tags_map)
    }

    /// Delete a specific tag
    pub fn delete_package_tag(
        &self,
        package_name: &str,
        tag_name: &str,
    ) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(package_tags::table)
            .filter(package_tags::package_name.eq(package_name))
            .filter(package_tags::tag_name.eq(tag_name))
            .execute(&mut conn)
    }

    /// Delete all tags for a package
    pub fn delete_all_package_tags(
        &self,
        package_name: &str,
    ) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(package_tags::table)
            .filter(package_tags::package_name.eq(package_name))
            .execute(&mut conn)
    }
}
