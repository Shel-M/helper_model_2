#![allow(unused)]

use serde::Serialize;
use sqlx::{query, query_as, QueryBuilder};
use tracing::info;

use crate::api::user::UpdateUser;

#[derive(Debug, Serialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub discord_tag: Option<String>,
}

impl User {
    pub fn new(name: &str, discord_tag: Option<String>) -> Self {
        Self {
            id: 0,
            name: name.into(),
            discord_tag,
        }
    }

    pub async fn insert(mut self, db: &crate::DB) -> Result<Self, sqlx::error::Error> {
        let result = query!(
            "insert into person (name, discord_tag) values (?1, ?2)",
            self.name,
            self.discord_tag
        )
        .execute(db)
        .await?
        .last_insert_rowid();

        self.id = result;
        info!("{result:?}");
        Ok(self)
    }

    pub async fn get_by_id(db: &crate::DB, id: i64) -> sqlx::Result<Self> {
        query_as!(
            Self,
            "select id, name, discord_tag from person where id = ?1 limit 1",
            id
        )
        .fetch_one(db)
        .await
    }

    pub async fn get_by_name(db: &crate::DB, name: &str) -> sqlx::Result<Vec<Self>> {
        query_as!(
            Self,
            "select id, name, discord_tag from person where name = ?1 limit 1",
            name
        )
        .fetch_all(db)
        .await
    }

    pub async fn update(self, db: &crate::DB, update: UpdateUser) -> sqlx::Result<()> {
        // Guard pointless update calls
        if let (None, None) = (&update.name, &update.discord_tag) {
            return Ok(());
        }

        let mut query = QueryBuilder::new("update person set");

        let mut query_has_effect = false;

        if let Some(name) = update.name {
            if name != self.name {
                query_has_effect = true;
                query.push(" name = ");
                query.push_bind(name);
            }
        }
        if let Some(discord_tag) = update.discord_tag {
            if !matches!(self.discord_tag, Some(discord_tag)) {
                if query_has_effect {
                    query.push(",");
                } else {
                    query_has_effect = true;
                }
                query.push(" discord_tag = ");
                query.push_bind(discord_tag);
            }
        }

        query.push(" where id = ");
        query.push_bind(update.id);

        if query_has_effect {
            query.build().execute(db).await?;
        }

        Ok(())
    }

    pub async fn delete(self, db: &crate::DB) -> sqlx::Result<()> {
        query!("delete from person where id = ?1", self.id)
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn delete_ref(&self, db: &crate::DB) -> sqlx::Result<()> {
        query!("delete from person where id = ?1", self.id)
            .execute(db)
            .await?;
        // Cascade over the relationships
        query!("delete from chores_persons where person_id = ?1", self.id)
            .execute(db)
            .await?;
        Ok(())
    }
}

pub struct Chore {
    id: i32,
    pub name: String,
    pub desc: String,
    pub frequency: i32,
    pub discord_channel: String,
}

pub struct Assignment {
    id: i32,
    chore_id: i32,
    person_id: i32,
    pub assignment_date: i32,
    pub reminder_date: i32,
    pub completion_date: i32,
    completed_person: i32,
}
