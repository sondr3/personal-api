use crate::{github::api::repositories_query::RepositoriesQueryUserRepositories, DbPool};
use anyhow::{bail, Result};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

fn rename_language(language: String) -> String {
    match language.as_str() {
        "Dockerfile" => "Docker".to_string(),
        _ => language,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    name: String,
    owner: String,
    license: String,
    license_id: String,
    url: String,
    stars: i32,
    primary_language: String,
    languages: Vec<String>,
    created_at: String,
}

pub async fn get_repo(
    Path((owner, name)): Path<(String, String)>,
    Extension(db): Extension<DbPool>,
) -> impl IntoResponse {
    match sqlx::query_as!(
        Repository,
        "select * from repository where name = $1 and owner = $2",
        name,
        owner
    )
    .fetch_optional(&db)
    .await
    {
        Ok(Some(repo)) => Ok((StatusCode::FOUND, Json(repo))),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

impl Repository {
    pub async fn upsert(&self, db: &DbPool) -> Result<()> {
        let res = sqlx::query!(
            r#"
insert into repository (name, owner, license, stars, primary_language, languages, created_at)
values ($1, $2, $3, $4, $5, $6, $7)
on conflict (name, owner)
do update
set name = $1,
    owner = $2,
    license = $3,
    stars = $4,
    primary_language = $5,
    languages = $6,
    created_at = $7,
    url = $8,
    license_id = $9"#,
            self.name,
            self.owner,
            self.license,
            self.stars,
            self.primary_language,
            &self.languages,
            self.created_at,
            self.url,
            self.license_id
        )
        .execute(db)
        .await?;

        if res.rows_affected() > 0 {
            Ok(())
        } else {
            bail!("Did not upsert any repository")
        }
    }

    pub fn from_query(repositories: &RepositoriesQueryUserRepositories) -> Result<Vec<Repository>> {
        if let Some(nodes) = &repositories.nodes {
            let repos: std::result::Result<Vec<_>, _> = nodes
                .iter()
                .map(|node| {
                    if let Some(repo) = node {
                        let primary_language = repo
                            .primary_language
                            .as_ref()
                            .map(|l| l.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string());

                        return Ok(Repository {
                            name: repo.name.clone(),
                            owner: repo.owner.login.clone(),
                            license: repo
                                .license_info
                                .as_ref()
                                .map(|l| l.name.clone())
                                .unwrap_or_else(|| "None".to_string()),
                            license_id: repo
                                .license_info
                                .as_ref()
                                .map(|l| l.spdx_id.clone())
                                .unwrap_or_else(|| Some("None".to_string()))
                                .unwrap(),
                            url: repo.url.clone(),
                            stars: repo.stargazer_count as i32,
                            created_at: repo.created_at.clone(),
                            languages: repo
                                .languages
                                .as_ref()
                                .map(|l| {
                                    l.nodes
                                        .as_ref()
                                        .map(|l| {
                                            l.iter()
                                                .map(|lang| {
                                                    lang.as_ref()
                                                        .map(|l| rename_language(l.name.clone()))
                                                })
                                                .filter(|l| l != &Some(primary_language.clone()))
                                                .flatten()
                                                .collect::<Vec<String>>()
                                        })
                                        .unwrap_or_default()
                                })
                                .unwrap_or_default(),
                            primary_language,
                        });
                    } else {
                        Err(())
                    }
                })
                .collect();

            return match repos {
                Ok(repos) => Ok(repos),
                Err(_) => bail!("Could not convert repositories"),
            };
        };

        bail!("Expected Some(repositories), but got None")
    }
}
