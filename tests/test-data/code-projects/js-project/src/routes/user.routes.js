const express = require('express');
const userController = require('../controllers/user.controller');

const router = express.Router();

/**
 * @route GET /api/users
 * @desc Get all users
 * @access Private
 */
router.get('/', userController.getAllUsers);

/**
 * @route GET /api/users/:id
 * @desc Get user by ID
 * @access Private
 */
router.get('/:id', userController.getUserById);

/**
 * @route POST /api/users
 * @desc Create a new user
 * @access Private
 */
router.post('/', userController.createUser);

/**
 * @route PUT /api/users/:id
 * @desc Update a user
 * @access Private
 */
router.put('/:id', userController.updateUser);

/**
 * @route DELETE /api/users/:id
 * @desc Delete a user
 * @access Private
 */
router.delete('/:id', userController.deleteUser);

// TODO: Add route for user authentication
// TODO: Add route for password reset

module.exports = router; 