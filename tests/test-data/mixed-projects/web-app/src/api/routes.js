/**
 * API Routes for the Test Web App
 * @module api/routes
 */

const express = require('express');
const userController = require('./controllers/userController');
const productController = require('./controllers/productController');
const authMiddleware = require('./middleware/auth');

const router = express.Router();

/**
 * @route GET /api
 * @desc API status endpoint
 * @access Public
 */
router.get('/', (req, res) => {
  res.json({
    status: 'API is running',
    version: '1.0.0',
    documentation: '/docs/api'
  });
});

// User routes
/**
 * @route GET /api/users
 * @desc Get all users
 * @access Private (admin only)
 */
router.get('/users', authMiddleware.isAdmin, userController.getAllUsers);

/**
 * @route GET /api/users/:id
 * @desc Get user by ID
 * @access Private
 */
router.get('/users/:id', authMiddleware.isAuthenticated, userController.getUserById);

/**
 * @route POST /api/users
 * @desc Create a new user
 * @access Public
 */
router.post('/users', userController.createUser);

/**
 * @route PUT /api/users/:id
 * @desc Update a user
 * @access Private
 */
router.put('/users/:id', authMiddleware.isAuthenticated, userController.updateUser);

/**
 * @route DELETE /api/users/:id
 * @desc Delete a user
 * @access Private (admin only)
 */
router.delete('/users/:id', authMiddleware.isAdmin, userController.deleteUser);

// Product routes
/**
 * @route GET /api/products
 * @desc Get all products
 * @access Public
 */
router.get('/products', productController.getAllProducts);

/**
 * @route GET /api/products/:id
 * @desc Get product by ID
 * @access Public
 */
router.get('/products/:id', productController.getProductById);

/**
 * @route POST /api/products
 * @desc Create a new product
 * @access Private (admin only)
 */
router.post('/products', authMiddleware.isAdmin, productController.createProduct);

/**
 * @route PUT /api/products/:id
 * @desc Update a product
 * @access Private (admin only)
 */
router.put('/products/:id', authMiddleware.isAdmin, productController.updateProduct);

/**
 * @route DELETE /api/products/:id
 * @desc Delete a product
 * @access Private (admin only)
 */
router.delete('/products/:id', authMiddleware.isAdmin, productController.deleteProduct);

// Authentication routes
/**
 * @route POST /api/auth/login
 * @desc User login
 * @access Public
 */
router.post('/auth/login', userController.login);

/**
 * @route POST /api/auth/register
 * @desc User registration
 * @access Public
 */
router.post('/auth/register', userController.register);

/**
 * @route POST /api/auth/logout
 * @desc User logout
 * @access Private
 */
router.post('/auth/logout', authMiddleware.isAuthenticated, userController.logout);

// FIXME: Add proper error handling middleware
// TODO: Add rate limiting to prevent abuse

module.exports = router; 