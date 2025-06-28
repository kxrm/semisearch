# API Documentation

This document provides detailed information about the Test Web App API endpoints.

## Base URL

All API endpoints are relative to:

```
http://localhost:3000/api
```

## Authentication

Most API endpoints require authentication. Include a JWT token in the Authorization header:

```
Authorization: Bearer YOUR_JWT_TOKEN
```

To obtain a JWT token, use the login endpoint.

## Endpoints

### Authentication

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /auth/login | User login |
| POST | /auth/register | User registration |
| POST | /auth/logout | User logout |

#### POST /auth/login

Login with username/email and password to get a JWT token.

**Request Body:**

```json
{
  "email": "user@example.com",
  "password": "password123"
}
```

**Response:**

```json
{
  "success": true,
  "token": "YOUR_JWT_TOKEN",
  "user": {
    "id": "user_id",
    "username": "username",
    "email": "user@example.com",
    "role": "user"
  }
}
```

### Users

| Method | Endpoint | Description | Access |
|--------|----------|-------------|--------|
| GET | /users | Get all users | Admin |
| GET | /users/:id | Get user by ID | Authenticated |
| POST | /users | Create a new user | Public |
| PUT | /users/:id | Update a user | Authenticated |
| DELETE | /users/:id | Delete a user | Admin |

#### GET /users

Get a list of all users (admin only).

**Response:**

```json
{
  "success": true,
  "count": 2,
  "data": [
    {
      "id": "user_id_1",
      "username": "user1",
      "email": "user1@example.com",
      "role": "user"
    },
    {
      "id": "user_id_2",
      "username": "admin",
      "email": "admin@example.com",
      "role": "admin"
    }
  ]
}
```

### Products

| Method | Endpoint | Description | Access |
|--------|----------|-------------|--------|
| GET | /products | Get all products | Public |
| GET | /products/:id | Get product by ID | Public |
| POST | /products | Create a new product | Admin |
| PUT | /products/:id | Update a product | Admin |
| DELETE | /products/:id | Delete a product | Admin |

#### GET /products

Get a list of all products.

**Response:**

```json
{
  "success": true,
  "count": 2,
  "data": [
    {
      "id": "product_id_1",
      "name": "Product 1",
      "price": 19.99,
      "description": "Description of product 1"
    },
    {
      "id": "product_id_2",
      "name": "Product 2",
      "price": 29.99,
      "description": "Description of product 2"
    }
  ]
}
```

## Error Handling

All API endpoints return appropriate HTTP status codes:

- 200: Success
- 201: Created
- 400: Bad Request
- 401: Unauthorized
- 403: Forbidden
- 404: Not Found
- 500: Server Error

Error responses have the following format:

```json
{
  "success": false,
  "error": {
    "message": "Error message",
    "code": "ERROR_CODE"
  }
}
```

## Rate Limiting

API requests are limited to 100 requests per minute per IP address. When exceeded, a 429 Too Many Requests response will be returned. 