use crate::github::{
    contributions_query::ContributionsQueryUserContributionsCollection,
    repositories_query::RepositoriesQueryUserRepositories,
};
use anyhow::Result;
use graphql_client::*;

type Date = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.docs.graphql",
    query_path = "queries/Contributions.graphql",
    response_derives = "Debug"
)]
struct ContributionsQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.docs.graphql",
    query_path = "queries/Repositories.graphql",
    response_derives = "Debug"
)]
struct RepositoriesQuery;

pub struct GitHub {
    pub contributions: Option<ContributionsQueryUserContributionsCollection>,
    pub repositories: Option<RepositoriesQueryUserRepositories>,
}

impl GitHub {
    pub fn new() -> Self {
        Self {
            contributions: None,
            repositories: None,
        }
    }

    pub async fn update(&mut self, login: &str, token: &str) -> Result<()> {
        self.query_contributions(login, token).await?;
        self.query_repositories(login, token).await?;

        Ok(())
    }

    async fn query_repositories(&mut self, login: &str, token: &str) -> Result<()> {
        let query = RepositoriesQuery::build_query(repositories_query::Variables {
            login: login.to_string(),
        });

        let client = reqwest::Client::builder()
            .user_agent(format!("sondr3/personal-api#{}", env!("CARGO_PKG_VERSION")))
            .build()?;

        let res = client
            .post("https://api.github.com/graphql")
            .bearer_auth(token)
            .json(&query)
            .send()
            .await?;

        res.error_for_status_ref()?;

        let body: Response<repositories_query::ResponseData> = res.json().await?;
        let data: repositories_query::ResponseData = body.data.expect("missing response data");

        self.repositories = Some(data.user.unwrap().repositories);

        Ok(())
    }

    async fn query_contributions(&mut self, login: &str, token: &str) -> Result<()> {
        let query = ContributionsQuery::build_query(contributions_query::Variables {
            login: login.to_string(),
        });

        let client = reqwest::Client::builder()
            .user_agent(format!("sondr3/personal-api#{}", env!("CARGO_PKG_VERSION")))
            .build()?;

        let res = client
            .post("https://api.github.com/graphql")
            .bearer_auth(token)
            .json(&query)
            .send()
            .await?;

        res.error_for_status_ref()?;

        let body: Response<contributions_query::ResponseData> = res.json().await?;
        let data: contributions_query::ResponseData = body.data.expect("missing response data");

        self.contributions = Some(data.user.unwrap().contributions_collection);

        Ok(())
    }
}
