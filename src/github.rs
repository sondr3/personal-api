use crate::github::{
    contributions_query::ContributionsQueryUserContributionsCollection,
    repositories_query::RepositoriesQueryUserRepositories,
};
use anyhow::Result;
use graphql_client::*;
use serde::Serialize;

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

impl Default for GitHub {
    fn default() -> Self {
        Self::new()
    }
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

    async fn query<V: Serialize, T: GraphQLQuery>(
        &mut self,
        token: &str,
        query: QueryBody<V>,
    ) -> Result<T::ResponseData> {
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

        let body: Response<T::ResponseData> = res.json().await?;
        Ok(body.data.expect("missing response data"))
    }

    async fn query_repositories(&mut self, login: &str, token: &str) -> Result<()> {
        let query = RepositoriesQuery::build_query(repositories_query::Variables {
            login: login.to_string(),
        });

        let res: repositories_query::ResponseData = self
            .query::<repositories_query::Variables, RepositoriesQuery>(token, query)
            .await?;

        self.repositories = Some(res.user.unwrap().repositories);

        Ok(())
    }

    async fn query_contributions(&mut self, login: &str, token: &str) -> Result<()> {
        let query = ContributionsQuery::build_query(contributions_query::Variables {
            login: login.to_string(),
        });

        let res: contributions_query::ResponseData = self
            .query::<contributions_query::Variables, ContributionsQuery>(token, query)
            .await?;

        self.contributions = Some(res.user.unwrap().contributions_collection);

        Ok(())
    }
}
