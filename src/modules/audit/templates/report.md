# IntelliRouter Audit Report

## Summary

- **Timestamp**: {{report.timestamp}}
- **Status**: {{#if report.success}}✅ Success{{else}}❌ Failure{{/if}}
- **Services**: {{report.service_statuses.length}}
- **Tests**: {{report.test_results.length}}
- **Communication Tests**: {{report.communication_tests.length}}
- **Metrics**: {{report.metrics.length}}

## Service Statuses

| Service | Status |
|---------|--------|
{{#each report.service_statuses}}
| {{@key}} | {{#if (eq this "Running")}}✅ Running{{else if (eq this "Failed")}}❌ Failed{{else if (eq this "NotStarted")}}⏳ Not Started{{else if (eq this "Starting")}}🔄 Starting{{else if (eq this "ShuttingDown")}}🔄 Shutting Down{{else if (eq this "Stopped")}}⏹️ Stopped{{/if}} |
{{/each}}

{{#if report.successes.length}}
## Successes

{{#each report.successes}}
- ✅ {{this}}
{{/each}}
{{/if}}

{{#if report.warnings.length}}
## Warnings

{{#each report.warnings}}
- ⚠️ {{this}}
{{/each}}
{{/if}}

{{#if report.errors.length}}
## Errors

{{#each report.errors}}
- ❌ {{this}}
{{/each}}
{{/if}}

{{#if report.test_results.length}}
## Test Results

| Test | Status | Duration (ms) | Timestamp |
|------|--------|--------------|------------|
{{#each report.test_results}}
| {{this.test_flow}} | {{#if this.success}}✅ Success{{else}}❌ Failure{{/if}} | {{this.duration_ms}} | {{this.timestamp}} |
{{/each}}
{{/if}}

{{#if report.communication_tests.length}}
## Communication Tests

| Source | Target | Status | Response Time (ms) |
|--------|--------|--------|-------------------|
{{#each report.communication_tests}}
| {{this.source}} | {{this.target}} | {{#if this.success}}✅ Success{{else}}❌ Failure{{/if}} | {{this.response_time_ms}} |
{{/each}}
{{/if}}

{{#if report.metric_analyses.length}}
## Metric Analyses

| Service | Metric | Value | Description |
|---------|--------|-------|-------------|
{{#each report.metric_analyses}}
| {{this.service}} | {{this.metric_type}} | {{this.average_value}} | {{this.description}} |
{{/each}}
{{/if}}

---

Report generated: {{timestamp}}