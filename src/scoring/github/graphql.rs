use super::{datetime::DateTime, GithubUrl, GraphQlError, ScoringData};

use graphql_client::GraphQLQuery;

#[allow(dead_code)]
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/scoring/github/schema.json",
    query_path = "src/scoring/github/query.graphql",
    response_derives = "Debug"
)]
pub struct GithubQuery;

pub(in crate::scoring) async fn query<T: Into<github_query::Variables>>(
    vars: T,
) -> Result<ScoringData, GraphQlError> {
    let client = super::get_client();

    let body = GithubQuery::build_query(vars.into());
    let response = client
        .post("https://api.github.com/graphql")
        .bearer_auth(super::get_token())
        .json(&body)
        .send()
        .await?;

    log::debug!("resp: {:?}", response);

    response
        .json::<graphql_client::Response<github_query::ResponseData>>()
        .await?
        .try_into()
        .map_err(|_| GraphQlError::MissingData)
}

impl From<GithubUrl> for <GithubQuery as GraphQLQuery>::Variables {
    fn from(GithubUrl { name, owner }: GithubUrl) -> Self {
        Self { name, owner }
    }
}

impl<T> TryFrom<graphql_client::Response<T>> for ScoringData
where
    ScoringData: TryFrom<T, Error = ()>,
{
    type Error = ();
    fn try_from(value: graphql_client::Response<T>) -> Result<Self, Self::Error> {
        value.data.ok_or(())?.try_into()
    }
}

impl TryFrom<github_query::ResponseData> for ScoringData {
    type Error = ();
    fn try_from(value: github_query::ResponseData) -> Result<Self, Self::Error> {
        let github_query::GithubQueryRepository {
            issues_open,
            issues_closed,
            issue_last_opened,
            assignable_users,
            object,
            license_info,
            has_wiki_enabled,
        } = value.repository.ok_or(())?;

        let readme_exists = object.is_some();
        let documentation_exists = has_wiki_enabled;
        let issues_closed = issues_closed.total_count.max(0) as usize;
        let issues_open = issues_open.total_count.max(0) as usize;
        let num_contributors = assignable_users.total_count.max(0) as usize;

        let weeks_since_last_issue = if let Some(Some(Some(last_issue))) =
            issue_last_opened.nodes.as_ref().map(|i| i.get(0))
        {
            (last_issue
                .created_at
                .signed_duration_since(DateTime::now())
                .num_days() as f64
                / 7.)
                .max(0.)
        } else {
            0.
        };

        let license_correct = license_info
            .and_then(|l| license_good(l.key).then_some(()))
            .is_some();

        Ok(ScoringData {
            readme_exists,
            documentation_exists,
            issues_closed,
            issues_total: issues_closed + issues_open,
            num_contributors,
            weeks_since_last_issue,
            license_correct,
        })
    }
}

fn license_good(license: String) -> bool {
    let license = license.to_lowercase();
    static GOOD_LICENSES: [&str; 46] = [
        "gpl-3.0-only",
        "gpl-3.0-or-later",
        "gpl-2.0-only",
        "gpl-2.0-or-later",
        "lgpl-2.1-only",
        "lgpl-2.1-or-later",
        "lgpl-3.0-only",
        "lgpl-3.0-or-later",
        "agpl-3.0",
        "apache-2.0",
        "artistic-2.0",
        "clartistic",
        "bsl-1.0",
        "cecill-2.0",
        "ecos-2.0",
        "ecl-2.0",
        "efl-2.0",
        "eudatagrid",
        "bsd-2-clause-freebsd",
        "ftl",
        "hpnd",
        "imatix",
        "imlib2",
        "ijg",
        "intel",
        "isc",
        "mpl-2.0",
        "ncsa",
        "python-2.0.1",
        "python-2.1.1",
        "ruby",
        "sgi-b-2.0",
        "standardml-nj",
        "smlnj",
        "unicode-dfs-2015",
        "unicode-dfs-2016",
        "upl-1.0",
        "unlicense",
        "vim",
        "wtfpl",
        "x11",
        "mit",
        "xfree86-1.1",
        "zlib",
        "zpl-2.0",
        "zpl-2.1",
    ];
    GOOD_LICENSES.iter().any(|l| *l == license)
}
