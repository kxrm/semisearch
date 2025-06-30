# API Endpoints Reference

This section provides detailed documentation for all available API endpoints.

## User Management

| Endpoint | Method | Description |
|----------|--------|-------------|
| [/users](./users/list.md) | GET | List all users |
| [/users/{id}](./users/get.md) | GET | Get a specific user |
| [/users](./users/create.md) | POST | Create a new user |
| [/users/{id}](./users/update.md) | PUT | Update a user |
| [/users/{id}](./users/delete.md) | DELETE | Delete a user |

## Product Catalog

| Endpoint | Method | Description |
|----------|--------|-------------|
| [/products](./products/list.md) | GET | List all products |
| [/products/{id}](./products/get.md) | GET | Get a specific product |
| [/products](./products/create.md) | POST | Create a new product |
| [/products/{id}](./products/update.md) | PUT | Update a product |
| [/products/{id}](./products/delete.md) | DELETE | Delete a product |
| [/products/categories](./products/list-categories.md) | GET | List product categories |

## Order Processing

| Endpoint | Method | Description |
|----------|--------|-------------|
| [/orders](./orders/list.md) | GET | List all orders |
| [/orders/{id}](./orders/get.md) | GET | Get a specific order |
| [/orders](./orders/create.md) | POST | Create a new order |
| [/orders/{id}](./orders/update.md) | PUT | Update an order |
| [/orders/{id}/cancel](./orders/cancel.md) | POST | Cancel an order |

## Payment Processing

| Endpoint | Method | Description |
|----------|--------|-------------|
| [/payments](./payments/list.md) | GET | List all payments |
| [/payments/{id}](./payments/get.md) | GET | Get a specific payment |
| [/payments](./payments/create.md) | POST | Create a new payment |
| [/payments/{id}/refund](./payments/refund.md) | POST | Refund a payment |

## Analytics

| Endpoint | Method | Description |
|----------|--------|-------------|
| [/analytics/users](./analytics/users.md) | GET | Get user statistics |
| [/analytics/orders](./analytics/orders.md) | GET | Get order statistics |
| [/analytics/revenue](./analytics/revenue.md) | GET | Get revenue statistics |

## Common Parameters

Many endpoints support the following query parameters:

### Pagination

- `page`: Page number (default: 1)
- `limit`: Number of items per page (default: 20, max: 100)

### Filtering

- `filter[field]`: Filter by field value (e.g., `filter[status]=active`)
- `search`: Search term for text search across relevant fields

### Sorting

- `sort`: Field to sort by, prefix with `-` for descending order (e.g., `sort=-created_at`)

### Field Selection

- `fields`: Comma-separated list of fields to include in the response

### Example

```
GET /users?page=2&limit=50&filter[status]=active&sort=-created_at&fields=id,name,email
```

This request would:
- Return the second page of results
- Include 50 users per page
- Only include active users
- Sort by creation date (newest first)
- Only include the id, name, and email fields in the response 