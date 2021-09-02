use crate::github::{
    contributions_query::ContributionsQueryUserContributionsCollection,
    repositories_query::RepositoriesQueryUserRepositories,
};
use anyhow::Result;
use graphql_client::*;
use serde::Serialize;

type Date = String;
type DateTime = String;
#[allow(clippy::upper_case_acronyms)]
type URI = String;

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
    pub contributions: ContributionsQueryUserContributionsCollection,
    pub repositories: RepositoriesQueryUserRepositories,
}

async fn query_github<V: Serialize, T: GraphQLQuery>(
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

impl GitHub {
    pub async fn new(login: &str, token: &str) -> Result<GitHub> {
        let repositories = GitHub::query_repositories(login, token).await?;
        let contributions = GitHub::query_contributions(login, token).await?;

        Ok(GitHub {
            repositories,
            contributions,
        })
    }

    pub async fn _update(&mut self, login: &str, token: &str) -> Result<()> {
        self.contributions = GitHub::query_contributions(login, token).await?;
        self.repositories = GitHub::query_repositories(login, token).await?;

        Ok(())
    }

    async fn query_repositories(
        login: &str,
        token: &str,
    ) -> Result<RepositoriesQueryUserRepositories> {
        let query = RepositoriesQuery::build_query(repositories_query::Variables {
            login: login.to_string(),
        });

        let res: repositories_query::ResponseData =
            query_github::<repositories_query::Variables, RepositoriesQuery>(token, query).await?;

        Ok(res.user.unwrap().repositories)
    }

    async fn query_contributions(
        login: &str,
        token: &str,
    ) -> Result<ContributionsQueryUserContributionsCollection> {
        let query = ContributionsQuery::build_query(contributions_query::Variables {
            login: login.to_string(),
        });

        let res: contributions_query::ResponseData =
            query_github::<contributions_query::Variables, ContributionsQuery>(token, query)
                .await?;

        Ok(res.user.unwrap().contributions_collection)
    }
}
