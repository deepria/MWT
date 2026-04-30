use lambda_http::http::StatusCode;
use lambda_http::RequestExt;
use lambda_http::{Body, Error, Request, Response};
use mwt_domain::auth::AuthContext;
use mwt_domain::problem::ProblemSummary;
use mwt_infra::repository::{
    ProblemRepository, RepositoryError, StatementRepository, SubmissionRepository,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ErrorBody {
    message: String,
}

#[derive(Debug, Serialize)]
struct StatementBody {
    problem_id: String,
    format: &'static str,
    content: String,
}

pub async fn handle_request<R>(request: Request, repository: R) -> Result<Response<Body>, Error>
where
    R: ProblemRepository + StatementRepository + SubmissionRepository,
{
    let method = request.method().as_str();
    let path = request.uri().path();
    let segments = path
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    match (method, segments.as_slice()) {
        ("GET", ["problems"]) => list_problems(&repository).await,
        ("GET", ["problems", problem_id]) => get_problem(&repository, problem_id).await,
        ("GET", ["problems", problem_id, "statement"]) => {
            get_statement(&repository, problem_id).await
        }
        ("GET", ["submissions", submission_id]) => get_submission(&repository, submission_id).await,
        ("GET", ["users", "me", "submissions"]) => {
            let auth = auth_context(&request)?;
            list_my_submissions(&repository, &auth.user_id).await
        }
        _ => json_response(
            StatusCode::NOT_FOUND,
            &ErrorBody {
                message: "route not found".to_string(),
            },
        ),
    }
}

async fn list_problems<R>(repository: &R) -> Result<Response<Body>, Error>
where
    R: ProblemRepository,
{
    let problems = repository
        .list_public_problems()
        .await?
        .into_iter()
        .map(ProblemSummary::from)
        .collect::<Vec<_>>();

    json_response(StatusCode::OK, &problems)
}

async fn get_problem<R>(repository: &R, problem_id: &str) -> Result<Response<Body>, Error>
where
    R: ProblemRepository,
{
    match repository.get_problem(problem_id).await {
        Ok(problem) => json_response(StatusCode::OK, &problem),
        Err(RepositoryError::NotFound(_)) => not_found("problem not found"),
        Err(error) => Err(error.into()),
    }
}

async fn get_statement<R>(repository: &R, problem_id: &str) -> Result<Response<Body>, Error>
where
    R: ProblemRepository + StatementRepository,
{
    let problem = match repository.get_problem(problem_id).await {
        Ok(problem) => problem,
        Err(RepositoryError::NotFound(_)) => return not_found("problem not found"),
        Err(error) => return Err(error.into()),
    };
    let content = repository
        .get_statement_markdown(&problem.statement_location)
        .await?;

    json_response(
        StatusCode::OK,
        &StatementBody {
            problem_id: problem.problem_id,
            format: "markdown",
            content,
        },
    )
}

async fn get_submission<R>(repository: &R, submission_id: &str) -> Result<Response<Body>, Error>
where
    R: SubmissionRepository,
{
    match repository.get_submission(submission_id).await {
        Ok(submission) => json_response(StatusCode::OK, &submission),
        Err(RepositoryError::NotFound(_)) => not_found("submission not found"),
        Err(error) => Err(error.into()),
    }
}

async fn list_my_submissions<R>(repository: &R, user_id: &str) -> Result<Response<Body>, Error>
where
    R: SubmissionRepository,
{
    let submissions = repository.list_user_submissions(user_id).await?;
    json_response(StatusCode::OK, &submissions)
}

fn auth_context(request: &Request) -> Result<AuthContext, Error> {
    let claims = request
        .request_context_ref()
        .and_then(|context| context.authorizer())
        .and_then(|authorizer| authorizer.jwt.as_ref())
        .map(|jwt| &jwt.claims);

    let user_id = claims
        .and_then(|claims| claims.get("sub"))
        .cloned()
        .or_else(|| header(request, "x-mwt-user-id"))
        .unwrap_or_else(|| "mock-user-001".to_string());
    let email = claims
        .and_then(|claims| claims.get("email"))
        .cloned()
        .or_else(|| header(request, "x-mwt-email"));
    let groups = claims
        .and_then(|claims| claims.get("cognito:groups"))
        .cloned()
        .or_else(|| header(request, "x-mwt-groups"))
        .map(|value| {
            value
                .split(',')
                .filter(|group| !group.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    Ok(AuthContext {
        user_id,
        email,
        groups,
    })
}

fn header(request: &Request, name: &str) -> Option<String> {
    request
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

fn not_found(message: &str) -> Result<Response<Body>, Error> {
    json_response(
        StatusCode::NOT_FOUND,
        &ErrorBody {
            message: message.to_string(),
        },
    )
}

fn json_response<T>(status: StatusCode, body: &T) -> Result<Response<Body>, Error>
where
    T: Serialize,
{
    let body = serde_json::to_string(body)?;

    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::Text(body))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_http::http::{HeaderName, HeaderValue, Method};
    use mwt_infra::memory::MemoryRepository;

    #[tokio::test]
    async fn lists_problems() {
        let request = request(Method::GET, "/problems", &[]);

        let response = handle_request(request, MemoryRepository::default())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_text(response).contains("sum-path"));
    }

    #[tokio::test]
    async fn returns_statement() {
        let request = request(Method::GET, "/problems/sum-path/statement", &[]);

        let response = handle_request(request, MemoryRepository::default())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_text(response).contains("markdown"));
    }

    #[tokio::test]
    async fn lists_my_submissions() {
        let request = request(
            Method::GET,
            "/users/me/submissions",
            &[("x-mwt-user-id", "mock-user-001")],
        );

        let response = handle_request(request, MemoryRepository::default())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_text(response).contains("sub-20260429-001"));
    }

    fn body_text(response: Response<Body>) -> String {
        match response.into_body() {
            Body::Text(text) => text,
            other => panic!("unexpected body: {other:?}"),
        }
    }

    fn request(method: Method, uri: &str, headers: &[(&str, &str)]) -> Request {
        let mut request = Request::new(Body::Empty);
        *request.method_mut() = method;
        *request.uri_mut() = uri.parse().unwrap();

        for (name, value) in headers {
            request.headers_mut().insert(
                HeaderName::from_bytes(name.as_bytes()).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
        }

        request
    }
}
