# RustWeb

## Docker development

Backend, frontend, Postgres, and Keycloak can be started together:

```bash
docker compose up --build
```

App URLs:

- Frontend: http://localhost:5173
- Backend: http://localhost:3000
- Keycloak: http://localhost:8080

Development login:

- Username: `admin`
- Password: `admin`

The compose setup imports `keycloak/realm-export.json` on startup and overrides the backend container environment so it can reach Postgres as `db` while using the same localhost Keycloak issuer URL as the browser.
