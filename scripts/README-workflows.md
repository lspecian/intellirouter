# IntelliRouter CI/CD Workflows

This document explains the CI/CD workflow structure of the IntelliRouter project and how to test workflows locally.

## Workflow Structure

The project has 7 different workflow files, each handling a specific aspect of testing and deployment:

1. **cd.yml** - Continuous Deployment (handles deployment to production)
2. **ci.yml** - Continuous Integration (general build verification)
3. **codeql-analysis.yml** - Code Quality Analysis (security scanning)
4. **dependency-review.yml** - Dependency Review (checks for vulnerable dependencies)
5. **e2e-tests.yml** - End-to-End Tests (integration testing)
6. **security.yml** - Security Checks (additional security scanning)
7. **test.yml** - Unit Tests, Coverage, and Linting

This separation of concerns is a common practice in large projects because it:
- Makes the CI/CD pipeline more maintainable
- Allows for parallel execution of different types of tests
- Enables selective execution (e.g., run only security checks when security-related code changes)
- Provides clearer failure points when issues arise

## Testing Workflows Locally

You can test GitHub Actions workflows locally using [act](https://github.com/nektos/act), a tool that allows you to run GitHub Actions locally without pushing to GitHub.

### Prerequisites

1. Install Docker (required by act)
2. Install act:
   ```bash
   # macOS
   brew install act
   
   # Linux
   curl -s https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
   ```

### Using the Workflow Testing Script

We've provided a script to simplify testing workflows locally:

```bash
# Make the script executable (if not already)
chmod +x scripts/test-workflows-locally.sh

# Run the script
./scripts/test-workflows-locally.sh
```

The script provides a menu to:
- Run all workflows
- Run specific workflows (test, e2e-tests, codeql-analysis)
- Run specific jobs from a workflow

### Manual Usage of act

If you prefer to use act directly:

1. Run all workflows triggered by push:
   ```bash
   act push
   ```

2. Run a specific workflow:
   ```bash
   act -W .github/workflows/test.yml push
   ```

3. Run a specific job from a workflow:
   ```bash
   act -W .github/workflows/test.yml -j test
   ```

### Configuration

The `.actrc` file in the project root contains recommended settings for act:
```
-P ubuntu-latest=nektos/act-environments-ubuntu:18.04
--env FONTCONFIG_NO_PKG_CONFIG=1
```

This configuration:
- Uses the Ubuntu 18.04 Docker image for running workflows
- Sets the `FONTCONFIG_NO_PKG_CONFIG=1` environment variable to bypass pkg-config requirements for fontconfig

## Common Issues and Solutions

1. **System Dependencies**: Some workflows require system dependencies like `libfontconfig1-dev`. These are installed in the workflow files, but you may need to install them locally if testing with act.

2. **Docker Image Size**: The default Docker images used by act can be large. Use the `-P` flag to specify a smaller image if needed.

3. **Secrets**: If your workflows use secrets, you can provide them using the `--secret` flag or by creating a `.secrets` file.

4. **GitHub API Rate Limiting**: Some workflows may hit GitHub API rate limits. You can provide a GitHub token using the `--token` flag.

## Conclusion

Testing workflows locally with act can save time and reduce the number of commits needed to fix issues. It's a valuable tool for CI/CD pipeline development and maintenance.