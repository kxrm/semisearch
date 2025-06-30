/**
 * Main entry point for the Test Web App
 * @module index
 */

const express = require('express');
const mongoose = require('mongoose');
const path = require('path');
const apiRoutes = require('./api/routes');
const { logger } = require('./utils/logger');
const config = require('./utils/config');

// Create Express app
const app = express();
const PORT = config.PORT || 3000;

// Middleware
app.use(express.json());
app.use(express.urlencoded({ extended: true }));
app.use(express.static(path.join(__dirname, '../public')));

// API Routes
app.use('/api', apiRoutes);

// Serve React app for all other routes
app.get('*', (req, res) => {
  res.sendFile(path.join(__dirname, '../public/index.html'));
});

/**
 * Connect to MongoDB and start the server
 * @async
 * @function startServer
 * @returns {Promise<void>}
 */
async function startServer() {
  try {
    // Connect to MongoDB
    await mongoose.connect(config.MONGODB_URI, {
      useNewUrlParser: true,
      useUnifiedTopology: true,
    });
    logger.info('Connected to MongoDB');
    
    // Start the server
    app.listen(PORT, () => {
      logger.info(`Server running on port ${PORT}`);
      logger.info(`API available at http://localhost:${PORT}/api`);
    });
  } catch (error) {
    logger.error('Failed to start server:', error);
    process.exit(1);
  }
}

// Handle uncaught exceptions
process.on('uncaughtException', (error) => {
  logger.error('Uncaught exception:', error);
  process.exit(1);
});

// Handle unhandled promise rejections
process.on('unhandledRejection', (error) => {
  logger.error('Unhandled rejection:', error);
  process.exit(1);
});

// Start the server
startServer();

// TODO: Implement graceful shutdown
// TODO: Add clustering for production

module.exports = app; 