# Quick Start Guide

This guide will help you get started with our API in just a few minutes.

## Prerequisites

Before you begin, make sure you have:

1. A developer account (sign up at [developer.example.com](https://developer.example.com))
2. An API key (create one in your developer dashboard)
3. Basic knowledge of making HTTP requests

## Step 1: Authentication

All API requests require authentication using your API key. Include it in the `Authorization` header:

```
Authorization: Bearer YOUR_API_KEY
```

## Step 2: Make Your First API Request

Let's make a simple request to get a list of users:

### Using cURL

```bash
curl -X GET "https://api.example.com/v1/users" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

### Using JavaScript

```javascript
fetch('https://api.example.com/v1/users', {
  headers: {
    'Authorization': 'Bearer YOUR_API_KEY'
  }
})
.then(response => response.json())
.then(data => console.log(data));
```

### Using Python

```python
import requests

url = "https://api.example.com/v1/users"
headers = {
    "Authorization": "Bearer YOUR_API_KEY"
}

response = requests.get(url, headers=headers)
data = response.json()
print(data)
```

## Step 3: Create a Resource

Now let's create a new user:

### Using cURL

```bash
curl -X POST "https://api.example.com/v1/users" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "John Doe",
    "email": "john@example.com",
    "role": "user"
  }'
```

### Using JavaScript

```javascript
fetch('https://api.example.com/v1/users', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_API_KEY',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    name: 'John Doe',
    email: 'john@example.com',
    role: 'user'
  })
})
.then(response => response.json())
.then(data => console.log(data));
```

### Using Python

```python
import requests

url = "https://api.example.com/v1/users"
headers = {
    "Authorization": "Bearer YOUR_API_KEY",
    "Content-Type": "application/json"
}
payload = {
    "name": "John Doe",
    "email": "john@example.com",
    "role": "user"
}

response = requests.post(url, headers=headers, json=payload)
data = response.json()
print(data)
```

## Step 4: Handle Errors

Always check for error responses from the API:

```javascript
fetch('https://api.example.com/v1/users', {
  headers: {
    'Authorization': 'Bearer YOUR_API_KEY'
  }
})
.then(response => {
  if (!response.ok) {
    throw new Error(`HTTP error! Status: ${response.status}`);
  }
  return response.json();
})
.then(data => console.log(data))
.catch(error => console.error('Error:', error));
```

## Next Steps

Now that you've made your first API requests, you can:

1. Explore the [API Endpoints Reference](../endpoints/README.md) for all available endpoints
2. Learn about [Authentication](./authentication.md) in more detail
3. Check out the [User Management Guide](./user-management.md) for more complex operations
4. Set up [Webhooks](./webhooks.md) to receive real-time notifications

## Need Help?

If you run into any issues, check out our [Error Handling Guide](./error-handling.md) or contact our developer support team at api-support@example.com. 