# Documentation

## Client Class

The main entry point for making requests.

### Constructor

```typescript
new Client(config?: ClientConfig)
```

**ClientConfig Options:**

| Option | Type | Description |
| :--- | :--- | :--- |
| `auth` | `string[]` | Basic auth credentials `['user', 'pass']` |
| `auth_bearer` | `string` | Bearer token |
| `headers` | `Record<string, string>` | Default headers for all requests |
| `cookie_store` | `boolean` | Enable persistent cookie storage (default: `true`) |
| `referer` | `boolean` | Automatic referer header (default: `true`) |
| `proxy` | `string` | Proxy URL (e.g., `http://user:pass@host:port`) |
| `timeout` | `number` | Request timeout in seconds |
| `impersonate` | `string` | Browser to impersonate (e.g., `chrome_120`, `safari_17_0`) |
| `impersonate_os` | `string` | OS to impersonate (`windows`, `macos`, `linux`, `android`, `ios`) |
| `follow_redirects` | `boolean` | Follow HTTP redirects (default: `true`) |
| `max_redirects` | `number` | Maximum redirects (default: `20`) |
| `verify` | `boolean` | Verify SSL certificates (default: `true`) |
| `ca_cert_file` | `string` | Path to custom CA certificate file |
| `https_only` | `boolean` | Enforce HTTPS only |
| `http2_only` | `boolean` | Enforce HTTP/2 only |

### Methods

#### `request`

```typescript
request(
    method: HttpMethod,
    url: string,
    params?: Record<string, string>,
    headers?: Record<string, string>,
    cookies?: Record<string, string>,
    content?: Buffer,
    data?: Record<string, string>,
    json?: any,
    files?: Record<string, string>,
    auth?: string[],
    auth_bearer?: string,
    timeout?: number
): Promise<Response>
```

Sends an HTTP request.

- **method**: `HttpMethod.GET`, `HttpMethod.POST`, etc.
- **url**: Target URL.
- **params**: Query string parameters.
- **headers**: Request headers.
- **cookies**: Request cookies.
- **content**: Raw body content (Buffer).
- **data**: Form URL encoded data.
- **json**: JSON body data.
- **files**: Multipart form files `{ "field_name": "/path/to/file" }`.

## Response Class

Returned by `client.request()`.

### Properties

- `url`: The final URL (after redirects).
- `statusCode`: HTTP status code.

### Methods

- `headers()`: Returns request headers as key-value pairs.
- `cookies()`: Returns cookies as key-value pairs.
- `text()`: Returns body as string (decoded).
- `json()`: Returns body parsed as JSON.
- `content()`: Returns body as raw Buffer.

## HttpMethod Enum

```typescript
enum HttpMethod {
  GET = 0,
  HEAD = 1,
  OPTIONS = 2,
  DELETE = 3,
  POST = 4,
  PUT = 5,
  PATCH = 6
}
```
