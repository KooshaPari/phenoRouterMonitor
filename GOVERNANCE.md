# Governance notes for AgilePlus

- dashboard service ops API contract:
  - endpoints: /api/dashboard/services/{name}/toggle and /api/dashboard/services/{name}/restart
  - toggle persistence writes to Config services array and updates in-memory health state.
  - restart uses environment-driven `AGILEPLUS_SERVICE_RESTART_CMD` and uses a safe command registry (`systemctl`, `docker`, `process-compose`, `echo`).
  - test coverage states: unit tests in agileplus-dashboard routes + integration test in agileplus-integration-tests/service_control.rs.
