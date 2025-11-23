# 🔄 Fix: Reload Loop Infinito - Vite HMR WebSocket

## Problema

Al acceder a `http://localhost:8080`:
- ✅ La página carga
- ❌ Se recarga constantemente en bucle infinito
- ✅ En `localhost:5173` NO pasa

### Causa Raíz

**HMR (Hot Module Replacement) de Vite intenta conectarse al WebSocket equivocado**:

```
Browser en :8080 
    ↓
Vite Client carga desde :8080 (proxied)
    ↓
HMR intenta conectar WebSocket a ws://localhost:8080/__vite_hmr  ❌
    ↓
Firework NO tiene WebSocket server en esa ruta
    ↓
Conexión falla
    ↓
Vite client detecta "conexión perdida"
    ↓
Recarga la página completa
    ↓
LOOP INFINITO 🔄
```

**Lo que debería pasar**:
```
Browser en :8080
    ↓
HMR conecta a ws://localhost:5173/__vite_hmr  ✅
    ↓
WebSocket funciona
    ↓
HMR actualiza sin recargar
```

---

## Diagnóstico

### Console DevTools:
```
[vite] connecting...
[vite] server connection lost. Polling for restart...
[vite] server connection lost. Polling for restart...
[vite] server connection lost. Polling for restart...
# Reload infinito...
```

### Network Tab:
```
WS ws://localhost:8080/__vite_hmr
Status: 404 Not Found
# Intenta conectar a :8080 en vez de :5173
```

---

## Solución

**Archivo**: `/home/whattss/twitter-clone/frontend/vite.config.js`

### Antes (❌ Bug):

```javascript
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:8080'
    }
  }
})
```

**Problema**: Vite asume que el HMR WebSocket está en el mismo host/port que sirvió el HTML (localhost:8080).

### Después (✅ Fix):

```javascript
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    host: '0.0.0.0',
    hmr: {
      // Asegurar que HMR siempre apunte a :5173, no :8080
      clientPort: 5173,
      host: 'localhost',
    },
    proxy: {
      '/api': 'http://127.0.0.1:8080'
    }
  }
})
```

**Fix**: 
- `hmr.clientPort: 5173` → Forzar WebSocket a :5173
- `hmr.host: 'localhost'` → Host correcto
- Ahora el cliente HMR siempre conecta a `ws://localhost:5173/__vite_hmr`

---

## Alternativa: Proxy WebSocket en Firework

Si quisieras que el WebSocket también funcione via :8080, necesitarías:

```rust
// En server.rs - detectar upgrade a WebSocket
if req.headers.get("Upgrade") == Some("websocket") 
   && req.uri.path.contains("__vite_hmr") 
{
    // Forward WebSocket connection to :5173
}
```

**Pero es más complejo**, mejor dejar que HMR conecte directo a :5173.

---

## Verificación

### 1. Reiniciar Vite

```bash
# Matar proceso Vite actual (usa config vieja)
pkill -f vite

# Reiniciar servidor (Firework auto-inicia Vite con nueva config)
cd ~/twitter-clone
cargo run
```

### 2. Test en Browser

```bash
open http://localhost:8080
```

**DevTools Console**:
```
[vite] connected.  ✅
# NO más "connection lost"
# NO más reload loop
```

**Network Tab - WebSocket**:
```
WS ws://localhost:5173/__vite_hmr
Status: 101 Switching Protocols  ✅
```

### 3. Test HMR

```bash
# Editar archivo
echo "// test" >> frontend/src/App.jsx

# Browser debería actualizar SIN reload completo
# ✅ Hot Module Replacement funciona
```

---

## Flujo Correcto Ahora

```
Browser → http://localhost:8080/
    ↓
[HTML desde Vite via proxy :8080→:5173]
    ↓
<script src="/@vite/client"> carga
    ↓
Vite client lee config HMR:
    clientPort: 5173  ← configurado explícitamente
    ↓
WebSocket conecta a ws://localhost:5173/__vite_hmr  ✅
    ↓
HMR funciona sin reload loop
```

### HTTP vs WebSocket Routing

| Recurso | Puerto | Protocolo | Handler |
|---------|--------|-----------|---------|
| `/` HTML | :8080 → :5173 | HTTP | Firework proxy |
| `/api/*` | :8080 | HTTP | Firework routes |
| `/@vite/client` | :8080 → :5173 | HTTP | Firework proxy |
| `/__vite_hmr` | :5173 | WebSocket | Vite directo ✅ |

---

## Por Qué No Proxear WebSocket

**Opción 1: Proxy HTTP + WS separado** (actual)
- ✅ Simple
- ✅ No requiere código extra
- ✅ WebSocket directo es más eficiente
- ✅ Funciona out-of-the-box

**Opción 2: Proxy HTTP + WS juntos**
- ❌ Requiere WebSocket proxy en Firework
- ❌ Más código, más complejidad
- ❌ Latencia extra (doble salto)
- ❌ Manejo de errores más complejo

**Conclusión**: Para desarrollo, **mejor dejar HMR directo a :5173**.

---

## Configuración Adicional (Opcional)

### Para acceso remoto (Docker, VM, etc.):

```javascript
hmr: {
  clientPort: 5173,
  host: '0.0.0.0',  // Permite conexiones remotas
}
```

### Para HTTPS:

```javascript
hmr: {
  protocol: 'wss',  // WebSocket Secure
  clientPort: 5173,
}
```

---

## Testing en Producción

En producción NO hay HMR, así que este problema no existe:

```bash
cd frontend && npm run build
cargo run --release
# → Sirve static files desde dist/
# → No hay WebSocket HMR
# → No hay reload loop
```

---

## Archivos Modificados

1. **`twitter-clone/frontend/vite.config.js`**
   - ✅ Agregado `hmr.clientPort: 5173`
   - ✅ Agregado `hmr.host: 'localhost'`
   - ✅ Configurado port y host explícitos

---

## Lessons Learned

1. **WebSocket ≠ HTTP**: Los proxies HTTP no manejan WebSockets automáticamente
2. **HMR routing**: Vite necesita saber explícitamente dónde está el WebSocket
3. **Development vs Production**: HMR solo en dev, no afecta producción
4. **Configuración explícita**: Mejor ser explícito con ports/hosts que asumir defaults

---

**Fecha**: 2024-11-16  
**Status**: ✅ RESUELTO  
**Impact**: ALTO (desarrollo imposible con reload loop)  
**Fix**: 5 líneas en vite.config.js
