use crate::{
    contributions_query::ContributionsQueryUserContributionsCollection,
    repositories_query::RepositoriesQueryUserRepositories,
};
use anyhow::Result;
use dotenv::dotenv;
use graphql_client::*;
use log::info;
use pretty_env_logger;
use serde::Deserialize;
use warp::Filter;

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

#[derive(Debug, Deserialize)]
struct Env {
    login: String,
    token: String,
}

struct GitHub {
    pub contributions: Option<ContributionsQueryUserContributionsCollection>,
    pub repositories: Option<RepositoriesQueryUserRepositories>,
}

impl GitHub {
    fn new() -> Self {
        Self {
            contributions: None,
            repositories: None,
        }
    }

    async fn update(&mut self, login: &str, token: &str) -> Result<()> {
        self.query_contributions(login, token).await?;
        self.query_repositories(login, token).await?;

        Ok(())
    }

    pub async fn query_repositories(&mut self, login: &str, token: &str) -> Result<()> {
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

    pub async fn query_contributions(&mut self, login: &str, token: &str) -> Result<()> {
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

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    pretty_env_logger::init();

    let env = envy::from_env::<Env>()?;
    info!("{:?}", env);

    let mut gh = GitHub::new();
    gh.update(&env.login, &env.token).await?;

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    warp::serve(hello).run(([0, 0, 0, 0], 8080)).await;

    Ok(())
}
