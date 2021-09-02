use crate::{github::api::repositories_query::RepositoriesQueryUserRepositories, DbPool};
use anyhow::{bail, Result};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Repository {
    name: String,
    repository: String,
    license: String,
    stars: i32,
    primary_language: String,
    languages: Vec<String>,
    created_at: String,
}

impl Repository {
    async fn _upsert(&self, db: DbPool) -> Result<()> {
        let res = sqlx::query!(
            r#"
insert into repository (name, name_with_owner, license, stars, primary_language, languages, created_at)
values ($1, $2, $3, $4, $5, $6, $7)
on conflict (name)
do update
set name = $1,
    name_with_owner = $2,
    license = $3,
    stars = $4,
    primary_language = $5,
    languages = $6,
    created_at = $7"#,
		self.name,
		self.repository,
		self.license,
		self.stars,
		self.primary_language,
		&self.languages,
		self.created_at
        )
        .execute(&db)
        .await?;

        if res.rows_affected() > 0 {
            Ok(())
        } else {
            bail!("Did not upsert any repository")
        }
    }

    fn _from_query(repositories: &RepositoriesQueryUserRepositories) -> Result<Vec<Repository>> {
        if let Some(nodes) = &repositories.nodes {
            let things: std::result::Result<Vec<_>, _> = nodes
                .iter()
                .map(|node| {
                    if let Some(repo) = node {
                        return Ok(Repository {
                            name: repo.name.clone(),
                            repository: repo.name_with_owner.clone(),
                            license: repo
                                .license_info
                                .as_ref()
                                .map(|l| l.name.clone())
                                .unwrap_or_else(|| "None".to_string()),
                            stars: repo.stargazer_count as i32,
                            primary_language: repo
                                .primary_language
                                .as_ref()
                                .map(|l| l.name.clone())
                                .unwrap_or_else(|| "Unknown".to_string()),
                            languages: vec![],
                            created_at: repo.created_at.clone(),
                        });
                    } else {
                        Err(())
                    }
                })
                .collect();

            return match things {
                Ok(repos) => Ok(repos),
                Err(_) => bail!("Could not convert repositories"),
            };
        };

        bail!("Expected Some(repositories), but got None")
    }
}
