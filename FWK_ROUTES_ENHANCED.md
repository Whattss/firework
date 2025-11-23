# 🎯 fwk routes - Enhanced Features

**Fecha**: 2024-11-17  
**Status**: ✅ COMPLETADO

---

## 🎉 Nuevas Funcionalidades

El comando `fwk routes` ahora tiene **SUPERPODERES**:

| Feature | Flag | Descripción |
|---------|------|-------------|
| **Filtrar** | `--filter <pattern>` | Filtra rutas por patrón |
| **Verbose** | `--verbose` | Info detallada + ubicación |
| **Stats** | `--stats` | Estadísticas completas |
| **Check** | `--check` | Detecta conflictos |
| **Export OpenAPI** | `--export openapi` | Genera openapi.json |
| **Export Markdown** | `--export markdown` | Genera ROUTES.md |

---

## 📊 Feature #1: Statistics

```bash
$ fwk routes --stats
```

**Output**:
```
🔍 Scanning for routes...

📊 Route Statistics

  📍 Total Routes
    39

  🔤 By HTTP Method
    GET     17
    POST    14
    DELETE  7
    PATCH   1

  📁 Top Files
    tweets.rs            9
    uploads.rs           8
    users.rs             8
    comments.rs          6
    auth.rs              3

  🔖 Route Parameters
    24 total params in 24 routes
```

**Útil para**:
- Ver distribución de endpoints
- Identificar archivos más grandes
- Contar parámetros dinámicos

---

## 🔍 Feature #2: Check Conflicts

```bash
$ fwk routes --check
```

**Output**:
```
🔍 Checking for route conflicts...

⚠ Duplicate route found: GET /api/tweets/:id
    → src/routes/tweets.rs:176
    → src/extractors.rs:31

⚠ Potential parameter conflict: GET /api/tweets/:param
    → /api/tweets/:id (src/routes/tweets.rs:176)
    → /api/tweets/:id (src/extractors.rs:31)
```

**Detecta**:
- ✅ Rutas duplicadas (exactas)
- ✅ Conflictos de parámetros (`:id` vs `:user_id`)
- ✅ Ubicación exacta del conflicto

**Útil antes de**:
- Deploy
- Merge PR
- Debugging routing issues

---

## 📤 Feature #3: Export OpenAPI

```bash
$ fwk routes --export openapi
```

**Output**:
```
📤 Exporting routes to OpenAPI 3.0 format...

✓ OpenAPI spec exported to openapi.json

  You can now:
    • Import into Postman
    • Use with Swagger UI
    • Generate client SDKs
```

**Genera**: `openapi.json`
```json
{
  "openapi": "3.0.0",
  "info": {
    "title": "Firework API",
    "version": "1.0.0",
    "description": "Auto-generated API documentation"
  },
  "paths": {
    "/api/tweets": {
      "get": {
        "summary": "get_tweets",
        "operationId": "get_tweets",
        "responses": { "200": { "description": "Successful response" } },
        "tags": ["Tweets"]
      }
    }
  }
}
```

**Uso**:
1. **Postman**: File → Import → openapi.json
2. **Swagger UI**: https://editor.swagger.io → File → Import file
3. **Client SDKs**: `openapi-generator generate -i openapi.json -g typescript-axios`

---

## 📝 Feature #4: Export Markdown

```bash
$ fwk routes --export markdown
```

**Output**:
```
📤 Exporting routes to Markdown...

✓ Route documentation exported to ROUTES.md
```

**Genera**: `ROUTES.md`
```markdown
# API Routes

Auto-generated route documentation

---

## Tweets

### GET `/api/tweets`

**Handler:** `get_tweets`

---

### POST `/api/tweets`

**Handler:** `create_tweet`

---

## Users

### GET `/api/users/:username`

**Handler:** `get_user_profile`

---
```

**Útil para**:
- Documentación en README.md
- Onboarding de nuevos devs
- GitHub wiki

---

## 🎨 Feature #5: Verbose Mode

```bash
$ fwk routes --verbose
```

**Output**:
```
GET /api/tweets
    Handler: get_tweets
    Location: src/routes/tweets.rs:24

POST /api/tweets
    Handler: create_tweet
    Location: src/routes/tweets.rs:45
    Middleware: auth_required
```

**Muestra**:
- Handler function name
- File + line number
- Middleware (si se detecta)

---

## 🔎 Feature #6: Filter (mejorado)

```bash
$ fwk routes --filter tweets
```

**Output**:
```
  GET     /api/tweets                              get_tweets
  GET     /api/tweets/:id                          get_tweet
  POST    /api/tweets                              create_tweet
  POST    /api/tweets/:id/like                     like_tweet
  DELETE  /api/tweets/:id                          delete_tweet

✓ 12 routes registered
  (filtered)
```

**Filtra por**:
- Path: `/api/tweets`
- Handler: `get_tweets`
- Cualquier match

---

## 🚀 Casos de Uso

### 1. Debugging Routes

```bash
# Ver todas las rutas
fwk routes

# Buscar ruta específica
fwk routes --filter /api/users/:username --verbose

# Verificar conflictos antes de deploy
fwk routes --check
```

### 2. Documentation

```bash
# Generar docs para README
fwk routes --export markdown

# Agregar a README.md
cat ROUTES.md >> README.md
```

### 3. API Client Generation

```bash
# Generar OpenAPI
fwk routes --export openapi

# Generar TypeScript client
npx @openapitools/openapi-generator-cli generate \
  -i openapi.json \
  -g typescript-axios \
  -o ./client

# Generar Python client
openapi-generator generate \
  -i openapi.json \
  -g python \
  -o ./python-client
```

### 4. Postman Collection

```bash
# Exportar a OpenAPI
fwk routes --export openapi

# Importar en Postman:
# 1. Abrir Postman
# 2. File → Import
# 3. Select openapi.json
# 4. ✅ 39 requests importadas!
```

### 5. CI/CD Validation

```bash
#!/bin/bash
# .github/workflows/validate-routes.yml

# Check for route conflicts before merge
fwk routes --check || exit 1

# Verify minimum routes exist
ROUTE_COUNT=$(fwk routes | grep "routes registered" | grep -oE '[0-9]+')
if [ "$ROUTE_COUNT" -lt 10 ]; then
  echo "Error: Too few routes ($ROUTE_COUNT)"
  exit 1
fi
```

### 6. Performance Audit

```bash
# Ver stats para identificar hotspots
fwk routes --stats

# Si tweets.rs tiene 20+ routes:
# → Consider splitting into multiple files
# → tweets_crud.rs, tweets_likes.rs, etc.
```

---

## 📊 Comparación con Otros Frameworks

| Framework | Command | Stats | Check | Export |
|-----------|---------|-------|-------|--------|
| **Firework** | `fwk routes` | ✅ | ✅ | ✅ OpenAPI + MD |
| Rails | `rails routes` | ❌ | ❌ | ❌ |
| Express | *(plugin)* | ❌ | ❌ | ❌ |
| Django | `show_urls` | ❌ | ❌ | ❌ |
| FastAPI | Built-in docs | ✅ | ❌ | ✅ OpenAPI |
| Actix | ❌ | ❌ | ❌ | ❌ |

**Firework tiene el mejor tooling de rutas en Rust.** 🏆

---

## 🎯 Tips & Tricks

### Combine Filters

```bash
# Solo rutas GET de tweets
fwk routes --filter tweets | grep "GET"

# Count routes per method
fwk routes | grep POST | wc -l
```

### Export Both Formats

```bash
# Generar documentación completa
fwk routes --export openapi
fwk routes --export markdown

# Commit ambos
git add openapi.json ROUTES.md
git commit -m "docs: Update API documentation"
```

### Pre-commit Hook

```bash
# .git/hooks/pre-commit
#!/bin/bash

fwk routes --check
if [ $? -ne 0 ]; then
  echo "❌ Route conflicts detected!"
  echo "Run 'fwk routes --check' to see details"
  exit 1
fi
```

### VS Code Task

```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "List Routes",
      "type": "shell",
      "command": "fwk routes",
      "problemMatcher": []
    },
    {
      "label": "Check Routes",
      "type": "shell",
      "command": "fwk routes --check",
      "problemMatcher": []
    }
  ]
}
```

---

## 🔮 Future Features (Ideas)

### Response Type Detection

```rust
#[get("/api/tweets")]
pub async fn get_tweets() -> Json<Vec<Tweet>> {
    // Auto-detect: returns Vec<Tweet>
}
```

```bash
$ fwk routes --verbose
GET /api/tweets
    Handler: get_tweets
    Returns: Json<Vec<Tweet>>  # 🔥 AUTO-DETECTED
```

### Middleware Detection

```rust
#[middleware(auth_required)]
#[get("/api/me")]
pub async fn get_me() -> Json<User> {
    // ...
}
```

```bash
$ fwk routes
GET /api/me  [auth_required]  # 🔥 SHOWS MIDDLEWARE
```

### Performance Hints

```bash
$ fwk routes --stats
⚠️  tweets.rs has 20 routes - consider splitting
💡 10 routes don't use caching - add cache middleware
🔥 5 routes have N+1 query potential
```

---

## 📚 Referencias

- **OpenAPI 3.0 Spec**: https://swagger.io/specification/
- **Swagger Editor**: https://editor.swagger.io/
- **OpenAPI Generator**: https://openapi-generator.tech/
- **Postman**: https://www.postman.com/

---

**Fecha**: 2024-11-17  
**Status**: ✅ PRODUCTION READY  
**Features**: 6 nuevas funcionalidades

🎉 **fwk routes is now a BEAST!**
