# Test Web App

A mock web application with both code and documentation for testing purposes.

## Overview

This project contains a simple web application with:

- Backend API built with Express.js
- Frontend UI built with React
- MongoDB database integration
- Comprehensive documentation

## Project Structure

```
web-app/
├── docs/              # Documentation
│   ├── api/           # API documentation
│   ├── guides/        # User guides
│   └── development/   # Development documentation
├── src/               # Source code
│   ├── api/           # Backend API code
│   ├── components/    # React components
│   ├── models/        # Data models
│   ├── utils/         # Utility functions
│   └── index.js       # Application entry point
├── tests/             # Test files
├── package.json       # Project dependencies
└── README.md          # This file
```

## Getting Started

### Prerequisites

- Node.js 14.x or higher
- MongoDB 4.x or higher
- npm or yarn

### Installation

1. Clone the repository
2. Install dependencies:

```bash
npm install
```

3. Create a `.env` file with your configuration:

```
PORT=3000
MONGODB_URI=mongodb://localhost:27017/test-web-app
```

4. Start the development server:

```bash
npm run dev
```

## Documentation

Comprehensive documentation is available in the `docs` directory:

- [API Documentation](./docs/api/README.md)
- [User Guides](./docs/guides/README.md)
- [Development Documentation](./docs/development/README.md)

You can also generate the documentation website:

```bash
npm run docs
```

Then open `./docs/generated/index.html` in your browser.

## Features

- User authentication and authorization
- CRUD operations for resources
- RESTful API
- Responsive UI
- Comprehensive documentation

## Development

### Running Tests

```bash
npm test
```

### Building for Production

```bash
npm run build
```

## License

This project is licensed under the MIT License - see the LICENSE file for details. 