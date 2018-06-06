/* global jest */
global.fetch = require('jest-fetch-mock')
global.Date.now = jest.fn(() => 1526404200000)
