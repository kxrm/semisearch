const express = require('express');
const mongoose = require('mongoose');
const dotenv = require('dotenv');
const logger = require('./utils/logger');
const routes = require('./routes');
const errorHandler = require('./middleware/errorHandler');

// Load environment variables
dotenv.config();

// Create Express app
const app = express();
const PORT = process.env.PORT || 3000;
const MONGODB_URI = process.env.MONGODB_URI || 'mongodb://localhost:27017/test-db';

// Middleware
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Routes
app.use('/api', routes);

// Error handling
app.use(errorHandler);

// TODO: Add rate limiting middleware
// TODO: Add authentication middleware

// Connect to MongoDB
async function connectToDatabase() {
  try {
    await mongoose.connect(MONGODB_URI, {
      useNewUrlParser: true,
      useUnifiedTopology: true,
    });
    logger.info('Connected to MongoDB');
  } catch (error) {
    logger.error('Failed to connect to MongoDB', { error });
    process.exit(1);
  }
}

// Start server
async function startServer() {
  await connectToDatabase();
  
  app.listen(PORT, () => {
    logger.info(`Server running on port ${PORT}`);
  });
}

// Handle uncaught exceptions
process.on('uncaughtException', (error) => {
  logger.error('Uncaught exception', { error });
  process.exit(1);
});

// Handle unhandled promise rejections
process.on('unhandledRejection', (error) => {
  logger.error('Unhandled rejection', { error });
  process.exit(1);
});

// Start the server
startServer().catch((error) => {
  logger.error('Failed to start server', { error });
  process.exit(1);
});

// FIXME: Implement graceful shutdown
// FIXME: Add proper error handling for database connection 