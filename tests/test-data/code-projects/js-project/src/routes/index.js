const express = require('express');
const userRoutes = require('./user.routes');
const productRoutes = require('./product.routes');

const router = express.Router();

// Health check endpoint
router.get('/health', (req, res) => {
  res.status(200).json({ status: 'ok' });
});

// API version
router.get('/version', (req, res) => {
  res.status(200).json({ version: '1.0.0' });
});

// Mount routes
router.use('/users', userRoutes);
router.use('/products', productRoutes);

// Fallback for undefined routes
router.use('*', (req, res) => {
  res.status(404).json({ error: 'Endpoint not found' });
});

module.exports = router; 