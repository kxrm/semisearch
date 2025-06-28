# Authentication

This document explains how authentication works with our API.

## API Keys

All API requests must include an API key for authentication. API keys are long, random strings that identify your application or project when making API calls.

### Obtaining an API Key

To get an API key:

1. Sign up for a developer account at [developer.example.com](https://developer.example.com)
2. Navigate to the API Keys section in your dashboard
3. Click "Create New API Key"
4. Give your key a name and select the appropriate permission scopes
5. Copy and securely store your API key

**Important**: API keys should be kept secure and never exposed in client-side code or public repositories.

### Including the API Key in Requests

Include your API key in the `Authorization` header of each request:

```
Authorization: Bearer YOUR_API_KEY
```

Example using cURL:

```bash
curl -X GET "https://api.example.com/v1/users" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

Example using JavaScript:

```javascript
fetch('https://api.example.com/v1/users', {
  headers: {
    'Authorization': 'Bearer YOUR_API_KEY'
  }
})
.then(response => response.json())
.then(data => console.log(data));
```

## Permission Scopes

API keys can be restricted to specific permission scopes. This allows you to create keys with limited access for different purposes. Available scopes include:

- `read:users` - Read user data
- `write:users` - Create or modify user data
- `read:products` - Read product data
- `write:products` - Create or modify product data
- `read:orders` - Read order data
- `write:orders` - Create or modify order data

When creating an API key, select only the scopes your application needs.

## Rate Limiting

API requests are subject to rate limiting based on your subscription tier. Rate limit information is included in response headers:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1620000000
```

If you exceed your rate limit, the API will return a `429 Too Many Requests` response.

## Error Responses

Authentication errors will return appropriate HTTP status codes:

- `401 Unauthorized`: Missing or invalid API key
- `403 Forbidden`: Valid API key but insufficient permissions
- `429 Too Many Requests`: Rate limit exceeded

Example error response:

```json
{
  "status": "error",
  "error": {
    "code": "unauthorized",
    "message": "Invalid or missing API key"
  }
}
```

## Best Practices

1. **Keep API keys secure**: Never expose your API key in client-side code or public repositories
2. **Use environment variables**: Store API keys in environment variables, not in your code
3. **Limit permissions**: Create keys with only the permissions they need
4. **Rotate keys regularly**: Generate new API keys periodically and revoke old ones
5. **Monitor usage**: Regularly check your API usage for unusual patterns

## Need Help?

If you're having trouble with authentication, please contact our developer support team at api-support@example.com. 