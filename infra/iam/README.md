# IAM Setup

## GitHub Actions OIDC For Lambda Deploys

The Lambda deployment workflows use GitHub Actions OIDC instead of long-lived
AWS access keys. Public and admin Lambda deploys are intentionally separate.

AWS IAM prerequisites:

- OIDC identity provider
  - Provider URL: `https://token.actions.githubusercontent.com`
  - Audience: `sts.amazonaws.com`
- IAM role trusted by that provider.
- Inline or attached policy allowing Lambda code update.

GitHub repository settings:

- Secret: `AWS_GITHUB_ACTIONS_ROLE_ARN`
  - Value: IAM role ARN created for this workflow.
- Optional variables:
  - `PUBLIC_API_LAMBDA_FUNCTION_NAME`
    - Default used by workflow: `mwt-public-api`
  - `ADMIN_API_LAMBDA_FUNCTION_NAME`
    - Default used by workflow: `mwt-admin-api`

Attach this permissions policy to the role:

```text
infra/iam/github-actions-public-api-deploy-policy.json
infra/iam/github-actions-admin-api-deploy-policy.json
```

Trust policy template:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "Federated": "arn:aws:iam::123456789012:oidc-provider/token.actions.githubusercontent.com"
      },
      "Action": "sts:AssumeRoleWithWebIdentity",
      "Condition": {
        "StringEquals": {
          "token.actions.githubusercontent.com:aud": "sts.amazonaws.com"
        },
        "StringLike": {
          "token.actions.githubusercontent.com:sub": "repo:OWNER/REPO:ref:refs/heads/main"
        }
      }
    }
  ]
}
```

Replace `OWNER/REPO` with the GitHub repository slug before creating the role.

For manual first setup in the AWS console:

1. IAM > Identity providers > Add provider.
2. Choose OpenID Connect.
3. Provider URL: `https://token.actions.githubusercontent.com`.
4. Audience: `sts.amazonaws.com`.
5. IAM > Roles > Create role > Web identity.
6. Select the GitHub OIDC provider.
7. Set the trust policy to the template above, with `OWNER/REPO` replaced.
8. Attach `github-actions-public-api-deploy-policy.json` and
   `github-actions-admin-api-deploy-policy.json`.
9. Copy the role ARN into GitHub secret `AWS_GITHUB_ACTIONS_ROLE_ARN`.

The workflow file is:

```text
.github/workflows/deploy-public-api-lambda.yml
.github/workflows/deploy-admin-api-lambda.yml
```

The workflow is intentionally manual-only for the first repository push. After
the OIDC role and GitHub secret are configured and one manual deployment
succeeds, add a `push` trigger for `backend/**` changes if automatic deployment
from `main` is desired.
