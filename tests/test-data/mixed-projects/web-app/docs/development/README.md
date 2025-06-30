# Development Guide

This document provides information for developers working on the Test Web App project.

## Development Environment Setup

### Prerequisites

- Node.js 14.x or higher
- MongoDB 4.x or higher
- Git
- npm or yarn
- Code editor (VSCode recommended)

### Setup Steps

1. Clone the repository:

```bash
git clone https://github.com/example/test-web-app.git
cd test-web-app
```

2. Install dependencies:

```bash
npm install
```

3. Create a `.env` file in the root directory:

```
PORT=3000
MONGODB_URI=mongodb://localhost:27017/test-web-app
JWT_SECRET=your_jwt_secret
NODE_ENV=development
```

4. Start MongoDB:

```bash
# Using Docker
docker run -d -p 27017:27017 --name mongodb mongo:latest

# Or using a local installation
mongod
```

5. Start the development server:

```bash
npm run dev
```

## Project Structure

```
test-web-app/
├── docs/              # Documentation
├── src/               # Source code
│   ├── api/           # API endpoints and controllers
│   │   ├── controllers/ # Request handlers
│   │   ├── middleware/  # Express middleware
│   │   └── routes.js    # API routes
│   ├── components/    # React components
│   ├── models/        # Mongoose models
│   ├── utils/         # Utility functions
│   └── index.js       # Application entry point
├── public/            # Static files
├── tests/             # Test files
├── .env               # Environment variables (not in git)
├── .gitignore         # Git ignore file
├── package.json       # Project dependencies
└── README.md          # Project overview
```

## Coding Standards

### JavaScript

- Use ES6+ features
- Follow Airbnb JavaScript Style Guide
- Use JSDoc for documentation
- Use async/await for asynchronous code

### React

- Use functional components with hooks
- Use PropTypes for type checking
- Follow component file structure:
  - One component per file
  - Named exports
  - Component, styles, and tests in the same directory

### API Development

- Follow RESTful principles
- Use descriptive route names
- Include proper error handling
- Document all endpoints with JSDoc

## Testing

### Running Tests

```bash
# Run all tests
npm test

# Run specific test file
npm test -- tests/api/users.test.js

# Run tests with coverage
npm test -- --coverage
```

### Writing Tests

- Place tests in the `tests` directory
- Follow the same directory structure as the source code
- Use descriptive test names
- Test both success and error cases

## Build Process

### Development Build

```bash
npm run dev
```

### Production Build

```bash
npm run build
```

### Deployment

1. Build the application:

```bash
npm run build
```

2. Start the production server:

```bash
npm start
```

## Continuous Integration

The project uses GitHub Actions for CI/CD:

- Tests run on every push and pull request
- Linting checks ensure code quality
- Automatic deployment to staging on merge to `develop`
- Automatic deployment to production on merge to `main`

## Contributing

1. Create a new branch from `develop`:

```bash
git checkout -b feature/your-feature-name
```

2. Make your changes and commit:

```bash
git add .
git commit -m "feat: add your feature"
```

3. Push to your branch:

```bash
git push origin feature/your-feature-name
```

4. Create a pull request to the `develop` branch

## Documentation

- Update documentation when making changes
- Generate API documentation:

```bash
npm run docs
```

## Troubleshooting

### Common Issues

- **MongoDB connection errors**: Ensure MongoDB is running and the connection string is correct
- **Missing dependencies**: Run `npm install` to update dependencies
- **Port already in use**: Change the PORT in .env or kill the process using the port 