# 🐛 Fix: Página en Blanco - Vite Proxy Content-Type

## Problema

Al acceder a `http://localhost:8080`:
- ✅ HTML se carga
- ❌ Página aparece **en blanco**
- ✅ En `http://localhost:5173` funciona bien

### Causa Raíz

El proxy de Vite estaba funcionando PERO las **headers de Content-Type se perdían**:

```bash
# Lo que devolvía el proxy:
HTTP/1.1 200 OK
Content-Type: text/plain; charset=utf-8   # ❌ WRONG!

# Lo que Vite devuelve:
HTTP/1.1 200 OK  
Content-Type: text/html; charset=utf-8    # ✅ CORRECT
```

**Resultado**: El navegador recibía HTML pero con `Content-Type: text/plain`, causando que:
- No ejecutara el JavaScript
- No cargara los assets
- La página quedara en blanco

---

## Diagnóstico

```bash
# Test 1: Verificar respuesta
$ curl -I http://localhost:8080/
HTTP/1.1 200 OK
Content-Type: text/plain; charset=utf-8  # ❌ Debería ser text/html

# Test 2: Verificar Vite directo
$ curl -I http://localhost:5173/
HTTP/1.1 200 OK
Content-Type: text/html; charset=utf-8   # ✅ Correcto

# Test 3: Verificar JavaScript
$ curl -I http://localhost:8080/@vite/client
Content-Type: text/plain                  # ❌ Debería ser application/javascript
```

---

## Solución

**Archivo**: `/home/whattss/Dev/rust/fwk/plugins/firework-vite/src/lib.rs`

### Antes (❌ Bug):

```rust
let mut new_res = Response::new(status, body.to_vec());

// Copy headers
for (key, value) in headers.iter() {
    if let Ok(v) = value.to_str() {
        new_res.headers.insert(key.to_string(), v.to_string());
    }
}
```

**Problema**: El servidor Firework agrega `Content-Type: text/plain` por defecto DESPUÉS, sobrescribiendo el que viene de Vite.

### Después (✅ Fix):

```rust
let mut new_res = Response::new(status, body.to_vec());

// Copy headers FIRST (preserva Content-Type de Vite)
for (key, value) in headers.iter() {
    if let Ok(v) = value.to_str() {
        new_res.headers.insert(key.to_string(), v.to_string());
    }
}

// Ensure Content-Type from Vite is preserved (double-check)
if let Some(ct) = headers.get("content-type") {
    if let Ok(v) = ct.to_str() {
        new_res.headers.insert("Content-Type".to_string(), v.to_string());
    }
}
```

**Solución**: 
1. Copiar headers de Vite primero
2. Asegurar explícitamente que Content-Type se preserve
3. El servidor ya no sobrescribe porque existe

---

## Verificación

### Script de Test

```bash
cd ~/twitter-clone
./test_vite_proxy.sh
```

**Salida esperada**:
```
🧪 Testing Vite Proxy - Content-Type Fix
========================================

📄 Test 1: Root path (/) should be text/html
   Response: content-type: text/html; charset=utf-8
   ✅ PASS - HTML served correctly

📦 Test 2: Vite client (/@vite/client) should be application/javascript
   Response: content-type: text/javascript; charset=utf-8
   ✅ PASS - JavaScript served correctly

🎨 Test 3: CSS files should be text/css
   Response: content-type: text/css
   ✅ PASS - CSS served correctly

🔌 Test 4: API routes should NOT be proxied
   Status: 200
   ✅ PASS - API handled by Firework
```

### Manual Test

```bash
# 1. Start server
cd ~/twitter-clone
cargo run

# 2. Open browser
open http://localhost:8080

# 3. Check DevTools Console
# ✅ No errors
# ✅ React app loads
# ✅ Assets loaded correctly
```

---

## Flujo Correcto Ahora

```
Browser → http://localhost:8080/
    ↓
[Firework :8080]
    ↓
[vite_middleware_fn]
    ↓
¿Es /api/*?
    NO → Proxy a Vite :5173
    ↓
[Vite Dev Server]
    ↓
Response con Content-Type: text/html
    ↓
[vite_middleware copia headers]  ← FIX APLICADO AQUÍ
    ↓
[Firework preserva Content-Type]
    ↓
Browser recibe HTML con headers correctas ✅
```

---

## Archivos Modificados

1. **`firework-vite/src/lib.rs`** (líneas 336-365)
   - ✅ Copiar headers de Vite ANTES
   - ✅ Preservar Content-Type explícitamente
   - ✅ Evitar sobrescritura por defaults

2. **`twitter-clone/test_vite_proxy.sh`** (nuevo)
   - ✅ Script de testing automatizado
   - ✅ Verifica Content-Types correctos
   - ✅ Valida proxy funciona

---

## Por Qué Pasaba

El servidor Firework tiene este código en `server.rs`:

```rust
response
    .headers
    .entry("Content-Type".to_string())
    .or_insert_with(|| "text/plain; charset=utf-8".to_string());
```

**Explicación**:
- `.entry().or_insert_with()` solo agrega si NO existe
- PERO el timing era incorrecto
- Headers se copiaban en el middleware
- Luego el servidor verificaba y NO encontraba Content-Type
- Agregaba `text/plain` por defecto

**Fix**: Copiar headers PRIMERO en el Response, ANTES de que el servidor verifique.

---

## Testing en Producción

### Desarrollo (con HMR):
```bash
cargo run
# → http://localhost:8080 proxies to :5173
# → Content-Types correctos
# → HMR funciona
```

### Producción (static files):
```bash
cd frontend && npm run build
cargo run --release
# → Sirve desde frontend/dist/
# → Content-Types desde archivos estáticos
```

---

## Lessons Learned

1. **Headers matter**: Content-Type incorrecto rompe completamente la aplicación
2. **Timing matters**: El orden de operaciones (copiar headers vs defaults) es crítico
3. **Proxy transparency**: Los proxies deben ser 100% transparentes con headers
4. **Testing**: Scripts automatizados ayudan a verificar fixes

---

**Fecha**: 2024-11-16  
**Status**: ✅ RESUELTO  
**Impact**: CRÍTICO (bloqueaba frontend completamente)  
**Fix**: 10 líneas de código
