const logger = require('../utils/logger');

/**
 * Global error handling middleware
 * 
 * @param {Error} err - The error object
 * @param {Request} req - Express request object
 * @param {Response} res - Express response object
 * @param {NextFunction} next - Express next function
 */
function errorHandler(err, req, res, next) {
  // Log the error
  logger.error('Error occurred', {
    error: err.message,
    stack: err.stack,
    path: req.path,
    method: req.method,
    ip: req.ip,
  });

  // Set default error status and message
  const status = err.statusCode || 500;
  const message = err.message || 'Internal Server Error';
  
  // Handle different types of errors
  if (err.name === 'ValidationError') {
    return res.status(400).json({
      error: 'Validation Error',
      details: err.details || message,
    });
  }
  
  if (err.name === 'UnauthorizedError') {
    return res.status(401).json({
      error: 'Unauthorized',
      message: 'Authentication required',
    });
  }
  
  if (err.name === 'ForbiddenError') {
    return res.status(403).json({
      error: 'Forbidden',
      message: 'Access denied',
    });
  }
  
  if (err.name === 'NotFoundError') {
    return res.status(404).json({
      error: 'Not Found',
      message: message,
    });
  }
  
  // Generic error response
  res.status(status).json({
    error: 'Server Error',
    message: process.env.NODE_ENV === 'production' ? 'An unexpected error occurred' : message,
  });
}

module.exports = errorHandler; 