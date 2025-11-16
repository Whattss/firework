# `fwk routes` Command

**Automatically discover and list all routes** in your Firework application.

## Usage

```bash
# List all routes
fwk routes

# Show detailed information
fwk routes --verbose

# Filter routes by pattern
fwk routes --filter "upload"
fwk routes --filter "/users/"

# Combine filters with verbose
fwk routes --filter "api" --verbose
```

## Output

### Basic Mode (default)

```
🔍 Scanning for routes...

  ────────────────────────────────────────────────────────────────
  GET     /api/users                               list_users
  GET     /api/users/:id                           get_user
  POST    /api/users                               create_user
  PATCH   /api/users/:id                           update_user
  DELETE  /api/users/:id                           delete_user
  ────────────────────────────────────────────────────────────────

✓ 5 routes registered

  Tip: Use --verbose for detailed information
       Use --filter <pattern> to filter routes
```

### Verbose Mode

```
🔍 Scanning for routes...

  GET /api/users
    Handler: list_users
    Location: src/routes/users.rs:10

  GET /api/users/:id
    Handler: get_user
    Location: src/routes/users.rs:25

  POST /api/users
    Handler: create_user
    Location: src/routes/users.rs:40

✓ 3 routes registered
```

### Filtered Mode

```bash
$ fwk routes --filter "upload"
```

```
🔍 Scanning for routes...

  ────────────────────────────────────────────────────────────────
  POST    /api/upload/avatar                       upload_avatar
  POST    /api/upload/banner                       upload_banner
  POST    /api/upload/image                        upload_image
  ────────────────────────────────────────────────────────────────

✓ 3 routes registered
  (filtered)
```

## How It Works

The `fwk routes` command:

1. **Scans your `src/` directory** for `.rs` files
2. **Parses route macros** (`#[get]`, `#[post]`, etc.)
3. **Extracts route information**:
   - HTTP method
   - Path pattern
   - Handler function name
   - File location and line number
4. **Groups and sorts** by method and path
5. **Pretty-prints** with colors

## Supported Macros

Detects all Firework route macros:

- `#[get("/path")]`
- `#[post("/path")]`
- `#[put("/path")]`
- `#[patch("/path")]`
- `#[delete("/path")]`
- `#[options("/path")]`
- `#[head("/path")]`

## Examples

### Find all authentication routes

```bash
fwk routes --filter "auth"
```

### Find all routes in a specific file

```bash
fwk routes --filter "users.rs" --verbose
```

### See all POST endpoints

```bash
fwk routes | grep POST
```

### Count total routes

```bash
fwk routes | grep "routes registered"
```

## Color Coding

- 🟢 **GET** - Green
- 🔵 **POST** - Blue
- 🟡 **PUT** - Yellow
- 🔵 **PATCH** - Cyan
- 🔴 **DELETE** - Red
- ⚪ **Others** - White

## Options

### `--verbose` / `-v`

Show detailed information including:
- Handler function name
- File path and line number
- More spacing for readability

### `--filter <pattern>` / `-f <pattern>`

Filter routes by pattern matching:
- Matches against path (`/api/users`)
- Matches against handler name (`get_user`)
- Case-sensitive

## Benefits

1. **Quick Overview** - See all routes at a glance
2. **Documentation** - Know what endpoints exist
3. **Debugging** - Find duplicate or missing routes
4. **Planning** - See API surface area
5. **Onboarding** - Help new devs understand the app

## Limitations

- Only detects routes defined with Firework macros
- Doesn't detect dynamically registered routes
- Requires valid Rust syntax (commented routes are ignored)

## Tips

### Generate API documentation

```bash
fwk routes > API_ROUTES.txt
```

### Check for route conflicts

```bash
fwk routes | sort
```

### Find all public API routes

```bash
fwk routes --filter "/api/"
```

### Export for scripts

```bash
fwk routes | grep -E "GET|POST" > routes.txt
```

## Future Enhancements

Possible future features:

- [ ] Export to JSON/YAML
- [ ] Show middleware applied to routes
- [ ] Detect route conflicts
- [ ] Show expected parameters
- [ ] Integration with OpenAPI/Swagger
- [ ] Route testing helpers

## Example Output (Real Project)

From the twitter-clone example:

```
🔍 Scanning for routes...

  ────────────────────────────────────────────────────────────────
  GET     /api/auth/me                             get_me
  GET     /api/tweets                              get_tweets
  GET     /api/tweets/:id                          get_single_tweet
  GET     /api/users/:username                     get_user_profile
  POST    /api/auth/login                          login
  POST    /api/auth/register                       register
  POST    /api/tweets                              create_tweet
  POST    /api/tweets/:id/like                     like_tweet
  POST    /api/upload/avatar                       upload_avatar
  DELETE  /api/tweets/:id                          delete_tweet
  DELETE  /api/tweets/:id/like                     unlike_tweet
  ────────────────────────────────────────────────────────────────

✓ 33 routes registered

  Tip: Use --verbose for detailed information
       Use --filter <pattern> to filter routes
```

---

**Part of Firework CLI** - Making development faster and more enjoyable! 🔥
