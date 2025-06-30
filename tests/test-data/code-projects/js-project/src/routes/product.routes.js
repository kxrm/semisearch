const express = require('express');
const productController = require('../controllers/product.controller');

const router = express.Router();

/**
 * @route GET /api/products
 * @desc Get all products
 * @access Public
 */
router.get('/', productController.getAllProducts);

/**
 * @route GET /api/products/:id
 * @desc Get product by ID
 * @access Public
 */
router.get('/:id', productController.getProductById);

/**
 * @route POST /api/products
 * @desc Create a new product
 * @access Private
 */
router.post('/', productController.createProduct);

/**
 * @route PUT /api/products/:id
 * @desc Update a product
 * @access Private
 */
router.put('/:id', productController.updateProduct);

/**
 * @route DELETE /api/products/:id
 * @desc Delete a product
 * @access Private
 */
router.delete('/:id', productController.deleteProduct);

/**
 * @route GET /api/products/category/:category
 * @desc Get products by category
 * @access Public
 */
router.get('/category/:category', productController.getProductsByCategory);

// FIXME: Add proper validation middleware
// FIXME: Implement pagination

module.exports = router; 