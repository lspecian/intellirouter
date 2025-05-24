# IntelliRouter Compilation Warning Report

**Total Warnings:** 236

## Warning Types

| Type | Count |
|------|-------|
| unused_variable | 37 |
| unused_import | 190 |
| other | 9 |

## Top 10 Files with Most Warnings

| File | Count |
|------|-------|
| src/modules/chain_engine/executors/mod.rs | 8 |
| src/modules/audit/validation/mod.rs | 8 |
| src/modules/model_registry/connectors/ollama.rs | 7 |
| src/modules/llm_proxy/service/mod.rs | 7 |
| src/modules/router_core/router.rs | 6 |
| src/modules/orchestrator/reporting.rs | 6 |
| src/modules/model_registry/connectors/openai.rs | 6 |
| src/modules/monitoring/dashboard.rs | 5 |
| src/modules/llm_proxy/router_integration.rs | 5 |
| src/modules/ipc/redis_pubsub.rs | 5 |

## Suggestions for Common Warning Types

### Unused Variables (37 warnings)

- Prefix unused variables with underscore (_variable_name)
- Remove unused variables if they're not needed
- Example fix: `let unused = 5;` → `let _unused = 5;`

### Unused Imports (190 warnings)

- Remove unused imports
- Use cargo-fix with `cargo fix --allow-dirty` to automatically remove unused imports
- Consider using an IDE with auto-import cleanup

